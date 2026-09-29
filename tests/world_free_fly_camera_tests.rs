#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/prism.rs"]
    pub mod prism;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/reflection.rs"]
    pub mod reflection;
    #[path = "../src/core/refraction.rs"]
    pub mod refraction;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_free_fly.rs"]
    pub mod world_free_fly;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/block.rs"]
    pub mod block;
    #[path = "../src/scene/block_geometry.rs"]
    pub mod block_geometry;
    #[path = "../src/scene/block_shape_factory.rs"]
    pub mod block_shape_factory;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/light.rs"]
    pub mod light;
    #[path = "../src/scene/material_gallery.rs"]
    pub mod material_gallery;
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/overworld.rs"]
    pub mod overworld;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/rhombus.rs"]
    pub mod rhombus;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
    #[path = "../src/scene/world.rs"]
    pub mod world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use camera::diagnostic::DiagnosticCameraState;
use camera::world_free_fly::{
    FREE_FLY_MAX_PITCH, FreeFlyPose, WorldFreeFlyCameraState, WorldRealm, angles_for, heading,
    look_direction,
};
use core::math::Vec3;
use scene::world::{WorldScene, world_camera, world_free_fly_camera};

const ASPECT: f32 = 4.0 / 3.0;

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

fn unit(v: Vec3) -> bool {
    finite(v) && (v.length() - 1.0).abs() < 1e-4
}

#[test]
fn the_initial_pose_is_finite_and_matches_the_official_framing() {
    let state = world_free_fly_camera();
    assert!(finite(state.position));
    assert!(state.yaw.is_finite() && state.pitch.is_finite());
    let official = world_camera(ASPECT);
    assert_eq!(state.position, official.position);
    // It looks where the official framing looks.
    let expected = (official.target - official.position).normalize();
    assert!((state.forward() - expected).length() < 1e-3);
    let camera = state.camera(ASPECT);
    assert_eq!(camera.position, official.position);
    assert!((camera.basis().forward - expected).length() < 1e-3);
    assert_eq!(camera.fov, 60.0);
}

#[test]
fn the_initial_realm_is_overworld_with_up_positive_y() {
    let state = world_free_fly_camera();
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(WorldRealm::Overworld.up(), Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(WorldRealm::RedBlack.up(), Vec3::new(0.0, -1.0, 0.0));
    assert!(state.up().y > 0.5);
}

#[test]
fn forward_right_and_up_form_an_orthonormal_right_handed_basis() {
    for (yaw, pitch) in [(0.0, 0.0), (1.2, 0.4), (-2.5, -0.9), (3.0, 1.5)] {
        for realm in [WorldRealm::Overworld, WorldRealm::RedBlack] {
            let mut state = WorldFreeFlyCameraState::from_pose(FreeFlyPose {
                position: Vec3::new(1.0, 2.0, 3.0),
                yaw,
                pitch,
            });
            state.realm = realm;
            state.local_up = realm.up();
            let (f, r, u) = (state.forward(), state.right(), state.up());
            assert!(unit(f) && unit(r) && unit(u), "{yaw} {pitch} {realm:?}");
            assert!(f.dot(r).abs() < 1e-4 && f.dot(u).abs() < 1e-4 && r.dot(u).abs() < 1e-4);
            // Right-handed: forward x up = right, and up points toward the
            // realm's vertical.
            assert!((f.cross(u) - r).length() < 1e-4);
            assert!(u.dot(realm.up()) > 0.0);
            // The camera built from the state uses that same basis.
            let basis = state.camera(ASPECT).basis();
            assert!((basis.forward - f).length() < 1e-4);
            assert!((basis.right - r).length() < 1e-4);
            assert!((basis.up - u).length() < 1e-4);
        }
    }
}

#[test]
fn yaw_and_pitch_conventions_round_trip() {
    assert!((heading(0.0) - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);
    assert!((heading(std::f32::consts::FRAC_PI_2) - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);
    let up = Vec3::new(0.0, 1.0, 0.0);
    for (yaw, pitch) in [(0.3, 0.2), (-1.1, -0.7), (2.9, 1.2)] {
        let f = look_direction(yaw, pitch, up);
        let (y2, p2) = angles_for(f, up);
        assert!((look_direction(y2, p2, up) - f).length() < 1e-4);
        assert!((p2 - pitch).abs() < 1e-4);
    }
    assert!(FREE_FLY_MAX_PITCH < std::f32::consts::FRAC_PI_2);
    assert!(FREE_FLY_MAX_PITCH > 1.5);
}

#[test]
fn reset_restores_the_pose_and_the_overworld_frame() {
    let mut state = world_free_fly_camera();
    let home = state.home();
    state.position = Vec3::new(-40.0, -30.0, 9.0);
    state.yaw += 2.0;
    state.pitch = 0.9;
    state.realm = WorldRealm::RedBlack;
    state.local_up = WorldRealm::RedBlack.up();
    state.reset();
    assert_eq!(state.position, home.position);
    assert_eq!(state.yaw, home.yaw);
    assert_eq!(state.pitch, home.pitch);
    assert_eq!(state.realm, WorldRealm::Overworld);
    assert_eq!(state.local_up, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(state.home(), home);
}

#[test]
fn the_catalog_keeps_its_diagnostic_camera_and_the_world_builds_its_own() {
    // The diagnostic orbit camera type still exists and still starts from
    // the gallery framing.
    let gallery = scene::material_gallery::gallery_camera(ASPECT);
    let view = DiagnosticCameraState::from_pose(gallery.position, gallery.target);
    assert!((view.build_camera(ASPECT).position - gallery.position).length() < 1e-3);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog =
        &app[app.find("struct CatalogMode").unwrap()..app.find("struct WorldMode").unwrap()];
    assert!(!catalog.contains("WorldFreeFlyCameraState"));
    let world =
        &app[app.find("struct WorldMode").unwrap()..app.find("/// Casts one primary ray").unwrap()];
    assert!(world.contains("free_fly: WorldFreeFlyCameraState"));
    // Building the camera does not touch the world.
    let before = WorldScene::new();
    let _ = world_free_fly_camera();
    let after = WorldScene::new();
    assert_eq!(before.world().len(), after.world().len());
    assert_eq!(after.world().len(), 6784);
}

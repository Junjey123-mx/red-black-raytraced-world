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
    #[path = "../src/camera/portal_crossing.rs"]
    pub mod portal_crossing;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_collision.rs"]
    pub mod world_collision;
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
    #[path = "../src/scene/castle.rs"]
    pub mod castle;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm};
use core::math::Vec3;
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, trace_ray};
use renderer::skybox::{Background, CUBEMAP_FACE_NAMES, SkyCubemap, cubemap_face_uv};
use scene::environment::{WorldEnvironmentProfile, WorldSkyboxes};
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, sun_direction,
    world_lights, world_materials,
};
use std::collections::HashSet;

const SKYBOX_DIR: &str = "assets/textures/skybox";

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn skies(manager: &mut TextureManager) -> WorldSkyboxes {
    WorldSkyboxes::load(manager, SKYBOX_DIR).expect("skybox faces")
}

fn cube(skies: &WorldSkyboxes, realm: WorldRealm) -> SkyCubemap {
    let (faces, profile) = match realm {
        WorldRealm::Overworld => (skies.day, WorldEnvironmentProfile::DAY),
        WorldRealm::RedBlack => (skies.red_black, WorldEnvironmentProfile::RED_BLACK),
    };
    SkyCubemap {
        from: faces,
        to: faces,
        blend: 0.0,
        up: realm.up(),
        fallback: profile.sky(realm.up()),
    }
}

// 1
#[test]
fn each_realm_has_six_square_faces_on_disk() {
    let mut manager = TextureManager::new();
    let s = skies(&mut manager);
    assert_eq!(CUBEMAP_FACE_NAMES, ["px", "nx", "py", "ny", "pz", "nz"]);
    let ids: HashSet<_> = s.day.0.iter().chain(s.red_black.0.iter()).collect();
    assert_eq!(ids.len(), 12, "twelve distinct images");
    for id in s.day.0.iter().chain(s.red_black.0.iter()) {
        let t = manager.get(*id).unwrap();
        assert_eq!(t.width(), t.height());
        assert!(t.width() >= 64);
    }
}

// 2
#[test]
fn a_direction_selects_its_face_and_texel() {
    for (i, d) in [
        v(1.0, 0.0, 0.0),
        v(-1.0, 0.0, 0.0),
        v(0.0, 1.0, 0.0),
        v(0.0, -1.0, 0.0),
        v(0.0, 0.0, 1.0),
        v(0.0, 0.0, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let (face, uv) = cubemap_face_uv(d);
        assert_eq!(face, i);
        assert!((uv.x - 0.5).abs() < 1e-6 && (uv.y - 0.5).abs() < 1e-6);
    }
    // Looking up on a side face moves toward the top of its image.
    assert!(cubemap_face_uv(v(0.2, 0.5, 1.0)).1.y < 0.5);
    let (face, uv) = cubemap_face_uv(v(0.3, -0.9, 0.2));
    assert_eq!(face, 3);
    assert!((0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y));
}

// 3
#[test]
fn the_skybox_is_an_image_not_a_color() {
    let mut manager = TextureManager::new();
    let s = skies(&mut manager);
    let distinct = |sky: &SkyCubemap, manager: &TextureManager| {
        let mut seen = HashSet::new();
        for i in 0..64 {
            for j in 0..64 {
                let d = v(i as f32 / 32.0 - 1.0, 1.0, j as f32 / 32.0 - 1.0);
                seen.insert(sky.color(d, manager).to_rgba8());
            }
        }
        seen.len()
    };
    let day = cube(&s, WorldRealm::Overworld);
    let night = cube(&s, WorldRealm::RedBlack);
    assert!(distinct(&day, &manager) >= 8, "clouds over the zenith");
    assert!(distinct(&night, &manager) >= 8, "stars over the zenith");
    // The sun sits where the scene's directional light comes from, and it
    // is the brightest thing in the day sky.
    let sun = day.color(sun_direction(), &manager);
    assert!(sun.r > 0.9 && sun.g > 0.85, "{sun:?}");
    let zenith_blue = day.color(v(-0.6, 1.0, -0.6), &manager);
    assert!(zenith_blue.b > zenith_blue.r, "{zenith_blue:?}");
    // The two realms keep their identity: celeste day, dark cornelian night.
    let (d, n) = (
        day.color(v(1.0, -0.2, 0.3), &manager),
        night.color(v(1.0, 0.2, 0.3), &manager),
    );
    assert!(d.b > 0.8 && d.r > 0.6, "{d:?}");
    assert!(n.r > n.g && n.r < 0.6 && n.g < 0.3, "{n:?}");
}

// 4
#[test]
fn faces_meet_without_a_seam() {
    let mut manager = TextureManager::new();
    let s = skies(&mut manager);
    let day = cube(&s, WorldRealm::Overworld);
    // Below the clouds the day sky is smooth: both sides of every cube
    // edge agree.
    for (a, b) in [
        (v(1.0, -0.3, 0.99), v(0.99, -0.3, 1.0)),
        (v(-1.0, -0.3, 0.99), v(-0.99, -0.3, 1.0)),
        (v(1.0, -0.3, -0.99), v(0.99, -0.3, -1.0)),
        (v(-1.0, -0.3, -0.99), v(-0.99, -0.3, -1.0)),
        (v(0.4, -1.0, 0.99), v(0.4, -0.99, 1.0)),
    ] {
        assert_ne!(cubemap_face_uv(a).0, cubemap_face_uv(b).0);
        let (ca, cb) = (day.color(a, &manager), day.color(b, &manager));
        let diff = (ca.r - cb.r).abs() + (ca.g - cb.g).abs() + (ca.b - cb.b).abs();
        assert!(diff < 0.03, "{a:?} {b:?}: {ca:?} vs {cb:?}");
    }
}

// 5
#[test]
fn the_inverted_realm_hangs_its_sky_along_minus_y() {
    let mut manager = TextureManager::new();
    let s = skies(&mut manager);
    let mut up = cube(&s, WorldRealm::RedBlack);
    up.up = v(0.0, 1.0, 0.0);
    let down = cube(&s, WorldRealm::RedBlack);
    assert_eq!(down.up, v(0.0, -1.0, 0.0));
    for d in [v(0.3, 0.8, -0.2), v(-0.7, 0.1, 0.5), v(0.1, -0.9, 0.4)] {
        assert_eq!(
            down.color(d, &manager),
            up.color(v(d.x, -d.y, -d.z), &manager)
        );
    }
    // A tilted sky (mid portal roll) is a rotation, not a jump.
    let mut tilted = cube(&s, WorldRealm::Overworld);
    tilted.up = v(1.0, 0.0, 0.0);
    assert_eq!(
        tilted.color(v(1.0, 0.0, 0.0), &manager),
        cube(&s, WorldRealm::Overworld).color(v(0.0, 1.0, 0.0), &manager)
    );
}

// 6
#[test]
fn miss_rays_sample_the_skybox_on_the_cpu() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let s = skies(&mut manager);
    let scene = WorldScene::new();
    let materials = world_materials(&textures);
    let lights = world_lights();
    let eye = v(12.0, 60.0, 12.0);
    for realm in [WorldRealm::Overworld, WorldRealm::RedBlack] {
        let mut state = WorldFreeFlyCameraState::looking_at(eye, v(12.0, 61.0, 13.0));
        state.realm = realm;
        state.local_up = realm.up();
        let background = state.skybox(&s);
        assert!(background.is_cubemap() && background.is_sky());
        let Background::Cubemap(sky) = background else {
            unreachable!()
        };
        assert_eq!(sky, cube(&s, realm));
        let view = VoxelScene {
            world: scene.world(),
            materials: &materials,
            camera_position: eye,
            lights: &lights,
            ambient_factor: WORLD_AMBIENT_FACTOR,
            background,
            texture_manager: &manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        for d in [v(0.2, 1.0, 0.1), v(-0.5, 0.6, 0.7), v(0.9, 0.3, -0.2)] {
            let ray = Ray::new(eye, d.normalize());
            assert_eq!(trace_ray(&view, &ray, 0), sky.color(d, &manager));
        }
    }
    // The window's World mode traces against the skybox.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("self.free_fly.skybox(&self.skies)"));
    assert!(app.contains("WorldSkyboxes::load(manager, SKYBOX_TEXTURES_DIR)"));
}

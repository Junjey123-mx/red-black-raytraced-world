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

use camera::portal_crossing::{
    PORTAL_CROSSING_EPSILON, PORTAL_REARM_DISTANCE, PortalCrossingDetector, PortalCrossingEvent,
    PortalVolume, detect_portal_crossing,
};
use camera::world_free_fly::WorldRealm;
use core::math::Vec3;
use scene::orientation::Orientation;
use scene::rhombus::RhombusConfig;
use scene::world::WorldScene;

fn volume() -> PortalVolume {
    PortalVolume::from_anchor(&RhombusConfig::official().portal)
}

/// A point in the opening's middle, `dz` in front of (+) or behind (-) the
/// membrane.
fn at(dz: f32) -> Vec3 {
    let v = volume();
    v.plane_point + v.normal * dz
}

#[test]
fn the_volume_is_derived_from_the_real_portal_contract() {
    let r = RhombusConfig::official();
    let p = r.portal;
    let v = volume();
    assert_eq!(p.facing, Orientation::South);
    assert_eq!(v.normal, Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(v.plane_point.z, p.wall_z as f32 + 0.5);
    assert_eq!(v.min_x, (p.min_x + 1) as f32);
    assert_eq!(v.max_x, p.max_x as f32);
    assert_eq!(v.min_y, (p.min_y + 1) as f32);
    assert_eq!(v.max_y, p.max_y as f32);
    // The opening covers exactly the core cells of the built scene.
    let scene = WorldScene::new();
    for cell in &scene.portal().core {
        let center = Vec3::new(
            cell.x as f32 + 0.5,
            cell.y as f32 + 0.5,
            cell.z as f32 + 0.5,
        );
        assert!(v.inside_aperture(center), "{cell:?}");
        assert!(v.signed_distance(center).abs() < 1e-4);
    }
    for cell in &scene.portal().frame {
        let center = Vec3::new(
            cell.x as f32 + 0.5,
            cell.y as f32 + 0.5,
            cell.z as f32 + 0.5,
        );
        assert!(
            !v.inside_aperture(center),
            "frame {cell:?} counts as opening"
        );
    }
    assert!(PORTAL_CROSSING_EPSILON > 0.0 && PORTAL_CROSSING_EPSILON < 0.25);
    assert!(PORTAL_REARM_DISTANCE >= 1.0);
}

#[test]
fn crossing_the_center_forward_and_backward_gives_both_events() {
    let v = volume();
    assert_eq!(
        detect_portal_crossing(at(2.0), at(-1.0), &v, WorldRealm::Overworld),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    assert_eq!(
        detect_portal_crossing(at(-2.0), at(1.0), &v, WorldRealm::RedBlack),
        Some(PortalCrossingEvent::RedBlackToOverworld)
    );
    // Direction follows the realm, whichever side is left.
    assert_eq!(
        detect_portal_crossing(at(-1.0), at(1.0), &v, WorldRealm::Overworld),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    let e = PortalCrossingEvent::OverworldToRedBlack;
    assert_eq!(
        (e.from_realm(), e.to_realm()),
        (WorldRealm::Overworld, WorldRealm::RedBlack)
    );
}

#[test]
fn movement_outside_the_opening_or_along_the_plane_never_triggers() {
    let v = volume();
    let realm = WorldRealm::Overworld;
    // Same plane, but beside the frame: e.g. under the diorama or past the
    // frame's edge.
    let beside = Vec3::new(v.min_x - 3.0, v.plane_point.y, 0.0);
    assert_eq!(
        detect_portal_crossing(beside + v.normal * 2.0, beside - v.normal * 2.0, &v, realm),
        None
    );
    let under = Vec3::new(v.plane_point.x, v.min_y - 20.0, 0.0);
    assert_eq!(
        detect_portal_crossing(under + v.normal * 3.0, under - v.normal * 3.0, &v, realm),
        None
    );
    // Through the frame post column, just outside the opening.
    let post = Vec3::new(v.min_x - 0.5, v.plane_point.y, v.plane_point.z);
    assert_eq!(
        detect_portal_crossing(post + v.normal, post - v.normal, &v, realm),
        None
    );
    // Grazing the inner edge inside the epsilon band.
    let edge = Vec3::new(
        v.min_x + PORTAL_CROSSING_EPSILON / 2.0,
        v.plane_point.y,
        v.plane_point.z,
    );
    assert_eq!(
        detect_portal_crossing(edge + v.normal, edge - v.normal, &v, realm),
        None
    );
    // Parallel to the plane, in front of the opening.
    let a = at(0.5);
    let b = a + Vec3::new(2.0, 0.5, 0.0);
    assert_eq!(detect_portal_crossing(a, b, &v, realm), None);
    // No movement.
    assert_eq!(detect_portal_crossing(a, a, &v, realm), None);
    // Touching the plane without crossing it.
    assert_eq!(detect_portal_crossing(at(1.0), at(0.0), &v, realm), None);
    assert_eq!(detect_portal_crossing(at(0.0), at(1.0), &v, realm), None);
    // Approaching and backing off.
    assert_eq!(detect_portal_crossing(at(3.0), at(0.2), &v, realm), None);
    assert_eq!(detect_portal_crossing(at(0.2), at(3.0), &v, realm), None);
    // A height change below the waist, away from the portal, is not a crossing.
    let low_a = Vec3::new(2.0, 0.0, 20.0);
    let low_b = Vec3::new(2.0, -30.0, 20.0);
    assert_eq!(detect_portal_crossing(low_a, low_b, &v, realm), None);
}

#[test]
fn a_fast_segment_is_still_detected_and_the_hit_point_is_exact() {
    let v = volume();
    let from = at(40.0) + Vec3::new(0.3, 0.2, 0.0);
    let to = at(-40.0) + Vec3::new(-0.3, -0.2, 0.0);
    assert_eq!(
        detect_portal_crossing(from, to, &v, WorldRealm::Overworld),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    let hit = v.segment_intersection(from, to).unwrap();
    assert!(v.signed_distance(hit).abs() < 1e-4);
    assert!((hit - v.plane_point).length() < 0.05);
    // Non-finite input never panics or triggers.
    assert_eq!(
        detect_portal_crossing(Vec3::new(f32::NAN, 0.0, 0.0), to, &v, WorldRealm::Overworld),
        None
    );
}

#[test]
fn the_detector_fires_once_and_rearms_only_after_leaving_the_membrane() {
    let mut d = PortalCrossingDetector::new(volume());
    assert!(d.is_armed());
    assert_eq!(
        d.detect(at(2.0), at(-0.3), WorldRealm::Overworld),
        Some(PortalCrossingEvent::OverworldToRedBlack)
    );
    assert!(!d.is_armed());
    // Hovering back and forth right at the plane: nothing.
    for i in 0..10 {
        let a = at(if i % 2 == 0 { -0.3 } else { 0.3 });
        let b = at(if i % 2 == 0 { 0.3 } else { -0.3 });
        assert_eq!(d.detect(a, b, WorldRealm::RedBlack), None);
        assert!(!d.is_armed());
    }
    // Leaving beyond the rearm distance arms it again; the next real
    // crossing (now from the Red-Black realm) reports the return trip.
    assert_eq!(
        d.detect(
            at(-0.3),
            at(-PORTAL_REARM_DISTANCE - 0.1),
            WorldRealm::RedBlack
        ),
        None
    );
    assert!(d.is_armed());
    assert_eq!(
        d.detect(at(-2.0), at(0.5), WorldRealm::RedBlack),
        Some(PortalCrossingEvent::RedBlackToOverworld)
    );
    assert!(!d.is_armed());
    // Reset rearms immediately.
    d.reset();
    assert!(d.is_armed());
}

#[test]
fn the_world_mode_owns_a_detector_built_from_the_scene_and_reports_only() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let world =
        &app[app.find("struct WorldMode").unwrap()..app.find("/// Casts one primary ray").unwrap()];
    assert!(world.contains("PortalVolume::from_anchor(&scene.rhombus().portal)"));
    assert!(
        world.contains(
            "portal_detector.detect(previous, self.free_fly.position, self.free_fly.realm)"
        ) || world.contains(".detect(previous, self.free_fly.position, self.free_fly.realm)")
    );
    // No realm switch yet, no height rule anywhere.
    assert!(!world.contains("realm = WorldRealm::RedBlack"));
    assert!(!world.contains("waist_y") && !world.contains("position.y <"));
    let catalog =
        &app[app.find("struct CatalogMode").unwrap()..app.find("struct WorldMode").unwrap()];
    assert!(!catalog.contains("portal"));
}

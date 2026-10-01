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

use camera::world_collision::{
    CAMERA_COLLISION_RADIUS, CameraCollisionConfig, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::overworld_blocks::wood_stairs_material_id;
use scene::world::WorldScene;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn center(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 + 0.5, c.z as f32 + 0.5)
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(
        scene.world(),
        from,
        delta,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

/// Where the inverted viewer's eye sits under tread `k`: 1.5 cells toward
/// -Y from the tread's cell (its floor is the tread's -Y faces).
fn eye_under(scene: &WorldScene, k: i32) -> Vec3 {
    let t = scene.inverted_route().layout.tread(k);
    v(t.x as f32 + 0.5, t.y as f32 - 1.5, t.z as f32 + 0.5)
}

#[test]
fn the_first_stair_stands_right_behind_the_exit_clearance() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    let first = route.first().expect("the flight was built");
    assert_eq!(first, route.layout.first_step);
    assert_eq!(first, IVec3::new(16, -6, 12));
    // One cell north of the clearance, in the same column, at its top row.
    assert_eq!(first.z, route.layout.portal_exit.z - 1);
    assert_eq!(first.x, route.layout.x);
    assert_eq!(first.y, route.layout.exit_max_y);
    assert!(
        route
            .exit_clearance
            .contains(&IVec3::new(first.x, first.y, first.z + 1))
    );
    // The flight inside the mass plus the rim step that meets the landing.
    assert_eq!(route.treads.len(), route.layout.step_count as usize + 1);
    assert_eq!(route.treads.len(), 10);
}

#[test]
fn nothing_solid_separates_the_core_from_the_first_step() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    // The air under the first tread touches the exit clearance.
    for cell in route.layout.clearance_under(0) {
        assert!(!scene.world().contains(cell), "{cell:?} is solid");
        assert!(
            route
                .exit_clearance
                .contains(&IVec3::new(cell.x, cell.y, cell.z + 1))
        );
    }
    // The camera flies from the membrane straight under the first tread.
    let membrane = v(16.5, -7.5, 14.5);
    let under_first = eye_under(&scene, 0);
    let to = resolve(&scene, membrane, under_first - membrane);
    assert!((to - under_first).length() < 1e-3, "blocked at {to:?}");
}

#[test]
fn every_tread_is_a_down_wooden_stair() {
    let scene = WorldScene::new();
    for (cell, orientation) in &scene.inverted_route().treads {
        assert_eq!(*orientation, Orientation::Down);
        let block = scene.world().get(*cell).unwrap();
        assert_eq!(block.block_type(), BlockType::WoodStairs);
        assert_eq!(block.material_id(), wood_stairs_material_id());
        assert_eq!(block.orientation(), Orientation::Down);
        // Hanging from rock: the cell above (world +Y) is solid.
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "{cell:?} floats"
        );
    }
}

#[test]
fn the_functional_top_faces_the_inverted_up() {
    // A Down stair: the full slab at the top of the cell and the raised
    // half at the bottom-north, so its walkable faces point -Y and the
    // step rises toward -Z (north), the direction the flight runs.
    let parts = block_geometry(BlockType::WoodStairs, Orientation::Down).parts();
    assert_eq!(parts.len(), 2);
    let slab = parts
        .iter()
        .find(|p| (p.max().y - 1.0).abs() < 1e-6 && (p.min().y - 0.5).abs() < 1e-6)
        .expect("top slab");
    assert!((slab.min().z).abs() < 1e-6 && (slab.max().z - 1.0).abs() < 1e-6);
    let raised = parts
        .iter()
        .find(|p| p.min().y.abs() < 1e-6)
        .expect("low half");
    assert!((raised.max().y - 0.5).abs() < 1e-6);
    assert!(
        raised.max().z <= 0.5 + 1e-6,
        "the low half must be north: {raised:?}"
    );
    let scene = WorldScene::new();
    assert_eq!(
        scene.inverted_route().layout.direction,
        IVec3::new(0, 0, -1)
    );
    assert_eq!(
        camera::world_free_fly::WorldRealm::RedBlack.up(),
        v(0.0, -1.0, 0.0)
    );
}

#[test]
fn the_flight_is_contiguous() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    let treads: Vec<IVec3> = route.treads.iter().map(|(c, _)| *c).collect();
    for pair in treads.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert_eq!((b.x, b.y, b.z), (a.x, a.y - 1, a.z - 1), "{a:?} -> {b:?}");
    }
    // The air under consecutive treads shares rows, so the tunnel is one
    // continuous cavity from the clearance to the landing.
    for k in 1..route.layout.step_count {
        let prev = route.layout.clearance_under(k - 1);
        let next = route.layout.clearance_under(k);
        assert!(
            prev.iter()
                .any(|p| next.iter().any(|n| n.y == p.y && (n.z - p.z).abs() == 1))
        );
        for cell in next {
            assert!(!scene.world().contains(cell), "{cell:?} blocks the tunnel");
        }
    }
}

#[test]
fn the_collision_volume_fits_through_the_whole_flight() {
    let scene = WorldScene::new();
    let steps = scene.inverted_route().layout.step_count;
    let mut position = v(16.5, -7.5, 14.5);
    for k in 0..steps {
        let target = eye_under(&scene, k);
        assert!(
            clear(&scene, target),
            "eye under tread {k} is blocked: {target:?}"
        );
        let reached = resolve(&scene, position, target - position);
        assert!(
            (reached - target).length() < 1e-3,
            "step {k}: stuck at {reached:?} short of {target:?}"
        );
        position = reached;
    }
    assert!(CAMERA_COLLISION_RADIUS <= 0.3);
}

#[test]
fn the_portal_frame_is_intact() {
    let scene = WorldScene::new();
    assert_eq!(scene.portal().frame.len(), 18);
    for cell in &scene.portal().frame {
        assert_eq!(
            scene.world().get(*cell).map(|b| b.block_type()),
            Some(BlockType::PortalFrameRedObsidian)
        );
    }
}

#[test]
fn the_portal_core_is_intact() {
    let scene = WorldScene::new();
    assert_eq!(scene.portal().core.len(), 12);
    for cell in &scene.portal().core {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::PortalCoreDarkCrimson);
        assert_eq!(b.orientation(), Orientation::South);
    }
}

#[test]
fn no_tread_or_tunnel_cell_touches_the_core() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    for cell in route.dug.iter().chain(route.treads.iter().map(|(c, _)| c)) {
        assert!(!scene.portal().core.contains(cell) && !scene.portal().frame.contains(cell));
        assert!(
            cell.z < scene.rhombus().portal.wall_z,
            "{cell:?} is not behind the wall"
        );
    }
}

#[test]
fn the_flight_can_be_walked_back_to_the_portal() {
    let scene = WorldScene::new();
    let steps = scene.inverted_route().layout.step_count;
    let mut position = eye_under(&scene, steps - 1);
    for k in (0..steps - 1).rev() {
        let target = eye_under(&scene, k);
        let reached = resolve(&scene, position, target - position);
        assert!(
            (reached - target).length() < 1e-3,
            "back at step {k}: {reached:?}"
        );
        position = reached;
    }
    // Into the clearance and through the membrane to the approach.
    let membrane = v(16.5, -7.5, 14.5);
    let reached = resolve(&scene, position, membrane - position);
    assert!((reached - membrane).length() < 1e-3, "{reached:?}");
    let approach = v(16.5, -7.5, 16.5);
    let reached = resolve(&scene, reached, approach - reached);
    assert!((reached - approach).length() < 1e-3, "{reached:?}");
}

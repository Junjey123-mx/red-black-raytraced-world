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

use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, nearest_visible_hit};
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::red_black_timber::{
    red_black_wood_door_bottom_material_id, red_black_wood_door_top_material_id,
};
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_lights,
    world_materials,
};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn cfg() -> CameraCollisionConfig {
    CameraCollisionConfig::default()
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(scene.world(), p, &cfg(), &is_camera_solid)
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(scene.world(), from, delta, &cfg(), &is_camera_solid)
}

/// The inverted viewer's eye under a ground cell.
fn eye(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 - 1.5, c.z as f32 + 0.5)
}

fn step(scene: &WorldScene, from: Vec3, target: Vec3) -> Vec3 {
    let direct = resolve(scene, from, target - from);
    if (direct - target).length() < 1e-3 {
        return direct;
    }
    let (f, t) = (from, target);
    for corner in [
        v(t.x, f.y, t.z),
        v(f.x, t.y, f.z),
        v(t.x, f.y, f.z),
        v(f.x, f.y, t.z),
        v(t.x, t.y, f.z),
        v(f.x, t.y, t.z),
    ] {
        let mid = resolve(scene, from, corner - from);
        if (mid - corner).length() < 1e-3 {
            let end = resolve(scene, mid, target - mid);
            if (end - target).length() < 1e-3 {
                return end;
            }
        }
    }
    direct
}

fn walk(scene: &WorldScene, points: &[Vec3]) {
    let mut position = points[0];
    for (i, target) in points.iter().enumerate().skip(1) {
        let reached = step(scene, position, *target);
        assert!(
            (reached - *target).length() < 1e-3,
            "segment {i}: stuck at {reached:?} short of {target:?}"
        );
        position = reached;
    }
}

fn walk_both_ways(scene: &WorldScene, points: &[Vec3]) {
    walk(scene, points);
    let mut back = points.to_vec();
    back.reverse();
    walk(scene, &back);
}

fn entrance(scene: &WorldScene) -> Vec<Vec3> {
    let l = scene.fortress_layout();
    let mut points: Vec<Vec3> = scene
        .red_black_approach()
        .main
        .iter()
        .map(|c| eye(*c))
        .collect();
    points.extend(
        l.main_route()
            .iter()
            .take_while(|r| r.x < l.central_keep.min_x)
            .map(|r| eye(*r)),
    );
    points
}

// 1
#[test]
fn the_door_is_visible_from_the_approach() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = &scene.red_black_identity().gatehouse;
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    let lights = world_lights();
    // From every path cell facing the gate, a ray meets the door.
    let stretch: Vec<IVec3> = scene
        .red_black_approach()
        .main
        .iter()
        .copied()
        .filter(|c| {
            // The gate-facing stretch: its outer column, and the cells right
            // in front of the gate (a wall-hugging cell sees only the jamb).
            c.z >= l.gate.base.z - 3
                && (c.x == l.gate.base.x - 2
                    || (c.x == l.gate.base.x - 1 && c.z >= l.gate.base.z - 1))
        })
        .collect();
    assert!(stretch.len() >= 4, "{}", stretch.len());
    for c in &stretch {
        let from = eye(*c);
        let view = VoxelScene {
            world: scene.world(),
            materials: &materials,
            camera_position: from,
            lights: &lights,
            ambient_factor: WORLD_AMBIENT_FACTOR,
            background: scene::world::world_background(),
            texture_manager: &manager,
            max_distance: WORLD_MAX_DISTANCE,
        };
        let sees = g.door.iter().any(|d| {
            let target = v(d.x as f32 + 0.5, d.y as f32 + 0.5, d.z as f32 + 0.5);
            let ray = Ray::new(from, (target - from).normalize());
            nearest_visible_hit(&view, &ray, WORLD_MAX_DISTANCE)
                .is_some_and(|hit| hit.block.block_type() == BlockType::RedBlackWoodDoor)
        });
        assert!(sees, "the door is hidden from {c:?}");
    }
    println!(
        "GATE17_5 gatehouse: door={} corridor={} gallery={} rails={} seen_from={} identity_voxels={} voxels={}",
        g.door.len(),
        g.corridor_floor.len(),
        g.hood.len(),
        g.rails.len(),
        stretch.len(),
        scene.identity_voxels(),
        scene.world().len()
    );
}

// 2
#[test]
fn the_door_is_passable() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = &scene.red_black_identity().gatehouse;
    assert_eq!(g.door.len(), 4);
    let mut cells = g.door.clone();
    cells.sort_by_key(|c| (c.z, -c.y));
    let mut gate = l.gate.cells();
    gate.sort_by_key(|c| (c.z, -c.y));
    assert_eq!(cells, gate, "the door fills the gate opening");
    for c in &g.door {
        let b = scene.world().get(*c).unwrap();
        assert_eq!(b.block_type(), BlockType::RedBlackWoodDoor);
        assert_eq!(b.orientation(), Orientation::West);
        assert!(is_camera_passable(b.block_type()) && !is_camera_solid(b));
        // Lower half next to the ground, upper half one course toward -Y.
        let expected = if c.y == l.levels.base_y {
            red_black_wood_door_bottom_material_id()
        } else {
            red_black_wood_door_top_material_id()
        };
        assert_eq!(b.material_id(), expected, "{c:?}");
    }
    let d = l.gate.base;
    assert!(clear(
        &scene,
        v(d.x as f32 + 0.5, d.y as f32 - 0.5, d.z as f32 + 1.0)
    ));
}

// 3
#[test]
fn the_surrounding_walls_are_solid() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = l.gate;
    for cell in [
        IVec3::new(g.base.x, g.base.y, g.base.z - 1),
        IVec3::new(g.base.x, g.base.y - 1, g.base.z - 1),
        IVec3::new(g.base.x, g.base.y, g.base.z + g.width),
        IVec3::new(g.base.x, g.base.y - 2, g.base.z),
        IVec3::new(g.base.x, g.base.y - 2, g.base.z + 1),
    ] {
        let b = scene
            .world()
            .get(cell)
            .unwrap_or_else(|| panic!("{cell:?} missing"));
        assert!(is_camera_solid(b), "{cell:?}");
    }
    // Straight at a jamb from the forecourt side: stopped before the wall.
    let jamb = IVec3::new(g.base.x, g.base.y, g.base.z - 1);
    let from = v(
        jamb.x as f32 - 1.5,
        jamb.y as f32 + 0.5,
        jamb.z as f32 + 0.5,
    );
    let to = v(
        jamb.x as f32 + 1.5,
        jamb.y as f32 + 0.5,
        jamb.z as f32 + 0.5,
    );
    let reached = resolve(&scene, from, to - from);
    assert!(reached.x < jamb.x as f32, "{reached:?}");
    let gh = &scene.red_black_identity().gatehouse;
    for c in gh.hood.iter().chain(gh.rails.iter()) {
        assert!(is_camera_solid(scene.world().get(*c).unwrap()));
    }
}

// 4
#[test]
fn the_path_aligns_with_the_door() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let p = &scene.red_black_identity().path;
    for (x, z) in l.gate.columns() {
        let front = p
            .planks
            .iter()
            .chain(p.accents.iter())
            .any(|c| c.x == x - 1 && c.z == z);
        assert!(front, "nothing paved in front of door column ({x},{z})");
    }
    let g = &scene.red_black_identity().gatehouse;
    for c in &g.corridor_floor {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
    }
    assert_eq!(g.corridor_floor.len(), l.gate_corridor().len());
    assert!(!g.hood.is_empty() && !g.rails.is_empty());
    for c in &g.hood {
        assert!(l.gatehouse.contains(c.x, c.z));
        assert_eq!(
            c.y,
            l.levels.wall_top_y - 2,
            "the gallery sits above the vault"
        );
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackWoodPlanks
        );
        assert!(
            scene.world().contains(IVec3::new(c.x, c.y + 1, c.z)),
            "{c:?} rests on the vault"
        );
    }
    for c in &g.rails {
        assert_eq!(
            scene.world().get(*c).unwrap().block_type(),
            BlockType::RedBlackFence
        );
    }
}

// 5
#[test]
fn the_corridor_is_clear() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = l.levels.ground_y;
    for (x, z) in l.gate_corridor() {
        for dy in 1..=2 {
            let cell = IVec3::new(x, g - dy, z);
            let open = scene.world().get(cell).is_none_or(|b| !is_camera_solid(b));
            assert!(open, "{cell:?} blocks the corridor");
        }
        assert!(clear(&scene, eye(IVec3::new(x, g, z))), "({x},{z})");
    }
    // The hood keeps the forecourt's head room.
    for c in &scene.red_black_identity().gatehouse.hood {
        let ground = scene
            .red_black_approach()
            .main
            .iter()
            .find(|p| p.x == c.x && p.z == c.z);
        if let Some(gc) = ground {
            assert!(c.y < gc.y - 2, "{c:?} sits in the head room of {gc:?}");
        }
    }
}

// 6
#[test]
fn the_courtyard_is_reachable() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    walk(&scene, &entrance(&scene));
    let l = scene.fortress_layout();
    let last = *entrance(&scene).last().unwrap();
    assert!(
        l.courtyard
            .contains(&(last.x.floor() as i32, last.z.floor() as i32))
    );
}

// 7
#[test]
fn the_entrance_reverses() {
    let scene = WorldScene::new();
    let mut points = entrance(&scene);
    points.reverse();
    walk(&scene, &points);
}

// 8
#[test]
fn no_portal_semantics_are_reused() {
    let scene = WorldScene::new();
    for c in &scene.red_black_identity().gatehouse.door {
        let t = scene.world().get(*c).unwrap().block_type();
        assert!(!matches!(
            t,
            BlockType::PortalCoreDarkCrimson | BlockType::PortalFrameRedObsidian
        ));
    }
    let src = std::fs::read_to_string("src/scene/red_black_identity.rs").unwrap();
    for token in [
        "PortalCore",
        "PortalFrame",
        "portal_crossing",
        "PortalVolume",
        "PortalCrossingDetector",
    ] {
        assert!(!src.contains(token), "{token}");
    }
    let portal = scene.portal();
    assert_eq!(portal.core.len(), 12);
    assert_eq!(portal.frame.len(), 18);
}

// 9
#[test]
fn no_noclip_is_needed() {
    let scene = WorldScene::new();
    let mut points = vec![v(16.5, -7.5, 13.5)];
    for (t, _) in &scene.inverted_route().treads {
        points.push(eye(*t));
    }
    let r = scene.inverted_route().layout;
    points.push(eye(IVec3::new(r.x, r.platform_y, r.platform_z_max)));
    points.extend(entrance(&scene));
    points.extend(
        scene
            .fortress_layout()
            .main_route()
            .iter()
            .skip_while(|c| c.x < scene.fortress_layout().central_keep.min_x)
            .map(|c| eye(*c)),
    );
    walk_both_ways(&scene, &points);
}

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
    CameraCollisionConfig, CollisionState, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::{BlockType, OFFICIAL_BLOCK_COUNT};
use scene::fortress::{is_dark_structure, is_symbol};
use scene::orientation::Orientation;
use scene::world::WorldScene;

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

fn main_route(scene: &WorldScene) -> Vec<Vec3> {
    scene
        .fortress_layout()
        .main_route()
        .iter()
        .map(|c| eye(*c))
        .collect()
}

#[test]
fn the_hall_is_reachable_from_the_gate() {
    let scene = WorldScene::new();
    assert!(CollisionState::new().collision_enabled());
    let route = main_route(&scene);
    walk(&scene, &route);
    let l = scene.fortress_layout();
    // The route passes the keep door, crosses the hall's door row, climbs
    // the stair and ends in the upper room.
    let cols: Vec<(i32, i32)> = l.main_route().iter().map(|r| (r.x, r.z)).collect();
    assert!(cols.contains(&(l.keep_door.base.x, l.keep_door.base.z)));
    for (tread, _) in &scene.fortress_hall().treads {
        assert!(l.main_route().contains(tread));
    }
    assert_eq!(l.main_route().last().unwrap().y, l.levels.upper_floor_y);
    let h = scene.fortress_hall();
    println!(
        "GATE17 hall: treads={} dais={} seats={} table={} crest={} wood_stairs={} fortress_voxels={} voxels={}",
        h.treads.len(),
        h.dais.len(),
        h.seats.len(),
        h.table.len(),
        h.crest.len(),
        h.wood_stairs(),
        scene.fortress_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_entrance_and_exit_stay_clear() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let d = l.keep_door.base;
    for dy in 0..2 {
        for dx in [-1, 0, 1] {
            let cell = IVec3::new(d.x + dx, d.y - dy, d.z);
            assert!(
                !scene.world().contains(cell),
                "{cell:?} blocks the keep door"
            );
        }
    }
    // The door row inside the hall is open wall to wall, two cells high.
    let g = l.levels.ground_y;
    for x in l.hall.min_x..=l.hall.max_x {
        for dy in 1..=2 {
            assert!(
                !scene.world().contains(IVec3::new(x, g - dy, l.hall.max_z)),
                "({x},{}) blocked",
                l.hall.max_z
            );
        }
    }
    let mut route = main_route(&scene);
    route.reverse();
    walk(&scene, &route);
}

#[test]
fn seating_and_platform_reuse_existing_geometry() {
    let scene = WorldScene::new();
    let h = scene.fortress_hall();
    let l = scene.fortress_layout();
    assert!(
        h.wood_stairs() <= 6,
        "too many wood stairs: {}",
        h.wood_stairs()
    );
    for cell in h.seats.iter().chain(std::iter::once(&h.throne)) {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodStairs);
        assert_eq!(b.orientation(), Orientation::Down, "{cell:?}");
        assert!(l.hall.contains(cell.x, cell.z));
    }
    for (cell, o) in &h.treads {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodStairs);
        assert_eq!(b.orientation(), *o);
        assert_eq!(*o, Orientation::Down);
    }
    // The throne sits on the dais, its backrest (raised half, north) at the wall.
    assert!(
        h.dais
            .contains(&IVec3::new(h.throne.x, h.throne.y + 1, h.throne.z))
    );
    assert_eq!(h.throne.z, l.hall.min_z);
    assert!(
        scene
            .world()
            .contains(IVec3::new(h.throne.x, h.throne.y, h.throne.z - 1)),
        "nothing behind the throne"
    );
    for cell in &h.dais {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::PolishedBlackstoneBricks
        );
        assert_eq!(cell.y, l.levels.base_y);
    }
    for cell in &h.table {
        assert!(is_dark_structure(
            scene.world().get(*cell).unwrap().block_type()
        ));
    }
    // Seats face (open side, south) the hall; the table stands beside them.
    for seat in &h.seats {
        assert!(
            h.table
                .iter()
                .any(|t| (t.x - seat.x).abs() + (t.z - seat.z).abs() <= 2)
        );
    }
}

#[test]
fn the_crest_wall_shows_all_three_families() {
    let scene = WorldScene::new();
    let h = scene.fortress_hall();
    let l = scene.fortress_layout();
    assert_eq!(h.crest.len(), 3);
    let kinds: Vec<BlockType> = h.crest.iter().map(|(_, t)| *t).collect();
    assert!(
        kinds
            .iter()
            .any(|t| matches!(t, BlockType::CrimsonHeart | BlockType::CrimsonDiamond))
    );
    assert!(
        kinds
            .iter()
            .any(|t| matches!(t, BlockType::OrangeClub | BlockType::OrangeSpade))
    );
    assert!(kinds.iter().any(|t| matches!(
        t,
        BlockType::PurpleHeart
            | BlockType::PurpleDiamond
            | BlockType::PurpleClub
            | BlockType::PurpleSpade
    )));
    for (cell, t) in &h.crest {
        assert!(is_symbol(*t));
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *t);
        assert_eq!(cell.z, l.central_keep.min_z, "set into the north wall");
        assert_eq!(cell.y, l.levels.base_y - 1, "at eye level");
        assert_eq!(
            scene.world().get(*cell).unwrap().orientation(),
            Orientation::Down
        );
    }
    // Visible from the hall: the cells in front of the crest are open.
    for (cell, _) in &h.crest {
        let front = IVec3::new(cell.x, cell.y, cell.z + 1);
        let open = !scene.world().contains(front) || h.throne == front;
        assert!(open, "{front:?} hides the crest");
    }
}

#[test]
fn no_new_block_type_was_added() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    let src = std::fs::read_to_string("src/scene/block_type.rs").unwrap();
    for token in ["Throne", "Chair", "Seat", "Table", "Dais"] {
        assert!(!src.contains(token), "{token} in block_type.rs");
    }
    let scene = WorldScene::new();
    for cell in scene.fortress_hall().cells() {
        let t = scene.world().get(cell).unwrap().block_type();
        assert!(
            matches!(
                t,
                BlockType::WoodStairs
                    | BlockType::PolishedBlackstoneBricks
                    | BlockType::SmoothBasalt
            ) || is_symbol(t),
            "{t:?}"
        );
    }
}

#[test]
fn the_collision_path_through_the_hall_is_clear() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let h = scene.fortress_hall();
    let g = l.levels.ground_y;
    // Standing cells: every hall floor cell without a fitting above it has
    // head room and a clear eye; the cell before the dais is one of them.
    let fittings: Vec<(i32, i32)> = h.cells().iter().map(|c| (c.x, c.z)).collect();
    let mut standing = 0;
    for z in l.hall.min_z..=l.hall.max_z {
        for x in l.hall.min_x..=l.hall.max_x {
            if fittings.contains(&(x, z)) {
                continue;
            }
            standing += 1;
            for dy in 1..=2 {
                assert!(
                    !scene.world().contains(IVec3::new(x, g - dy, z)),
                    "({x},{z})"
                );
            }
            assert!(clear(&scene, eye(IVec3::new(x, g, z))), "({x},{z})");
        }
    }
    assert!(standing >= 6, "{standing} standing cells");
    let before_dais = IVec3::new(h.throne.x, g, h.throne.z + 1);
    assert!(clear(&scene, eye(before_dais)));
    // From the door row to the throne's foot and back.
    walk_both_ways(
        &scene,
        &[
            eye(IVec3::new(h.throne.x, g, l.hall.max_z)),
            eye(IVec3::new(h.throne.x, g, l.hall.max_z - 1)),
            eye(before_dais),
        ],
    );
    // Head room over the treads toward the upper room.
    for (tread, _) in &h.treads {
        for dy in 1..=2 {
            assert!(!scene.world().contains(above(*tread, dy)), "{tread:?}");
        }
        assert!(clear(&scene, eye(*tread)));
    }
}

fn above(c: IVec3, dy: i32) -> IVec3 {
    IVec3::new(c.x, c.y - dy, c.z)
}

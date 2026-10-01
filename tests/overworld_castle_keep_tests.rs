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
    CameraCollisionConfig, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::castle::KEEP_HEIGHT;
use scene::orientation::Orientation;
use scene::world::WorldScene;
use std::collections::{HashSet, VecDeque};

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

fn eye(c: IVec3) -> Vec3 {
    v(c.x as f32 + 0.5, c.y as f32 + 2.0, c.z as f32 + 0.5)
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

fn reaches(scene: &WorldScene, points: &[Vec3]) -> bool {
    let mut position = points[0];
    for target in points.iter().skip(1) {
        position = step(scene, position, *target);
        if (position - *target).length() >= 1e-3 {
            return false;
        }
    }
    true
}

fn walk_both_ways(scene: &WorldScene, points: &[Vec3]) {
    walk(scene, points);
    let mut back = points.to_vec();
    back.reverse();
    walk(scene, &back);
}

fn stone_family(t: BlockType) -> bool {
    // A later stage may set a window bay (Glass) into a shell cell.
    matches!(
        t,
        BlockType::Stone
            | BlockType::Cobblestone
            | BlockType::DeepslateBricks
            | BlockType::Glass
            | BlockType::RedstoneLampLit
    )
}

#[test]
fn the_keep_shell_is_connected_and_anchored() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let k = scene.castle_keep();
    assert!(!k.shell.is_empty());
    for cell in k.shell.iter().chain(k.corners.iter()) {
        assert!(c.is_keep_shell(cell.x, cell.z), "{cell:?}");
        assert!(cell.y >= c.levels.base_y && cell.y <= c.levels.keep_roof_y);
        let b = scene.world().get(*cell).unwrap();
        assert!(is_camera_solid(b));
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
                || k.door.contains(&IVec3::new(cell.x, cell.y - 1, cell.z)),
            "{cell:?} floats"
        );
    }
    for cell in &k.shell {
        assert!(stone_family(scene.world().get(*cell).unwrap().block_type()));
    }
    // One connected shell: flood over shell + corner cells by face adjacency
    // (the two east corners are tower cells and complete the outline).
    let all: HashSet<IVec3> = k
        .shell
        .iter()
        .chain(k.corners.iter())
        .chain(scene.castle_towers().shells.iter())
        .filter(|cell| c.keep.contains(cell.x, cell.z))
        .copied()
        .collect();
    let start = *k.shell.first().unwrap();
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(cur) = queue.pop_front() {
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(cur.x + dx, cur.y + dy, cur.z + dz);
            if all.contains(&n) && seen.insert(n) {
                queue.push_back(n);
            }
        }
    }
    assert_eq!(seen.len(), all.len(), "the shell is in pieces");
    // Every shell column rises from the ground to the roof (door excepted).
    for z in c.keep.min_z..=c.keep.max_z {
        for x in c.keep.min_x..=c.keep.max_x {
            if !c.is_keep_shell(x, z) {
                continue;
            }
            for y in c.levels.base_y..=c.levels.keep_roof_y {
                assert!(
                    scene.world().contains(IVec3::new(x, y, z)),
                    "({x},{y},{z}) missing"
                );
            }
        }
    }
    assert_eq!(c.levels.keep_roof_y - c.levels.ground_y, KEEP_HEIGHT);
    println!(
        "GATE16 keep: shell={} corners={} door={} ground={} upper={} beams={} roof={} castle_voxels={} voxels={}",
        k.shell.len(),
        k.corners.len(),
        k.door.len(),
        k.ground_floor.len(),
        k.upper_floor.len(),
        k.beams.len(),
        k.roof.len(),
        scene.castle_voxels(),
        scene.world().len()
    );
}

#[test]
fn the_interior_is_air_on_both_levels() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let h = c.hall;
    let lv = c.levels;
    // Rooms, not fill: whatever stands inside is a fitting of a later
    // stage (stairs, rails, seats, tables, lamps), never masonry or logs,
    // and most of the volume stays air.
    let mut volume = 0;
    let mut occupied = 0;
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            for y in (lv.base_y..lv.upper_floor_y).chain((lv.upper_floor_y + 1)..lv.keep_roof_y) {
                let cell = IVec3::new(x, y, z);
                volume += 1;
                if let Some(b) = scene.world().get(cell) {
                    occupied += 1;
                    assert!(
                        matches!(
                            b.block_type(),
                            BlockType::WoodStairs
                                | BlockType::Fence
                                | BlockType::DoubleWoodSlab
                                | BlockType::WoodPlanks
                                | BlockType::RedstoneLampLit
                        ),
                        "{cell:?} fills the keep with {:?}",
                        b.block_type()
                    );
                }
            }
        }
    }
    assert_eq!(volume as i32, h.width() * h.depth() * 4);
    assert!(
        occupied * 3 < volume,
        "{occupied} of {volume} cells occupied"
    );
    // The well is open from the ground floor up through the upper floor.
    for (x, z) in &c.stair_well {
        assert!(!scene.world().contains(IVec3::new(*x, lv.upper_floor_y, *z)));
    }
    // Roof above, floor below: both closed.
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            assert_eq!(
                scene
                    .world()
                    .get(IVec3::new(x, lv.keep_roof_y, z))
                    .unwrap()
                    .block_type(),
                BlockType::DoubleWoodSlab
            );
            assert_eq!(
                scene
                    .world()
                    .get(IVec3::new(x, lv.ground_y, z))
                    .unwrap()
                    .block_type(),
                BlockType::WoodPlanks
            );
        }
    }
}

#[test]
fn the_free_corners_are_logs() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let k = scene.castle_keep();
    let free: Vec<(i32, i32)> = [
        (c.keep.min_x, c.keep.min_z),
        (c.keep.min_x, c.keep.max_z),
        (c.keep.max_x, c.keep.min_z),
        (c.keep.max_x, c.keep.max_z),
    ]
    .into_iter()
    .filter(|(x, z)| c.tower_at(*x, *z).is_none())
    .collect();
    assert!(free.len() >= 2, "{free:?}");
    assert_eq!(k.corners.len(), free.len() * KEEP_HEIGHT as usize);
    for cell in &k.corners {
        assert!(free.contains(&(cell.x, cell.z)));
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::Log
        );
    }
}

#[test]
fn the_floors_are_wood_and_the_door_passable() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let k = scene.castle_keep();
    assert_eq!(
        k.ground_floor.len(),
        (c.hall.width() * c.hall.depth()) as usize
    );
    assert_eq!(
        k.upper_floor.len() + k.beams.len() + c.stair_well.len(),
        k.ground_floor.len()
    );
    for cell in &k.upper_floor {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::WoodPlanks
        );
        assert_eq!(cell.y, c.levels.upper_floor_y);
    }
    for cell in &k.beams {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::Log
        );
        assert!(cell.z == c.hall.min_z || cell.z == c.hall.max_z);
    }
    assert_eq!(k.door.len(), 2);
    for cell in &k.door {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::WoodDoor);
        assert_eq!(b.orientation(), Orientation::West);
        assert!(is_camera_passable(b.block_type()));
    }
    let d = c.keep_door.base;
    // From the courtyard through the door into the hall.
    let points = [
        v(d.x as f32 - 0.5, d.y as f32 + 1.0, d.z as f32 + 0.5),
        v(d.x as f32 + 0.5, d.y as f32 + 1.0, d.z as f32 + 0.5),
        v(d.x as f32 + 1.5, d.y as f32 + 1.0, d.z as f32 + 0.5),
    ];
    walk_both_ways(&scene, &points);
}

#[test]
fn the_second_level_is_usable() {
    let scene = WorldScene::new();
    let c = scene.castle_layout();
    let k = scene.castle_keep();
    let deck = k.upper_deck();
    assert!(deck.len() >= 10, "{}", deck.len());
    // Deck cells carrying a fitting (the stair rail, a seat) are not stood on.
    let free = |cell: &IVec3| {
        !scene
            .world()
            .contains(IVec3::new(cell.x, cell.y + 1, cell.z))
    };
    let deck: Vec<IVec3> = deck.into_iter().filter(free).collect();
    assert!(deck.len() >= 8, "{}", deck.len());
    for cell in &deck {
        // Two cells of head room under the roof, eye clear.
        for dy in 1..=2 {
            assert!(
                !scene
                    .world()
                    .contains(IVec3::new(cell.x, cell.y + dy, cell.z))
            );
        }
        assert!(clear(&scene, eye(*cell)), "{cell:?}");
    }
    // The whole deck is reachable from its first cell: flood the room at
    // eye height over the hall's columns (the open well included, since the
    // camera flies), stepping only where collision lets the eye through.
    let y = c.levels.upper_floor_y;
    let mut seen = HashSet::from([(deck[0].x, deck[0].z)]);
    let mut queue = VecDeque::from([(deck[0].x, deck[0].z)]);
    while let Some((x, z)) = queue.pop_front() {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if !c.hall.contains(nx, nz) || seen.contains(&(nx, nz)) {
                continue;
            }
            let from = eye(IVec3::new(x, y, z));
            let to = eye(IVec3::new(nx, y, nz));
            if (resolve(&scene, from, to - from) - to).length() < 1e-3 {
                seen.insert((nx, nz));
                queue.push_back((nx, nz));
            }
        }
    }
    for cell in &deck {
        assert!(seen.contains(&(cell.x, cell.z)), "{cell:?} unreachable");
    }
    // The ground floor is walkable from the door row across the hall.
    let g = c.levels.ground_y;
    let row: Vec<Vec3> = (c.hall.min_x..=c.hall.max_x)
        .map(|x| eye(IVec3::new(x, g, c.hall.min_z)))
        .collect();
    walk_both_ways(&scene, &row);
}

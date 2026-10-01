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
    CameraCollisionConfig, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::fortress::{is_dark_structure, is_family_brick, is_symbol};
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

fn family_of(t: BlockType) -> &'static str {
    match t {
        BlockType::CrimsonHeart | BlockType::CrimsonDiamond => "crimson",
        BlockType::OrangeClub | BlockType::OrangeSpade => "orange",
        BlockType::PurpleHeart
        | BlockType::PurpleDiamond
        | BlockType::PurpleClub
        | BlockType::PurpleSpade => "violet",
        _ => "none",
    }
}

#[test]
fn the_keep_is_connected_and_down_oriented() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let k = scene.fortress_keep();
    let outline: HashSet<IVec3> = k
        .shell
        .iter()
        .chain(k.pillars.iter())
        .chain(k.facade.iter().map(|(c, _)| c))
        .chain(k.heraldry.iter().map(|(c, _)| c))
        .chain(scene.fortress_bricks().cells().iter())
        .chain(
            scene
                .fortress_towers()
                .iter()
                .flat_map(|t| t.cells())
                .collect::<Vec<_>>()
                .iter(),
        )
        .filter(|c| l.central_keep.contains(c.x, c.z) && l.is_keep_shell(c.x, c.z))
        .copied()
        .collect();
    let start = *k.shell.first().unwrap();
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(c) = queue.pop_front() {
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let n = IVec3::new(c.x + dx, c.y + dy, c.z + dz);
            if outline.contains(&n) && seen.insert(n) {
                queue.push_back(n);
            }
        }
    }
    assert_eq!(seen.len(), outline.len(), "the shell is in pieces");
    for cell in k.cells() {
        let b = scene.world().get(cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down, "{cell:?}");
        assert!(l.central_keep.contains(cell.x, cell.z));
    }
    // Every shell column runs from the base course to the roof, door excepted.
    for z in l.central_keep.min_z..=l.central_keep.max_z {
        for x in l.central_keep.min_x..=l.central_keep.max_x {
            if !l.is_keep_shell(x, z) {
                continue;
            }
            for y in (l.levels.keep_roof_y..=l.levels.base_y).rev() {
                let cell = IVec3::new(x, y, z);
                assert!(
                    scene.world().contains(cell) || k.door.contains(&cell),
                    "{cell:?} missing"
                );
            }
        }
    }
    assert_eq!(k.pillars.len(), 3 * 6, "{}", k.pillars.len());
    println!(
        "GATE17 keep: shell={} pillars={} floor={} upper={} roof={} facade={} heraldry={} fortress_voxels={} voxels={}",
        k.shell.len(),
        k.pillars.len(),
        k.floor.len(),
        k.upper_floor.len(),
        k.roof.len(),
        k.facade.len(),
        k.heraldry.len(),
        scene.fortress_voxels(),
        scene.world().len()
    );
}

#[test]
fn all_three_families_are_represented_as_focal_accents() {
    let scene = WorldScene::new();
    let k = scene.fortress_keep();
    let symbols = k.symbols();
    assert_eq!(symbols.len(), 6);
    for fam in ["crimson", "orange", "violet"] {
        assert!(
            k.facade.iter().any(|(_, t)| family_of(*t) == fam),
            "{fam} missing from the façade"
        );
        assert!(
            k.heraldry.iter().any(|(_, t)| family_of(*t) == fam),
            "{fam} missing from the heraldry"
        );
    }
    for (cell, t) in &symbols {
        assert!(is_symbol(*t));
        assert_eq!(scene.world().get(*cell).unwrap().block_type(), *t);
        // Set into the dark shell: at most one symbol neighbour along the wall.
        let touching = symbols
            .iter()
            .filter(|(o, _)| {
                o != cell && (o.x - cell.x).abs() + (o.y - cell.y).abs() + (o.z - cell.z).abs() == 1
            })
            .count();
        assert!(touching <= 2, "{cell:?}");
        assert!(
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y + 1, cell.z)),
            "{cell:?} floats"
        );
    }
    let l = scene.fortress_layout();
    for (cell, _) in &k.facade {
        assert_eq!(cell.x, l.central_keep.min_x, "panels face the courtyard");
    }
    for (cell, _) in &k.heraldry {
        assert_eq!(cell.x, l.central_keep.max_x, "heraldry on the east wall");
    }
}

#[test]
fn the_dark_structure_dominates_the_keep() {
    let scene = WorldScene::new();
    let cells = scene.fortress_keep().cells();
    let dark = cells
        .iter()
        .filter(|c| is_dark_structure(scene.world().get(**c).unwrap().block_type()))
        .count();
    let symbols = cells
        .iter()
        .filter(|c| is_symbol(scene.world().get(**c).unwrap().block_type()))
        .count();
    let coloured = cells
        .iter()
        .filter(|c| is_family_brick(scene.world().get(**c).unwrap().block_type()))
        .count();
    assert!(dark * 4 > cells.len() * 3, "dark {dark} of {}", cells.len());
    assert!(symbols <= 8 && coloured <= 8);
    // Shell cells are dark masonry, or a suit the hall (C203) sets into the wall.
    assert!(scene.fortress_keep().shell.iter().all(|c| {
        let t = scene.world().get(*c).unwrap().block_type();
        matches!(
            t,
            BlockType::PolishedBlackstoneBricks | BlockType::SmoothBasalt
        ) || is_symbol(t)
    }));
}

#[test]
fn the_courtyard_sightline_to_the_keep_stays_open() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let g = l.levels.ground_y;
    // From just inside the gate to the keep door, straight across the yard.
    let from = eye(IVec3::new(l.gatehouse.max_x + 1, g, l.keep_door.base.z));
    let to = eye(IVec3::new(l.central_keep.min_x - 1, g, l.keep_door.base.z));
    let reached = resolve(&scene, from, to - from);
    assert!((reached - to).length() < 1e-3, "{reached:?}");
    for (x, z) in &l.courtyard {
        assert!(clear(&scene, eye(IVec3::new(*x, g, *z))), "({x},{z})");
    }
    for cell in scene.fortress_keep().cells() {
        assert!(
            !l.courtyard.contains(&(cell.x, cell.z)),
            "{cell:?} in the courtyard"
        );
    }
}

#[test]
fn the_interior_volume_is_usable_on_two_levels() {
    let scene = WorldScene::new();
    let l = scene.fortress_layout();
    let k = scene.fortress_keep();
    let h = l.hall;
    let lv = l.levels;
    let mut volume = 0;
    let mut occupied = 0;
    for z in h.min_z..=h.max_z {
        for x in h.min_x..=h.max_x {
            for y in
                ((lv.keep_roof_y + 1)..lv.upper_floor_y).chain((lv.upper_floor_y + 1)..=lv.base_y)
            {
                volume += 1;
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    occupied += 1;
                    assert!(
                        !is_dark_structure(b.block_type()) || y == lv.base_y,
                        "({x},{y},{z}) fills the keep with {:?}",
                        b.block_type()
                    );
                }
            }
        }
    }
    assert_eq!(volume as i32, h.width() * h.depth() * 4);
    assert!(occupied * 3 < volume, "{occupied} of {volume}");
    // Hall: every floor cell free of fittings is stood under with head room.
    for cell in &k.floor {
        // Fittings (dais, seats, treads) may take a cell's head room.
        if (1..=2).any(|dy| {
            scene
                .world()
                .contains(IVec3::new(cell.x, cell.y - dy, cell.z))
        }) {
            continue;
        }
        assert!(clear(&scene, eye(*cell)), "{cell:?}");
    }
    // Upper deck: head room toward -Y under the roof.
    assert!(k.upper_deck().len() >= 10);
    for cell in k.upper_deck() {
        if scene
            .world()
            .contains(IVec3::new(cell.x, cell.y - 1, cell.z))
        {
            continue;
        }
        for dy in 1..=2 {
            assert!(
                !scene
                    .world()
                    .contains(IVec3::new(cell.x, cell.y - dy, cell.z))
            );
        }
        assert!(clear(&scene, eye(cell)), "{cell:?}");
    }
    assert_eq!(k.roof.len(), (h.width() * h.depth()) as usize);
    for cell in &k.roof {
        assert_eq!(
            scene.world().get(*cell).unwrap().block_type(),
            BlockType::PolishedBlackstoneBricks
        );
    }
    // The door is open and leads into the hall's south row.
    for cell in &k.door {
        assert!(!scene.world().contains(*cell));
    }
    let d = l.keep_door.base;
    walk_both_ways(
        &scene,
        &[
            v(d.x as f32 - 0.5, d.y as f32 - 0.5, d.z as f32 + 0.5),
            v(d.x as f32 + 0.5, d.y as f32 - 0.5, d.z as f32 + 0.5),
            v(d.x as f32 + 1.5, d.y as f32 - 0.5, d.z as f32 + 0.5),
        ],
    );
}

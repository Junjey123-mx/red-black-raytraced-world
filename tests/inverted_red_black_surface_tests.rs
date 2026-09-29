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

use core::color::Color;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, cast_ray_voxel, nearest_visible_hit};
use renderer::skybox::Background;
use scene::block::BlockInstance;
use scene::block_type::{BlockFamily, BlockType};
use scene::cutaway::carve_cutaway;
use scene::descent::{build_inverted_descent, build_upper_descent};
use scene::orientation::Orientation;
use scene::overworld::{build_house, carve_pond, furnish_house, lay_path, plant_trees};
use scene::overworld_blocks::mycelium_material_id;
use scene::portal::{build_portal_core, build_portal_frame};
use scene::red_black_maze::{
    Family, RedBlackSurface, build_lower_mass, build_red_black_surface, family_zone,
};
use scene::rhombus::{RhombusConfig, build_upper_taper};
use scene::terrain::TerrainConfig;
use scene::terrain::generator::generate_terrain;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;
use scene::world::{WorldScene, WorldTextures, world_materials};
use std::collections::{BTreeSet, HashMap};

/// The surface stage: everything up to `build_red_black_surface`, before
/// the families decorate it.
struct Stage {
    config: TerrainConfig,
    rhombus: RhombusConfig,
    surface: RedBlackSurface,
    world: VoxelWorld,
}

impl Stage {
    fn new() -> Self {
        let config = TerrainConfig::official();
        let rhombus = RhombusConfig::derive(&config);
        let mut world = VoxelWorld::new();
        generate_terrain(&config, &mut world);
        let pond = carve_pond(&config, &mut world);
        build_house(&config, &mut world);
        furnish_house(&config, &mut world);
        let trees = plant_trees(&config, &mut world, &pond);
        lay_path(&config, &mut world, &pond, &trees);
        build_upper_taper(&config, &rhombus, &mut world);
        carve_cutaway(&config, &rhombus, &mut world);
        build_upper_descent(&config, &rhombus, &mut world);
        let mut portal = build_portal_frame(&rhombus, &mut world);
        build_portal_core(&rhombus, &mut world, &mut portal);
        let lower = build_lower_mass(&config, &rhombus, &mut world);
        build_inverted_descent(&rhombus, &lower, &mut world);
        let surface = build_red_black_surface(&config, &rhombus, &lower, &mut world);
        Self {
            config,
            rhombus,
            surface,
            world,
        }
    }

    fn world(&self) -> &VoxelWorld {
        &self.world
    }

    fn rhombus(&self) -> &RhombusConfig {
        &self.rhombus
    }

    fn config(&self) -> &TerrainConfig {
        &self.config
    }

    fn lower_surface(&self) -> &RedBlackSurface {
        &self.surface
    }
}

fn stage_block(stage: &Stage, cell: IVec3) -> Option<BlockType> {
    stage.world().get(cell).map(|b| b.block_type())
}

fn block_type(scene: &WorldScene, cell: IVec3) -> Option<BlockType> {
    scene.world().get(cell).map(|b| b.block_type())
}

#[test]
fn every_surface_cell_is_the_lowest_of_its_column_and_faces_down() {
    let scene = Stage::new();
    let r = scene.rhombus();
    let s = scene.lower_surface();
    assert!(s.cells.len() > 150, "{} surface cells", s.cells.len());
    for cell in &s.cells {
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.orientation(), Orientation::Down);
        for y in r.lower_tip_y..cell.y {
            assert!(
                !scene.world().contains(IVec3::new(cell.x, y, cell.z)),
                "{cell:?} is not the lowest"
            );
        }
        assert!(!r.cutaway.contains_column(cell.x, cell.z));
        assert!(cell.y < r.shelf_y);
    }
    // One surface cell per lower column outside the cut (except the
    // inverted descent's trench columns, which keep their stairs).
    let mut columns: BTreeSet<(i32, i32)> = BTreeSet::new();
    for cell in &s.cells {
        assert!(
            columns.insert((cell.x, cell.z)),
            "two surface cells in column {:?}",
            (cell.x, cell.z)
        );
    }
}

#[test]
fn mycelium_shows_its_top_toward_negative_y() {
    let scene = Stage::new();
    let mycelium: Vec<IVec3> = scene
        .lower_surface()
        .cells
        .iter()
        .copied()
        .filter(|c| stage_block(&scene, *c) == Some(BlockType::Mycelium))
        .collect();
    assert!(mycelium.len() > 20, "{} mycelium cells", mycelium.len());
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    // Reference: the top texel of an upright mycelium block seen from above.
    let mut single = VoxelWorld::new();
    single.insert(
        IVec3::zero(),
        BlockInstance::new(BlockType::Mycelium, mycelium_material_id(), Orientation::Up),
    );
    let reference_scene = VoxelScene {
        world: &single,
        materials: &materials,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Background::Solid(Color::black()),
        texture_manager: &manager,
        max_distance: 10.0,
    };
    let top = cast_ray_voxel(
        &reference_scene,
        &Ray::new(Vec3::new(0.5, 3.0, 0.5), Vec3::new(0.0, -1.0, 0.0)),
    );
    let bottom = cast_ray_voxel(
        &reference_scene,
        &Ray::new(Vec3::new(0.5, -3.0, 0.5), Vec3::new(0.0, 1.0, 0.0)),
    );
    assert_ne!(top.to_rgba8(), bottom.to_rgba8());
    // In the world, a ray from below the diamond hitting a surface mycelium
    // cell sees that same top texel.
    let view = VoxelScene {
        world: scene.world(),
        materials: &materials,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Background::Solid(Color::black()),
        texture_manager: &manager,
        max_distance: 60.0,
    };
    let cell = mycelium[0];
    let origin = Vec3::new(
        cell.x as f32 + 0.5,
        cell.y as f32 - 3.0,
        cell.z as f32 + 0.5,
    );
    let seen = cast_ray_voxel(&view, &Ray::new(origin, Vec3::new(0.0, 1.0, 0.0)));
    assert_eq!(seen.to_rgba8(), top.to_rgba8());
}

#[test]
fn the_inverted_stairs_reach_the_surface() {
    let scene = WorldScene::new();
    let last = scene.inverted_descent().last().unwrap();
    // A surface cell lies within one column of the last tread, at most two
    // rows away: the flight lands on the ground of the inverted world.
    let near = scene.lower_surface().cells.iter().any(|c| {
        (c.x - last.x).abs() <= 1 && (c.z - last.z).abs() <= 1 && (c.y - last.y).abs() <= 2
    });
    assert!(near, "no surface cell near the last tread {last:?}");
    for (tread, _) in &scene.inverted_descent().treads {
        assert_eq!(block_type(&scene, *tread), Some(BlockType::WoodStairs));
    }
}

#[test]
fn the_surface_has_modest_relief() {
    let scene = Stage::new();
    let s = scene.lower_surface();
    let levels: BTreeSet<i32> = s.cells.iter().map(|c| c.y).collect();
    assert!(levels.len() >= 4, "{levels:?}");
    assert!(!s.raised.is_empty());
    let by_column: HashMap<(i32, i32), i32> = s.cells.iter().map(|c| ((c.x, c.z), c.y)).collect();
    let mut steps = 0;
    let mut pairs = 0;
    for (&(x, z), &y) in &by_column {
        if let Some(&n) = by_column.get(&(x + 1, z)) {
            pairs += 1;
            let d = (y - n).abs();
            assert!(d <= 4, "cliff of {d} at ({x}, {z})");
            if d > 0 {
                steps += 1;
            }
        }
    }
    assert!(steps > pairs / 10, "{steps} steps in {pairs} pairs");
}

#[test]
fn the_surface_uses_authorized_dark_materials_without_symbols_yet() {
    let scene = Stage::new();
    let mut counts: HashMap<BlockType, usize> = HashMap::new();
    for cell in &scene.lower_surface().cells {
        let t = stage_block(&scene, *cell).unwrap();
        *counts.entry(t).or_insert(0) += 1;
        assert!(
            matches!(
                t,
                BlockType::Mycelium
                    | BlockType::SmoothBasalt
                    | BlockType::PolishedBlackstoneBricks
                    | BlockType::RedBlackDeepslateBricksCrimson
                    | BlockType::RedBlackDeepslateBricksOrange
                    | BlockType::RedBlackDeepslateBricksViolet
            ),
            "{t:?} on the surface"
        );
    }
    for t in [
        BlockType::Mycelium,
        BlockType::SmoothBasalt,
        BlockType::PolishedBlackstoneBricks,
    ] {
        assert!(counts[&t] > 10, "{t:?}: {counts:?}");
    }
    let bricks = Family::ALL
        .iter()
        .map(|f| counts.get(&f.bricks().0).copied().unwrap_or(0))
        .sum::<usize>();
    assert!(bricks > 10);
    // Symbols and crystals come with the families.
    let r = scene.rhombus();
    for z in -4..28 {
        for x in -4..28 {
            for y in r.lower_tip_y..r.shelf_y {
                if let Some(t) = stage_block(&scene, IVec3::new(x, y, z)) {
                    assert!(!matches!(
                        t,
                        BlockType::CrimsonHeart
                            | BlockType::CrimsonDiamond
                            | BlockType::OrangeClub
                            | BlockType::OrangeSpade
                            | BlockType::PurpleHeart
                            | BlockType::PurpleDiamond
                            | BlockType::PurpleClub
                            | BlockType::PurpleSpade
                            | BlockType::AmethystCluster
                            | BlockType::BuddingAmethyst
                            | BlockType::NetherWartBlock
                            | BlockType::CryingObsidianCrimson
                            | BlockType::CryingObsidianOrange
                            | BlockType::CryingObsidianViolet
                    ));
                }
            }
        }
    }
    let _ = BlockFamily::RedBlackBase;
}

#[test]
fn the_family_zones_cover_the_surface_in_three_connected_sectors() {
    let scene = Stage::new();
    let mut counts: HashMap<Family, usize> = HashMap::new();
    for cell in &scene.lower_surface().cells {
        *counts
            .entry(family_zone(scene.config(), scene.rhombus(), cell.x, cell.z))
            .or_insert(0) += 1;
    }
    for f in Family::ALL {
        assert!(
            counts.get(&f).copied().unwrap_or(0) > 20,
            "{f:?}: {counts:?}"
        );
    }
    assert_eq!(
        family_zone(scene.config(), scene.rhombus(), 12, 12),
        family_zone(scene.config(), scene.rhombus(), 12, 12)
    );
}

#[test]
fn the_surface_is_visible_from_below_and_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.lower_surface(), b.lower_surface());
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    let view = VoxelScene {
        world: a.world(),
        materials: &materials,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Background::Solid(Color::black()),
        texture_manager: &manager,
        max_distance: 80.0,
    };
    let mut hits = 0;
    for z in 0..24 {
        for x in 0..24 {
            let ray = Ray::new(
                Vec3::new(x as f32 + 0.5, -40.0, z as f32 + 0.5),
                Vec3::new(0.0, 1.0, 0.0),
            );
            if let Some(hit) = nearest_visible_hit(&view, &ray, 80.0) {
                if a.lower_surface().cells.contains(&hit.cell) {
                    hits += 1;
                }
            }
        }
    }
    assert!(hits > 100, "{hits} upward rays reach the surface");
    let src = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(src.matches("world: VoxelWorld,").count(), 1);
}

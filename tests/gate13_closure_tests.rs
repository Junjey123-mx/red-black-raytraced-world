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
use core::hit::Face;
use core::math::{IVec3, Vec2, Vec3};
use scene::block_type::{BlockFamily, BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::geometry_orientation::functional_face;
use scene::light::Light;
use scene::material_gallery::gallery_camera;
use scene::orientation::Orientation;
use scene::red_black_maze::Family;
use scene::rhombus::connected_component;
use scene::texture_manager::TextureManager;
use scene::world::{
    WorldScene, WorldTextures, world_background, world_camera, world_lights, world_materials,
};
use std::collections::HashMap;

fn census(scene: &WorldScene) -> HashMap<BlockType, usize> {
    let r = scene.rhombus();
    let mut counts = HashMap::new();
    for z in -6..30 {
        for x in -6..30 {
            for y in (r.lower_tip_y - 4)..=24 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    *counts.entry(b.block_type()).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

fn count(c: &HashMap<BlockType, usize>, t: BlockType) -> usize {
    c.get(&t).copied().unwrap_or(0)
}

#[test]
fn structure_1_to_9_single_world_overworld_taper_waist_cutaway() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let src = std::fs::read_to_string("src/scene/world.rs").unwrap();
    assert_eq!(src.matches("world: VoxelWorld,").count(), 1); // 1
    assert!(scene.trees()[0].base.y > 0 && scene.house().floor.len() == 35); // 2, 3
    assert_eq!(scene.pond().water.len(), 26); // 4
    assert_eq!(scene.trees().len(), 4); // 5
    let c = census(&scene);
    for t in [
        BlockType::Grass,
        BlockType::Dirt,
        BlockType::Stone,
        BlockType::Deepslate,
    ] {
        assert!(count(&c, t) > 100, "{t:?}"); // 6
    }
    assert!(scene.upper_taper().cells.len() > 800); // 7
    assert!(
        scene
            .world()
            .contains(IVec3::new(r.center_x, r.waist_y, r.center_z))
    ); // 8
    assert!(scene.cutaway().removed_count() > 200); // 9
    assert!(!scene.world().contains(IVec3::new(
        r.cutaway.min_x + 2,
        r.waist_y,
        r.cutaway.min_z + 2
    )));
}

#[test]
fn route_and_portal_10_to_14() {
    let scene = WorldScene::new();
    let d = scene.upper_descent();
    let end = scene.path().end().unwrap();
    let first = d.first().unwrap();
    assert!((first.x - end.x).abs() + (first.z - end.z).abs() <= 2); // 10
    assert_eq!(d.last().unwrap().y, scene.rhombus().shelf_y + 2);
    let c = census(&scene);
    assert_eq!(count(&c, BlockType::PortalFrameRedObsidian), 18); // 11
    assert_eq!(count(&c, BlockType::PortalCoreDarkCrimson), 12); // 12
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let lib = world_materials(&textures);
    let core = scene.world().get(scene.portal().core[0]).unwrap();
    assert!(lib.get(core.material_id()).unwrap().emission_strength > 2.0); // 13
    let points: Vec<_> = world_lights()
        .into_iter()
        .filter_map(|l| match l {
            Light::Point(p) => Some(p),
            _ => None,
        })
        .collect();
    assert!(points.iter().any(|p| p.color.r > 0.7
        && p.color.g < 0.2
        && p.position.y > scene.rhombus().shelf_y as f32)); // 14
}

#[test]
fn inversion_15_to_19_mapping_stairs_surface_mass_tip() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    assert_eq!(
        functional_face(Face::NegativeY, Orientation::Down),
        Face::PositiveY
    ); // 15
    assert_eq!(
        functional_face(Face::PositiveY, Orientation::Up),
        Face::PositiveY
    );
    // 16: the inverted route (Gate 14 rebuilt it behind the portal).
    let inv = scene.inverted_route();
    assert!(inv.layout.step_count >= 5 && inv.treads.iter().all(|(_, o)| *o == Orientation::Down));
    assert!(scene.lower_surface().cells.len() > 150); // 17
    assert!(
        scene
            .lower_surface()
            .cells
            .iter()
            .all(|c| scene.world().get(*c).unwrap().orientation() == Orientation::Down)
    );
    assert!(scene.lower_mass().cells.len() > 1000); // 18
    let tip = IVec3::new(r.center_x, r.lower_tip_y, r.center_z);
    assert!(scene.world().contains(tip)); // 19
    assert_eq!(scene.silhouette().tip, tip);
}

#[test]
fn families_20_to_25_present_connected_emissive_and_bounded_lights() {
    let scene = WorldScene::new();
    let c = census(&scene);
    let families: Vec<Family> = scene.families().iter().map(|f| f.family).collect();
    assert_eq!(families, Family::ALL.to_vec()); // 20-22
    for t in [
        BlockType::CrimsonHeart,
        BlockType::OrangeClub,
        BlockType::PurpleHeart,
        BlockType::AmethystCluster,
        BlockType::NetherWartBlock,
        BlockType::Mycelium,
    ] {
        assert!(count(&c, t) > 0, "{t:?}");
    }
    let main = connected_component(scene.world(), scene.silhouette().tip);
    assert_eq!(main.len(), scene.world().len()); // 23
    assert!(scene.transitions().boundary.len() > 30);
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let lib = world_materials(&textures);
    for f in scene.families() {
        let sym = f
            .symbols
            .iter()
            .find(|s| !scene.transitions().bridged.contains(s))
            .expect("a surviving symbol");
        let b = scene.world().get(*sym).unwrap();
        assert!(
            lib.get(b.material_id()).unwrap().emission_strength > 0.0,
            "{:?}",
            f.family
        ); // 24
    }
    let points = world_lights()
        .iter()
        .filter(|l| matches!(l, Light::Point(_)))
        .count();
    assert_eq!(points, 4); // 25
}

#[test]
fn camera_catalog_and_constraints_26_to_33() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let camera = world_camera(4.0 / 3.0);
    assert!(camera.target.y > r.lower_tip_y as f32 && camera.target.y < 0.0); // 26
    let mut view = DiagnosticCameraState::from_pose(camera.position, camera.target);
    let start = view;
    view.set_angles(view.yaw + 0.5, view.pitch + 0.2);
    view.set_distance(view.distance * 0.7);
    assert_ne!(view, start);
    view.reset();
    assert_eq!(view, start); // 27
    assert_eq!(OFFICIAL_BLOCK_COUNT, 44);
    assert_eq!(official_entries().len(), 44);
    assert_eq!(CatalogScene::new().len(), 46);
    assert_eq!(
        gallery_camera(4.0 / 3.0).position,
        Vec3::new(6.0, 5.6, 14.0)
    ); // 28
    let cargo = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = cargo
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split('[')
        .next()
        .unwrap();
    let crates: Vec<&str> = deps.lines().filter(|l| l.contains('=')).collect();
    assert_eq!(crates.len(), 1, "{crates:?}");
    assert!(crates[0].starts_with("raylib")); // 29
    let a = WorldScene::new();
    assert_eq!(census(&a), census(&scene)); // 30
    assert_eq!(a.world().len(), scene.world().len());
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    for forbidden in ["Player", "gravity", "HUD", "enemy", "pickup"] {
        assert!(!app.contains(forbidden), "{forbidden}"); // 31
    }
    assert!(world_background().is_sky()); // 32 (procedural sky, no cubemap yet)
    assert!(
        !std::fs::read_to_string("src/renderer/skybox.rs")
            .unwrap()
            .contains("Cubemap {")
    );
    // 33: Gate 13 shipped no parallel renderer; Gate 13.6 filled
    // `parallel.rs` with scoped std threads (still no rayon, and the
    // viewer itself spawns nothing: tracing stays inside the renderer).
    let parallel = std::fs::read_to_string("src/renderer/parallel.rs").unwrap();
    assert!(parallel.contains("std::thread::scope") && !parallel.contains("rayon"));
    assert!(!app.contains("std::thread"));
    for t in census(&scene).keys() {
        let _ = t.family() as BlockFamily;
    }
}

#[test]
fn gate13_metrics() {
    let scene = WorldScene::new();
    let r = scene.rhombus();
    let c = census(&scene);
    let total = scene.world().len();
    let upper = (0..total).len();
    let mut upper_count = 0;
    let mut lower_count = 0;
    let mut upper_max = i32::MIN;
    let mut lower_min = i32::MAX;
    for z in -6..30 {
        for x in -6..30 {
            for y in (r.lower_tip_y - 4)..=24 {
                if scene.world().contains(IVec3::new(x, y, z)) {
                    if y >= r.shelf_y {
                        upper_count += 1;
                        upper_max = upper_max.max(y);
                    } else {
                        lower_count += 1;
                        lower_min = lower_min.min(y);
                    }
                }
            }
        }
    }
    let _ = upper;
    let width = |y: i32| {
        (-6..30)
            .filter(|&x| scene.world().contains(IVec3::new(x, y, r.center_z)))
            .count()
    };
    let crimson = [
        BlockType::CrimsonHeart,
        BlockType::CrimsonDiamond,
        BlockType::CryingObsidianCrimson,
        BlockType::NetherWartBlock,
        BlockType::RedBlackDeepslateBricksCrimson,
    ]
    .iter()
    .map(|t| count(&c, *t))
    .sum::<usize>();
    let orange = [
        BlockType::OrangeClub,
        BlockType::OrangeSpade,
        BlockType::CryingObsidianOrange,
        BlockType::RedBlackDeepslateBricksOrange,
    ]
    .iter()
    .map(|t| count(&c, *t))
    .sum::<usize>();
    let violet = [
        BlockType::PurpleHeart,
        BlockType::PurpleDiamond,
        BlockType::PurpleClub,
        BlockType::PurpleSpade,
        BlockType::BuddingAmethyst,
        BlockType::AmethystCluster,
        BlockType::CryingObsidianViolet,
        BlockType::RedBlackDeepslateBricksViolet,
    ]
    .iter()
    .map(|t| count(&c, *t))
    .sum::<usize>();
    let points = world_lights()
        .iter()
        .filter(|l| matches!(l, Light::Point(_)))
        .count();
    let lines = [
        format!("total voxel count: {total}"),
        format!(
            "Overworld voxel count (y >= shelf {}): {upper_count}",
            r.shelf_y
        ),
        format!("lower-half voxel count (y < shelf): {lower_count}"),
        format!(
            "upper bounds: x 0..23, z 0..23, y {}..{} (surface {}..{}, roof/trees to {upper_max})",
            r.shelf_y,
            upper_max,
            scene.config().min_surface_height(),
            r.upper_surface_reference
        ),
        format!(
            "waist y: {} (width through center {} vs surface {})",
            r.waist_y,
            width(r.waist_y),
            width(r.upper_taper_top)
        ),
        format!(
            "lower bounds: y {}..{} (widest row {} width {})",
            lower_min,
            r.shelf_y - 1,
            r.lower_widest_y,
            width(r.lower_widest_y)
        ),
        format!(
            "lower tip y: {} (width {})",
            r.lower_tip_y,
            width(r.lower_tip_y)
        ),
        format!(
            "cutaway removed voxel estimate: {}",
            scene.cutaway().removed_count()
        ),
        format!(
            "portal frame count: {}",
            count(&c, BlockType::PortalFrameRedObsidian)
        ),
        format!(
            "portal core count: {}",
            count(&c, BlockType::PortalCoreDarkCrimson)
        ),
        format!("upper stair count: {}", scene.upper_descent().treads.len()),
        format!("lower stair count: {}", scene.inverted_route().treads.len()),
        format!("Crimson block count: {crimson}"),
        format!(
            "Orange block count: {orange} (+ {} SmoothBasalt, {} PolishedBlackstoneBricks shared)",
            count(&c, BlockType::SmoothBasalt),
            count(&c, BlockType::PolishedBlackstoneBricks)
        ),
        format!(
            "Violet block count: {violet} (+ {} Mycelium shared)",
            count(&c, BlockType::Mycelium)
        ),
        format!("PointLight count: {points} (+ 1 sun)"),
        format!(
            "family boundary columns bridged: {}",
            scene.transitions().bridged.len()
        ),
    ];
    for line in &lines {
        println!("GATE13 {line}");
    }
    assert!(total > 6000 && lower_count > 1500 && upper_count > 4000);
}

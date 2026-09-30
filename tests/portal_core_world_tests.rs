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
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
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

use core::math::IVec3;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{CatalogTextures, catalog_materials};
use scene::light::Light;
use scene::material_gallery::portal_core_material_id;
use scene::orientation::Orientation;
use scene::portal::{PORTAL_LIGHT_INTENSITY, portal_light_color, portal_light_position};
use scene::texture_manager::TextureManager;
use scene::world::{WorldScene, WorldTextures, world_lights, world_materials};

#[test]
fn the_core_fills_the_opening_inside_the_frame() {
    let scene = WorldScene::new();
    let p = scene.rhombus().portal;
    let core = &scene.portal().core;
    assert_eq!(core.len(), ((p.width() - 2) * (p.height() - 2)) as usize);
    assert_eq!(core.len(), 12);
    for cell in core {
        assert!(p.is_opening(cell.x, cell.y));
        assert_eq!(cell.z, p.wall_z);
        let b = scene.world().get(*cell).unwrap();
        assert_eq!(b.block_type(), BlockType::PortalCoreDarkCrimson);
        assert_eq!(b.material_id(), portal_core_material_id());
        assert_eq!(b.orientation(), p.facing);
        assert_eq!(b.orientation(), Orientation::South);
        // Framed on every side by obsidian or another core cell.
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = IVec3::new(cell.x + dx, cell.y + dy, cell.z);
            let t = scene.world().get(n).map(|b| b.block_type());
            assert!(
                matches!(
                    t,
                    Some(BlockType::PortalFrameRedObsidian | BlockType::PortalCoreDarkCrimson)
                ),
                "{n:?}: {t:?}"
            );
        }
    }
}

#[test]
fn the_membrane_is_a_thin_partial_geometry() {
    let geometry = block_geometry(BlockType::PortalCoreDarkCrimson, Orientation::South);
    match geometry {
        BlockGeometry::Prism(prism) => {
            let size = prism.size();
            assert!(size.z < 0.2 && size.x > 0.99 && size.y > 0.99, "{size:?}");
            assert!(prism.is_within_unit_cell());
        }
        other => panic!("{other:?} is not a thin prism"),
    }
}

#[test]
fn the_core_is_emissive_and_semitransparent_but_not_glass() {
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let lib = world_materials(&textures);
    let core = lib.get(portal_core_material_id()).expect("core material");
    assert!(core.emission_strength > 2.0);
    assert!(core.emissive_texture.is_some());
    assert!(
        core.transparency >= 0.2 && core.transparency <= 0.35,
        "{}",
        core.transparency
    );
    assert_eq!(core.refractive_index, 1.0);
    assert!(core.reflectivity < 0.1);
    // Dark wine, not saturated fuchsia.
    assert!(core.albedo.r > core.albedo.g && core.albedo.r < 0.6 && core.albedo.b < core.albedo.r);
    // Identical to the catalog's core material.
    let mut manager2 = TextureManager::new();
    let catalog_textures = CatalogTextures::load(
        &mut manager2,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let catalog = catalog_materials(&catalog_textures);
    let reference = catalog.get(portal_core_material_id()).unwrap();
    assert_eq!(core.albedo, reference.albedo);
    assert_eq!(core.emission_strength, reference.emission_strength);
    assert_eq!(core.transparency, reference.transparency);
    assert_eq!(core.specular, reference.specular);
}

#[test]
fn a_bounded_crimson_point_light_sits_in_front_of_the_membrane() {
    let scene = WorldScene::new();
    let lights = world_lights();
    let points: Vec<_> = lights
        .iter()
        .filter_map(|l| match l {
            Light::Point(p) => Some(p),
            Light::Directional(_) => None,
        })
        .collect();
    let light = points
        .iter()
        .find(|p| p.color == portal_light_color())
        .expect("portal light");
    let expected = portal_light_position(scene.rhombus());
    assert_eq!(light.position, expected);
    let p = scene.rhombus().portal;
    assert!(light.position.x > p.min_x as f32 && light.position.x < p.max_x as f32 + 1.0);
    assert!(light.position.y > p.min_y as f32 && light.position.y < p.max_y as f32 + 1.0);
    assert!(light.position.z > p.wall_z as f32 + 1.0 && light.position.z < p.wall_z as f32 + 3.0);
    assert_eq!(light.color, portal_light_color());
    assert!(light.color.r > light.color.g && light.color.r > light.color.b);
    assert!(light.intensity > 0.0 && light.intensity <= 0.5);
    assert_eq!(light.intensity, PORTAL_LIGHT_INTENSITY);
    // The sun still dominates.
    let sun = lights.iter().find_map(|l| match l {
        Light::Directional(d) => Some(d),
        _ => None,
    });
    assert!(sun.unwrap().intensity > light.intensity * 2.0);
}

#[test]
fn the_portal_is_deterministic() {
    let a = WorldScene::new();
    let b = WorldScene::new();
    assert_eq!(a.portal(), b.portal());
    assert_eq!(a.portal().frame.len(), 18);
    assert!(
        a.portal()
            .opening
            .iter()
            .all(|c| a.portal().core.contains(c))
    );
}

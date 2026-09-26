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
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
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
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
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
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use core::color::Color;
use core::hit::Face;
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::normal_mapping::{decode_normal, perturb_normal, sample_shading_normal};
use renderer::raytracer::{VoxelScene, nearest_voxel_hit, trace_ray};
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::deepslate_bricks_material_id;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const RANGE: f32 = 60.0;
const N: usize = 16;

struct Env {
    manager: TextureManager,
    library: MaterialLibrary,
}

fn env() -> Env {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let library = catalog_materials(&textures);
    Env { manager, library }
}

fn bricks_world() -> VoxelWorld {
    let mut w = VoxelWorld::new();
    w.insert(
        IVec3::new(0, 0, 0),
        scene::block::BlockInstance::new(
            BlockType::DeepslateBricks,
            deepslate_bricks_material_id(),
            Orientation::Up,
        ),
    );
    w
}

fn front_ray(i: usize, j: usize) -> Ray {
    Ray::new(
        Vec3::new((i as f32 + 0.5) / 16.0, 1.0 - (j as f32 + 0.5) / 16.0, 6.0),
        Vec3::new(0.0, 0.0, -1.0),
    )
}

#[test]
fn deepslate_bricks_is_the_official_full_cube_block_type() {
    assert!(matches!(
        block_geometry(BlockType::DeepslateBricks, Orientation::Up),
        BlockGeometry::FullCube
    ));
}

#[test]
fn albedo_normal_and_material_resolve_with_the_reference_profile() {
    let e = env();
    let m = e.library.get(deepslate_bricks_material_id()).unwrap();

    let faces = m.face_textures.unwrap();
    for f in [
        Face::PositiveX,
        Face::PositiveY,
        Face::NegativeY,
        Face::NegativeZ,
    ] {
        let t = e
            .manager
            .get(faces.texture_for_face(f))
            .expect("albedo resolves");
        assert_eq!((t.width(), t.height()), (16, 16));
    }
    let normal = e
        .manager
        .get(m.normal_texture.expect("normal map"))
        .expect("normal resolves");
    assert_eq!((normal.width(), normal.height()), (16, 16));

    assert!((m.specular - 0.08).abs() < 1e-6 && (m.shininess - 14.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.02).abs() < 1e-6);
}

#[test]
fn the_block_is_opaque_and_non_emissive() {
    let e = env();
    let m = e.library.get(deepslate_bricks_material_id()).unwrap();
    assert_eq!(m.transparency, 0.0);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none());
    assert_eq!(m.refractive_index, 1.0);
    let f = m.face_textures.unwrap();
    let t = e.manager.get(f.texture_for_face(Face::PositiveZ)).unwrap();
    assert!(t.pixels().iter().all(|c| c.a == 1.0));
}

#[test]
fn the_albedo_shows_two_courses_of_bricks_with_dark_joints() {
    let e = env();
    let m = e.library.get(deepslate_bricks_material_id()).unwrap();
    let t = e
        .manager
        .get(m.face_textures.unwrap().texture_for_face(Face::PositiveZ))
        .unwrap();
    let lum = |x: usize, y: usize| {
        let c = t.texel(x, y).unwrap();
        0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
    };
    // The horizontal joint between the two courses is dark across the row.
    let row_mean = |y: usize| (0..N).map(|x| lum(x, y)).sum::<f32>() / N as f32;
    let darkest_row = (0..N).map(row_mean).fold(1.0, f32::min);
    let overall = (0..N).map(row_mean).sum::<f32>() / N as f32;
    assert!(
        darkest_row < overall * 0.7,
        "joint row {darkest_row} vs {overall}"
    );
}

#[test]
fn it_is_a_single_official_catalog_entry_replacing_the_normal_map_sample() {
    let scene = CatalogScene::new();
    let entries: Vec<_> = scene
        .entries()
        .iter()
        .filter(|e| e.block_type == BlockType::DeepslateBricks)
        .collect();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display_name, "Deepslate Bricks");
    assert_eq!(entries[0].kind, SampleKind::NormalMapped);
    assert_eq!(entries[0].material_id, deepslate_bricks_material_id());
    assert_eq!(
        scene.world().get(entries[0].position),
        Some(&entries[0].block())
    );
    assert!(
        !catalog_entries()
            .iter()
            .any(|e| e.display_name.to_lowercase().contains("normal map"))
    );
    let p = entries[0].focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
}

#[test]
fn the_normal_map_changes_the_shading_normal() {
    let e = env();
    let m = e.library.get(deepslate_bricks_material_id()).unwrap();
    let flat = Face::PositiveZ.normal();

    let tilted = (0..N * N)
        .filter(|n| {
            let uv = Vec2::new(((n % N) as f32 + 0.5) / 16.0, ((n / N) as f32 + 0.5) / 16.0);
            let s = sample_shading_normal(m, Face::PositiveZ, uv, flat, &e.manager);
            (s - flat).length() > 0.05
        })
        .count();
    assert!(tilted > 30, "{tilted} texels are perturbed");

    let n = perturb_normal(
        flat,
        Face::PositiveZ,
        decode_normal(Color::new(0.9, 0.5, 0.9, 1.0)),
    );
    assert!((n.length() - 1.0).abs() < 1e-4);
}

#[test]
fn the_normal_map_changes_lit_pixels_without_changing_geometry() {
    let e = env();
    let mut plain_material = e
        .library
        .get(deepslate_bricks_material_id())
        .unwrap()
        .clone();
    plain_material.normal_texture = None;
    let mut library = MaterialLibrary::new();
    library.insert(deepslate_bricks_material_id(), plain_material);

    let world = bricks_world();
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.7, 0.5, 0.5),
        Color::white(),
        1.0,
    ))];
    let shade = |materials: &MaterialLibrary, r: &Ray| {
        let scene = VoxelScene {
            world: &world,
            materials,
            camera_position: r.origin,
            lights: &lights,
            ambient_factor: 0.1,
            background: Color::black(),
            texture_manager: &e.manager,
            max_distance: RANGE,
        };
        trace_ray(&scene, r, 0)
    };

    let mut changed = 0;
    for j in 0..N {
        for i in 0..N {
            let r = front_ray(i, j);
            // Geometry: the recorded normal is the exact face normal and the
            // hit point lies on the cube's plane — the traversal never sees
            // the normal map.
            let hit = nearest_voxel_hit(&world, &r, RANGE).unwrap();
            assert_eq!(hit.hit.normal, Face::PositiveZ.normal());
            assert!((hit.hit.point.z - 1.0).abs() < 1e-4);

            let a = shade(&e.library, &r);
            let b = shade(&library, &r);
            if (a.r - b.r).abs() > 0.01 {
                changed += 1;
            }
        }
    }
    assert!(changed > 30, "only {changed} texels changed shading");
}

#[test]
fn the_silhouette_stays_a_cube() {
    let world = bricks_world();
    // Just outside the cube on every side: relief adds no geometry.
    for (o, d) in [
        ((1.02, 0.5, 6.0), (0.0, 0.0, -1.0)),
        ((-0.02, 0.5, 6.0), (0.0, 0.0, -1.0)),
        ((0.5, 1.02, 6.0), (0.0, 0.0, -1.0)),
        ((0.5, -0.02, 6.0), (0.0, 0.0, -1.0)),
    ] {
        let r = Ray::new(Vec3::new(o.0, o.1, o.2), Vec3::new(d.0, d.1, d.2));
        assert!(nearest_voxel_hit(&world, &r, RANGE).is_none());
    }
    let inside = Ray::new(Vec3::new(0.98, 0.98, 6.0), Vec3::new(0.0, 0.0, -1.0));
    assert!(nearest_voxel_hit(&world, &inside, RANGE).is_some());
}

#[test]
fn the_texture_assets_are_the_single_shared_reference_derived_files() {
    // One albedo and one normal texture are loaded for the block: reusing the
    // Gate 07 assets, not a second copy under another path.
    let e = env();
    let m = e.library.get(deepslate_bricks_material_id()).unwrap();
    let f = m.face_textures.unwrap();
    let ids: Vec<_> = [
        Face::PositiveX,
        Face::PositiveY,
        Face::NegativeY,
        Face::PositiveZ,
        Face::NegativeZ,
    ]
    .iter()
    .map(|x| f.texture_for_face(*x))
    .collect();
    assert!(ids.iter().all(|i| *i == ids[0]), "one albedo on all faces");
    assert_ne!(Some(ids[0]), m.normal_texture);
    for dir in ["assets/textures/overworld/deepslate_bricks"] {
        let mut names: Vec<_> = std::fs::read_dir(format!("{}/{dir}", env!("CARGO_MANIFEST_DIR")))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, ["albedo.png", "normal.png"]);
    }
}

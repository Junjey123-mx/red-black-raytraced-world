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
use core::material::{Material, MaterialId};
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, light_visibility, nearest_visible_hit, trace_ray};
use renderer::texture_sampling::sample_nearest;
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::{
    CatalogScene, CatalogTextures, SampleKind, catalog_entries, catalog_materials,
};
use scene::material_gallery::{glass_material_id, water_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-3;
const RANGE: f32 = 60.0;
const RED: u32 = 200;
const GREEN: u32 = 201;

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
    let mut library = catalog_materials(&textures);
    library.insert(
        MaterialId::new(RED),
        Material::matte(Color::new(1.0, 0.0, 0.0, 1.0)),
    );
    library.insert(
        MaterialId::new(GREEN),
        Material::matte(Color::new(0.0, 1.0, 0.0, 1.0)),
    );
    Env { manager, library }
}

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(o: (f32, f32, f32), d: (f32, f32, f32)) -> Ray {
    Ray::new(Vec3::new(o.0, o.1, o.2), Vec3::new(d.0, d.1, d.2))
}

fn block(t: BlockType, id: MaterialId) -> BlockInstance {
    BlockInstance::new(t, id, Orientation::Up)
}

impl Env {
    fn trace(&self, world: &VoxelWorld, ambient: f32, r: &Ray) -> Color {
        let scene = VoxelScene {
            world,
            materials: &self.library,
            camera_position: r.origin,
            lights: &[],
            ambient_factor: ambient,
            background: Color::new(0.0, 0.0, 0.0, 1.0),
            texture_manager: &self.manager,
            max_distance: RANGE,
        };
        trace_ray(&scene, r, 0)
    }
}

/// Two water cells stacked in y with a red block off to the +x side at the
/// upper cell's height, and the ray geometry that a real air/water interface
/// between the two cells would trap by total internal reflection.
fn stacked_pair(second: MaterialId) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 1, 0), block(BlockType::Water, water_material_id()));
    world.insert(cell(0, 2, 0), block(BlockType::Water, second));
    world.insert(cell(3, 2, 0), block(BlockType::Stone, MaterialId::new(RED)));
    world
}

/// Enters the lower cell just under the shared face, rising gently.
fn grazing_ray() -> Ray {
    let angle = 20.0_f32.to_radians();
    let (dx, dy) = (angle.cos(), angle.sin());
    let run = 2.0;
    ray((-run, 1.9 - run * dy / dx, 0.5), (dx, dy, 0.0))
}

#[test]
fn water_has_its_definitive_material_id_and_profile() {
    let e = env();
    let m = e.library.get(water_material_id()).unwrap();

    assert!((m.specular - 0.55).abs() < 1e-6 && (m.shininess - 80.0).abs() < 1e-6);
    assert!((m.reflectivity - 0.18).abs() < 1e-6);
    assert!((m.transparency - 0.78).abs() < 1e-6);
    assert!((m.refractive_index - 1.33).abs() < 1e-6);
    assert_eq!(m.emission_strength, 0.0);
    assert!(m.emissive_texture.is_none() && m.normal_texture.is_none());
}

#[test]
fn water_is_a_full_cube_with_a_resolving_reference_texture() {
    let e = env();
    assert!(matches!(
        block_geometry(BlockType::Water, Orientation::Up),
        BlockGeometry::FullCube
    ));

    let f = e
        .library
        .get(water_material_id())
        .unwrap()
        .face_textures
        .unwrap();
    for face in [
        Face::PositiveX,
        Face::PositiveY,
        Face::NegativeY,
        Face::NegativeZ,
    ] {
        let t = e
            .manager
            .get(f.texture_for_face(face))
            .expect("texture resolves");
        assert_eq!((t.width(), t.height()), (16, 16));
        // Blue-dominant pixel art (the reference is a blue water surface).
        let blue: usize = t.pixels().iter().filter(|c| c.b > c.r && c.b > c.g).count();
        assert!(blue > 240, "{blue} blue texels");
    }
    // Nearest-neighbor: one constant color inside a texel.
    let t = e.manager.get(f.texture_for_face(Face::PositiveY)).unwrap();
    let a = sample_nearest(t, Vec2::new(3.05 / 16.0, 4.05 / 16.0));
    let b = sample_nearest(t, Vec2::new(3.95 / 16.0, 4.95 / 16.0));
    assert_eq!(a, b);
}

#[test]
fn water_is_one_official_catalog_entry_and_not_duplicated() {
    let scene = CatalogScene::new();
    let entries: Vec<_> = scene
        .entries()
        .iter()
        .filter(|e| e.block_type == BlockType::Water)
        .collect();

    assert_eq!(entries.len(), 1, "no Water diagnostic + Water final");
    let entry = entries[0];
    assert_eq!(entry.display_name, "Water");
    assert_eq!(entry.kind, SampleKind::Transmissive);
    assert_eq!(entry.material_id, water_material_id());
    assert_eq!(scene.world().get(entry.position), Some(&entry.block()));
    let p = entry.focus_point;
    assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
    assert_eq!(
        catalog_entries()
            .iter()
            .filter(|e| e.display_name.contains("Water"))
            .count(),
        1
    );
}

#[test]
fn an_object_behind_water_is_visible_and_refracted() {
    let e = env();
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 1, 0), block(BlockType::Water, water_material_id()));
    world.insert(
        cell(0, 1, -4),
        block(BlockType::Stone, MaterialId::new(RED)),
    );

    let straight = e.trace(&world, 1.0, &ray((0.5, 1.5, 5.0), (0.0, 0.0, -1.0)));
    assert!(
        straight.r > 0.4 && straight.r > 2.0 * straight.b,
        "red shows through: {straight:?}"
    );

    // An oblique ray is bent: in air it lands on the green block; the water pulls
    // it back toward the normal onto the red one.
    let mut two = VoxelWorld::new();
    two.insert(
        cell(1, 1, -4),
        block(BlockType::Stone, MaterialId::new(RED)),
    );
    two.insert(
        cell(2, 1, -4),
        block(BlockType::Stone, MaterialId::new(GREEN)),
    );
    let oblique = ray((-0.75, 1.5, 5.0), (0.35, 0.0, -1.0));
    let through_air = e.trace(&two, 1.0, &oblique);
    two.insert(cell(0, 1, 0), block(BlockType::Water, water_material_id()));
    let through_water = e.trace(&two, 1.0, &oblique);

    assert!(
        through_air.g > 0.9 && through_air.r < 0.05,
        "air -> green: {through_air:?}"
    );
    assert!(
        through_water.r > 0.4 && through_water.r > through_water.g,
        "water -> red: {through_water:?}"
    );
}

#[test]
fn two_contiguous_water_cells_have_no_false_internal_interface() {
    let e = env();
    let world = stacked_pair(water_material_id());
    let scene = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Color::new(0.0, 0.0, 0.0, 1.0),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };

    // The visible hits along the ray never include the shared face (y = 2):
    // only the real air/water faces are surfaces.
    let r = grazing_ray();
    let first = nearest_visible_hit(&scene, &r, RANGE).expect("enters the lower cell");
    assert_eq!(first.cell, cell(0, 1, 0));

    // Straight down through both cells: entry on top of B, exit under A; the
    // shared face between them is invisible.
    let down = ray((0.5, 6.0, 0.5), (0.0, -1.0, 0.0));
    let top = nearest_visible_hit(&scene, &down, RANGE).unwrap();
    assert_eq!(top.cell, cell(0, 2, 0));
    let inside = ray((0.5, 2.4, 0.5), (0.0, -1.0, 0.0));
    let next = nearest_visible_hit(&scene, &inside, RANGE).unwrap();
    assert_eq!(
        next.hit.face,
        Face::NegativeY,
        "skips the shared face, exits underneath"
    );
    assert!((next.hit.point.y - 1.0).abs() < EPS);
}

#[test]
fn a_shallow_ray_is_not_trapped_by_the_shared_face_between_water_cells() {
    let e = env();
    let r = grazing_ray();

    // One body of water: the ray climbs through both cells, leaves the upper
    // one and sees the red block.
    let one_body = e.trace(&stacked_pair(water_material_id()), 1.0, &r);
    // A different medium above (glass) makes the shared face a real
    // interface, so the ray is totally internally reflected instead.
    let two_media = e.trace(&stacked_pair(glass_material_id()), 1.0, &r);

    assert!(
        one_body.r > 0.3 && one_body.r > 2.0 * one_body.b,
        "one body: {one_body:?}"
    );
    assert!(
        one_body.r > two_media.r + 0.2,
        "the real interface must trap what the shared water face does not"
    );
    for c in [one_body, two_media] {
        assert!(c.r.is_finite() && c.g.is_finite() && c.b.is_finite());
    }
}

#[test]
fn only_faces_toward_a_same_medium_neighbor_are_removed() {
    let e = env();
    // A lone water cell keeps every face: rays still hit it from all sides.
    let mut lone = VoxelWorld::new();
    lone.insert(cell(0, 1, 0), block(BlockType::Water, water_material_id()));
    let scene = |world: &VoxelWorld| -> Option<IVec3> {
        let s = VoxelScene {
            world,
            materials: &e.library,
            camera_position: Vec3::zero(),
            lights: &[],
            ambient_factor: 1.0,
            background: Color::new(0.0, 0.0, 0.0, 1.0),
            texture_manager: &e.manager,
            max_distance: RANGE,
        };
        [
            ray((0.5, 1.5, 5.0), (0.0, 0.0, -1.0)),
            ray((5.0, 1.5, 0.5), (-1.0, 0.0, 0.0)),
            ray((0.5, 6.0, 0.5), (0.0, -1.0, 0.0)),
        ]
        .iter()
        .all(|r| nearest_visible_hit(&s, r, RANGE).is_some())
        .then_some(cell(0, 1, 0))
    };
    assert!(scene(&lone).is_some());

    // Water next to a *different* block keeps the shared face too.
    let mut mixed = lone;
    mixed.insert(cell(1, 1, 0), block(BlockType::Stone, MaterialId::new(RED)));
    let s = VoxelScene {
        world: &mixed,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Color::new(0.0, 0.0, 0.0, 1.0),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };
    let from_side = ray((-5.0, 1.5, 0.5), (1.0, 0.0, 0.0));
    assert_eq!(
        nearest_visible_hit(&s, &from_side, RANGE).unwrap().cell,
        cell(0, 1, 0)
    );
}

#[test]
fn water_still_reflects_partially_and_never_emits() {
    let e = env();
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::Water, water_material_id()));
    // Blue background reflected by the top face at a shallow angle.
    let scene = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 0.0,
        background: Color::new(0.0, 0.0, 1.0, 1.0),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };
    let c = trace_ray(&scene, &ray((-3.0, 3.0, 0.5), (1.0, -0.9, 0.0)), 0);
    assert!(
        c.b > 0.1,
        "the reflected ray sees the blue background: {c:?}"
    );
    assert!(c.r < 0.05, "no emission: {c:?}");
}

#[test]
fn shadows_through_water_are_attenuated_not_solid_black() {
    let e = env();
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 1, 0), block(BlockType::Water, water_material_id()));
    let scene = VoxelScene {
        world: &world,
        materials: &e.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: Color::new(0.0, 0.0, 0.0, 1.0),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };
    let up = ray((0.5, 0.9999, 0.5), (0.0, 1.0, 0.0));
    let v = light_visibility(&scene, &up, 10.0);
    // Enters and leaves the cell: transparency squared, strictly between 0 and 1.
    assert!(v > 0.3 && v < 0.9, "visibility {v}");
    let m = e.library.get(water_material_id()).unwrap();
    assert!((v - m.transparency * m.transparency).abs() < 0.02);
}

#[test]
fn contiguous_water_shadows_count_only_the_real_boundaries() {
    let e = env();
    let one = |cells: &[IVec3]| {
        let mut world = VoxelWorld::new();
        for c in cells {
            world.insert(*c, block(BlockType::Water, water_material_id()));
        }
        let scene = VoxelScene {
            world: &world,
            materials: &e.library,
            camera_position: Vec3::zero(),
            lights: &[],
            ambient_factor: 1.0,
            background: Color::new(0.0, 0.0, 0.0, 1.0),
            texture_manager: &e.manager,
            max_distance: RANGE,
        };
        light_visibility(&scene, &ray((0.5, 0.9999, 0.5), (0.0, 1.0, 0.0)), 20.0)
    };

    let single = one(&[cell(0, 1, 0)]);
    let stacked = one(&[cell(0, 1, 0), cell(0, 2, 0), cell(0, 3, 0)]);
    assert!(
        (single - stacked).abs() < 1e-3,
        "a taller body of water has the same two boundaries: {single} vs {stacked}"
    );
}

#[test]
fn rays_through_water_stay_finite_and_no_hydraulics_exist() {
    let e = env();
    let mut world = VoxelWorld::new();
    for x in 0..4 {
        for y in 1..4 {
            for z in 0..4 {
                world.insert(cell(x, y, z), block(BlockType::Water, water_material_id()));
            }
        }
    }
    for i in 0..40 {
        let t = i as f32 * 0.17;
        let r = ray(
            (-3.0 + t.sin(), 2.0 + t.cos(), -3.0),
            (0.6 * t.sin(), 0.2 * t.cos(), 1.0),
        );
        let c = e.trace(&world, 0.5, &r);
        assert!(c.r.is_finite() && c.g.is_finite() && c.b.is_finite());
        assert!((0.0..=1.0).contains(&c.b));
    }

    // Static data only: no fluid or flow code in the block/catalog modules.
    for path in [
        "src/scene/catalog.rs",
        "src/scene/overworld_blocks.rs",
        "src/renderer/raytracer.rs",
    ] {
        let source =
            std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let lower = source.to_lowercase();
        assert!(
            !lower.contains("hydraulic")
                && !lower.contains("fluid_sim")
                && !lower.contains("flow_level"),
            "{path}"
        );
    }
}

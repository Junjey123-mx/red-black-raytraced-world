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
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
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
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::emission::sample_emissive;
use renderer::raytracer::{VoxelScene, trace_ray};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::{
    GalleryTextures, advanced_materials_library, portal_core_material_id, redstone_lamp_material_id,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 50.0;
const N: usize = 16;
const WALL: u32 = 90;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

fn approx_color(a: Color, b: Color) -> bool {
    approx(a.r, b.r) && approx(a.g, b.g) && approx(a.b, b.b)
}

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn black() -> Color {
    Color::new(0.0, 0.0, 0.0, 1.0)
}

fn is_black(c: Color) -> bool {
    approx_color(c, black())
}

struct Fixture {
    manager: TextureManager,
    library: MaterialLibrary,
}

fn fixture() -> Fixture {
    let mut manager = TextureManager::new();
    let textures = GalleryTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
    )
    .unwrap();
    let mut library = advanced_materials_library(&textures);
    library.insert(
        MaterialId::new(WALL),
        Material::matte(Color::new(0.5, 0.5, 0.5, 1.0)),
    );
    Fixture { manager, library }
}

impl Fixture {
    fn trace(
        &self,
        world: &VoxelWorld,
        lights: &[Light],
        ambient: f32,
        background: Color,
        r: &Ray,
    ) -> Color {
        let scene = VoxelScene {
            world,
            materials: &self.library,
            camera_position: r.origin,
            lights,
            ambient_factor: ambient,
            background,
            texture_manager: &self.manager,
            max_distance: RANGE,
        };
        trace_ray(&scene, r, 0)
    }

    fn emissive_at(&self, id: MaterialId, i: usize, j: usize) -> Color {
        sample_emissive(self.library.get(id).unwrap(), uv(i, j), &self.manager)
    }

    /// All `(i, j)` texels whose emitted color is non-black.
    fn lit_texels(&self, id: MaterialId) -> Vec<(usize, usize)> {
        (0..N)
            .flat_map(|j| (0..N).map(move |i| (i, j)))
            .filter(|&(i, j)| !is_black(self.emissive_at(id, i, j)))
            .collect()
    }
}

fn uv(i: usize, j: usize) -> core::math::Vec2 {
    core::math::Vec2::new((i as f32 + 0.5) / N as f32, (j as f32 + 0.5) / N as f32)
}

/// A single cube of the given block/material at the origin cell.
fn lone_block(block_type: BlockType, id: MaterialId, orientation: Orientation) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        BlockInstance::new(block_type, id, orientation),
    );
    world
}

/// Eye ray hitting texel `(i, j)` of the +Z face of the origin cell head-on
/// (`uv = (lx, 1 - ly)`).
fn front_ray(i: usize, j: usize) -> Ray {
    let lx = (i as f32 + 0.5) / N as f32;
    let ly = 1.0 - (j as f32 + 0.5) / N as f32;
    ray((lx, ly, 9.0), (0.0, 0.0, -1.0))
}

fn lamp_world() -> VoxelWorld {
    lone_block(
        BlockType::RedstoneLampLit,
        redstone_lamp_material_id(),
        Orientation::Up,
    )
}

fn clamp_unit(c: Color) -> Color {
    Color::new(c.r.min(1.0), c.g.min(1.0), c.b.min(1.0), 1.0)
}

#[test]
fn emission_is_visible_without_any_direct_light() {
    let f = fixture();
    let world = lamp_world();
    let (i, j) = f.lit_texels(redstone_lamp_material_id())[0];

    // No lights, no ambient: only emission can light this pixel.
    let color = f.trace(&world, &[], 0.0, black(), &front_ray(i, j));
    let expected = clamp_unit(f.emissive_at(redstone_lamp_material_id(), i, j));

    assert!(!is_black(color), "lamp panel must glow in the dark");
    assert!(approx_color(color, expected), "{color:?} vs {expected:?}");
}

#[test]
fn a_black_mask_texel_contributes_no_emission() {
    let f = fixture();
    let world = lamp_world();

    // The outer frame ring is dark in the mask.
    for k in 0..N {
        let color = f.trace(&world, &[], 0.0, black(), &front_ray(k, 0));
        assert!(is_black(color), "frame texel ({k},0) glowed: {color:?}");
    }
}

#[test]
fn zero_emission_strength_removes_the_emission() {
    let mut f = fixture();
    let id = redstone_lamp_material_id();
    let (i, j) = f.lit_texels(id)[0];
    let off = f
        .library
        .get(id)
        .unwrap()
        .clone()
        .with_emission_strength(0.0);
    f.library.insert(id, off);

    let color = f.trace(&lamp_world(), &[], 0.0, black(), &front_ray(i, j));
    assert!(is_black(color), "color = {color:?}");
}

#[test]
fn the_redstone_lamp_keeps_its_albedo_and_only_adds_emission() {
    let f = fixture();
    let world = lamp_world();
    let id = redstone_lamp_material_id();
    let material = f.library.get(id).unwrap();
    let albedo_id = material
        .face_textures
        .unwrap()
        .texture_for_face(Face::PositiveZ);
    let albedo = f.manager.get(albedo_id).unwrap();
    let ambient = 0.5;

    // Frame texel: purely the lit (ambient) albedo, no emission.
    let frame = f.trace(&world, &[], ambient, black(), &front_ray(0, 0));
    let a = albedo.texel(0, 0).unwrap();
    assert!(approx_color(
        frame,
        Color::new(a.r * ambient, a.g * ambient, a.b * ambient, 1.0)
    ));

    // Panel texel: ambient albedo plus the emitted color.
    let (i, j) = f.lit_texels(id)[0];
    let panel = f.trace(&world, &[], ambient, black(), &front_ray(i, j));
    let a = albedo.texel(i, j).unwrap();
    let e = f.emissive_at(id, i, j);
    let expected = clamp_unit(Color::new(
        a.r * ambient + e.r,
        a.g * ambient + e.g,
        a.b * ambient + e.b,
        1.0,
    ));
    assert!(approx_color(panel, expected), "{panel:?} vs {expected:?}");
}

#[test]
fn the_lamp_and_the_portal_use_documented_emission_strengths() {
    let f = fixture();
    let lamp = f.library.get(redstone_lamp_material_id()).unwrap();
    let portal = f.library.get(portal_core_material_id()).unwrap();

    assert!((1.8..=3.0).contains(&lamp.emission_strength));
    assert!((2.5..=4.0).contains(&portal.emission_strength));
    assert!(lamp.emissive_texture.is_some());
    assert!(portal.emissive_texture.is_some());
}

#[test]
fn the_portal_core_uses_a_separate_emissive_mask() {
    let f = fixture();
    let id = portal_core_material_id();
    let portal = f.library.get(id).unwrap();

    let albedo_id = portal
        .face_textures
        .unwrap()
        .texture_for_face(Face::PositiveZ);
    assert_ne!(Some(albedo_id), portal.emissive_texture);

    // The mask lights only part of the membrane (the brighter swirls).
    let lit = f.lit_texels(id).len();
    assert!(lit > 20 && lit < N * N * 3 / 4, "lit texels = {lit}");

    // Dark wine albedo, corinto identity: red-dominant, never blue-heavy.
    let texture = f.manager.get(albedo_id).unwrap();
    for j in 0..N {
        for i in 0..N {
            let t = texture.texel(i, j).unwrap();
            assert!(t.r >= t.g && t.r >= t.b * 1.5, "texel ({i},{j}) = {t:?}");
        }
    }
}

#[test]
fn shadow_does_not_switch_off_a_texels_own_emission() {
    let f = fixture();
    let id = redstone_lamp_material_id();
    let (i, j) = f.lit_texels(id)[0];

    // A wall between the lamp's front face and a light shining from +Z.
    let mut world = lamp_world();
    world.insert(
        cell(0, 0, 3),
        BlockInstance::new(BlockType::Stone, MaterialId::new(WALL), Orientation::Up),
    );
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    // The eye looks from the side so the wall does not hide the lamp.
    let lx = (i as f32 + 0.5) / N as f32;
    let ly = 1.0 - (j as f32 + 0.5) / N as f32;
    let eye = ray((lx, ly, 2.0), (0.0, 0.0, -1.0));

    let shadowed = f.trace(&world, &lights, 0.0, black(), &eye);
    let expected = clamp_unit(f.emissive_at(id, i, j));
    assert!(
        approx_color(shadowed, expected),
        "{shadowed:?} vs {expected:?}"
    );
}

#[test]
fn portal_transparency_does_not_break_its_emission() {
    let f = fixture();
    let id = portal_core_material_id();
    let world = lone_block(BlockType::PortalCoreDarkCrimson, id, Orientation::South);
    let (i, j) = f.lit_texels(id)[0];
    let lx = (i as f32 + 0.5) / N as f32;
    let ly = 1.0 - (j as f32 + 0.5) / N as f32;
    let eye = ray((lx, ly, 9.0), (0.0, 0.0, -1.0));

    // Black background, no lights, no ambient: whatever shows is emission.
    let color = f.trace(&world, &[], 0.0, black(), &eye);
    // The membrane's tiny reflectivity mixes in the (black) reflected ray.
    let keep = 1.0 - f.library.get(id).unwrap().reflectivity;
    let e = clamp_unit(f.emissive_at(id, i, j));
    let expected = Color::new(e.r * keep, e.g * keep, e.b * keep, 1.0);

    assert!(!is_black(color));
    assert!(approx_color(color, expected), "{color:?} vs {expected:?}");
    // A dark spot of the mask stays dark: the membrane does not glow evenly.
    let dark = (0..N * N)
        .map(|n| (n % N, n / N))
        .find(|&(i, j)| is_black(f.emissive_at(id, i, j)))
        .unwrap();
    let ly = 1.0 - (dark.1 as f32 + 0.5) / N as f32;
    let lx = (dark.0 as f32 + 0.5) / N as f32;
    let dark_color = f.trace(
        &world,
        &[],
        0.0,
        black(),
        &ray((lx, ly, 9.0), (0.0, 0.0, -1.0)),
    );
    assert!(is_black(dark_color), "{dark_color:?}");
}

#[test]
fn the_portal_core_stays_a_thin_membrane() {
    let f = fixture();
    let id = portal_core_material_id();
    let world = lone_block(BlockType::PortalCoreDarkCrimson, id, Orientation::South);

    // A ray grazing the empty part of the cell (in front of the membrane's
    // thin slab) reaches the background instead of a full cube face.
    let miss = ray((0.5, 0.5, 0.2), (1.0, 0.0, 0.0));
    let color = f.trace(&world, &[], 0.0, Color::new(0.0, 0.0, 1.0, 1.0), &miss);
    assert!(
        approx_color(color, Color::new(0.0, 0.0, 1.0, 1.0)),
        "{color:?}"
    );
}

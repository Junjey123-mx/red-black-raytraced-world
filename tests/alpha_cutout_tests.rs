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
use core::material::{AlphaMode, CUTOUT_ALPHA_THRESHOLD, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{
    VoxelScene, is_voxel_occluded, light_visibility, nearest_visible_hit, nearest_voxel_hit,
    trace_ray,
};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::{GalleryTextures, advanced_materials_library, leaves_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 50.0;
const LEAVES: u32 = 18;
const RED: u32 = 2;
const N: usize = 16;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
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

fn block(block_type: BlockType, id: u32) -> BlockInstance {
    BlockInstance::new(block_type, MaterialId::new(id), Orientation::Up)
}

fn red() -> Color {
    Color::new(1.0, 0.0, 0.0, 1.0)
}

struct Fixture {
    manager: TextureManager,
    library: MaterialLibrary,
    /// `solid[y][x]`: whether the leaves texel is opaque.
    solid: Vec<Vec<bool>>,
    /// Color of each solid texel.
    color: Vec<Vec<Color>>,
}

fn fixture() -> Fixture {
    let mut manager = TextureManager::new();
    let textures = GalleryTextures::load(&mut manager, "assets/textures/overworld").unwrap();
    let mut library = advanced_materials_library(&textures);
    library.insert(MaterialId::new(RED), Material::matte(red()));

    let leaves = library.get(leaves_material_id()).unwrap();
    let id = leaves
        .face_textures
        .unwrap()
        .texture_for_face(core::hit::Face::PositiveZ);
    let texture = manager.get(id).unwrap();

    let mut solid = vec![vec![false; N]; N];
    let mut color = vec![vec![Color::black(); N]; N];
    for y in 0..N {
        for x in 0..N {
            let texel = texture.texel(x, y).unwrap();
            solid[y][x] = texel.a >= CUTOUT_ALPHA_THRESHOLD;
            color[y][x] = texel;
        }
    }

    Fixture {
        manager,
        library,
        solid,
        color,
    }
}

impl Fixture {
    /// Runs `f` with a `VoxelScene` over `world` (no lights, ambient = 1 so a
    /// hit shows exactly its albedo unless the test adds lights).
    fn scene<R>(
        &self,
        world: &VoxelWorld,
        lights: &[Light],
        ambient: f32,
        f: impl FnOnce(&VoxelScene) -> R,
    ) -> R {
        let scene = VoxelScene {
            world,
            materials: &self.library,
            camera_position: Vec3::new(0.5, 0.5, 9.0),
            lights,
            ambient_factor: ambient,
            background: Color::new(0.0, 0.0, 1.0, 1.0),
            texture_manager: &self.manager,
            max_distance: RANGE,
        };
        f(&scene)
    }
}

/// Center (in cell-local `[0,1]`) of texel `(i, j)` as seen on the +Z face:
/// `uv = (lx, 1 - ly)`.
fn front_local(i: usize, j: usize) -> (f32, f32) {
    (
        (i as f32 + 0.5) / N as f32,
        1.0 - (j as f32 + 0.5) / N as f32,
    )
}

fn leaves_cube_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::Leaves, LEAVES));
    world
}

/// A texel of the near (+Z) face that is opaque.
fn solid_front_texel(f: &Fixture) -> (usize, usize) {
    for j in 0..N {
        for i in 0..N {
            if f.solid[j][i] {
                return (i, j);
            }
        }
    }
    panic!("leaves texture has no opaque texel");
}

/// A texel that is empty on the near (+Z) face and on the far (-Z) face
/// (which is mirrored horizontally: `uv = (1 - lx, 1 - ly)`).
fn see_through_texel(f: &Fixture) -> (usize, usize) {
    for j in 0..N {
        for i in 0..N {
            if !f.solid[j][i] && !f.solid[j][N - 1 - i] {
                return (i, j);
            }
        }
    }
    panic!("leaves texture has no see-through column");
}

/// A texel empty on the near face but opaque on the far face.
fn hole_then_solid_texel(f: &Fixture) -> (usize, usize) {
    for j in 0..N {
        for i in 0..N {
            if !f.solid[j][i] && f.solid[j][N - 1 - i] {
                return (i, j);
            }
        }
    }
    panic!("leaves texture has no hole-then-solid column");
}

fn front_ray(i: usize, j: usize) -> Ray {
    let (lx, ly) = front_local(i, j);
    ray((lx, ly, 9.0), (0.0, 0.0, -1.0))
}

#[test]
fn an_opaque_texel_accepts_the_hit_and_shows_its_texture_color() {
    let f = fixture();
    let world = leaves_cube_world();
    let (i, j) = solid_front_texel(&f);
    let r = front_ray(i, j);

    let hit = f
        .scene(&world, &[], 1.0, |s| nearest_visible_hit(s, &r, RANGE))
        .expect("opaque texel must be hit");
    assert_eq!(hit.cell, cell(0, 0, 0));

    let color = f.scene(&world, &[], 1.0, |s| trace_ray(s, &r, 0));
    let expected = f.color[j][i];
    assert!(approx(color.r, expected.r) && approx(color.g, expected.g));
    assert!(approx(color.b, expected.b));
}

#[test]
fn an_empty_texel_rejects_the_hit() {
    let f = fixture();
    let world = leaves_cube_world();
    let (i, j) = see_through_texel(&f);
    let r = front_ray(i, j);

    // The raw geometric traversal still touches the cube...
    assert!(nearest_voxel_hit(&world, &r, RANGE).is_some());
    // ...but the visible-hit query discards it: air on both faces.
    assert!(
        f.scene(&world, &[], 1.0, |s| nearest_visible_hit(s, &r, RANGE))
            .is_none()
    );
}

#[test]
fn the_raycast_continues_behind_an_empty_texel() {
    let f = fixture();
    let mut world = leaves_cube_world();
    world.insert(cell(0, 0, -3), block(BlockType::Stone, RED));
    let (i, j) = see_through_texel(&f);
    let r = front_ray(i, j);

    let hit = f
        .scene(&world, &[], 1.0, |s| nearest_visible_hit(s, &r, RANGE))
        .expect("the block behind the hole must be reached");
    assert_eq!(hit.cell, cell(0, 0, -3));

    let color = f.scene(&world, &[], 1.0, |s| trace_ray(s, &r, 0));
    assert!(approx(color.r, 1.0) && approx(color.g, 0.0) && approx(color.b, 0.0));
}

#[test]
fn a_hole_in_the_near_face_can_reveal_the_far_faces_texel() {
    let f = fixture();
    let world = leaves_cube_world();
    let (i, j) = hole_then_solid_texel(&f);
    let r = front_ray(i, j);

    let hit = f
        .scene(&world, &[], 1.0, |s| nearest_visible_hit(s, &r, RANGE))
        .expect("the far face texel is solid");
    // Far (-Z) face at z = 0, seen from inside.
    assert!(approx(hit.hit.point.z, 0.0), "hit = {hit:?}");
    assert!(
        approx(hit.hit.distance, 9.0),
        "distance = {}",
        hit.hit.distance
    );
}

#[test]
fn the_uv_selects_the_matching_mask_texel() {
    let f = fixture();
    let world = leaves_cube_world();
    let (si, sj) = solid_front_texel(&f);
    let (hi, hj) = see_through_texel(&f);

    let solid_hit = f.scene(&world, &[], 1.0, |s| {
        nearest_visible_hit(s, &front_ray(si, sj), RANGE)
    });
    let hole_hit = f.scene(&world, &[], 1.0, |s| {
        nearest_visible_hit(s, &front_ray(hi, hj), RANGE)
    });

    let solid_hit = solid_hit.unwrap();
    let (lx, ly) = front_local(si, sj);
    assert!(approx(solid_hit.hit.uv.x, lx) && approx(solid_hit.hit.uv.y, 1.0 - ly));
    assert!(hole_hit.is_none());
}

/// Floor cell under a leaves cube, lit from straight above, so the shadow
/// ray from the floor goes up through the cube's bottom then top face.
fn shadow_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(BlockType::Stone, RED));
    world.insert(cell(0, 2, 0), block(BlockType::Leaves, LEAVES));
    world
}

/// Shadow ray from the floor top up through leaves cell (0,2,0): it crosses
/// the bottom (-Y) face at uv = (lx, 1 - lz) and the top (+Y) at (lx, lz).
fn shadow_ray_at(i: usize, j_top: usize) -> Ray {
    let lx = (i as f32 + 0.5) / N as f32;
    let lz = (j_top as f32 + 0.5) / N as f32;
    ray((lx, 1.0001, lz), (0.0, 1.0, 0.0))
}

#[test]
fn a_shadow_ray_passes_through_a_hole() {
    let f = fixture();
    let world = shadow_world();
    // Empty top texel (i, j) and empty bottom texel (i, N-1-j).
    let (i, j) = (0..N)
        .flat_map(|j| (0..N).map(move |i| (i, j)))
        .find(|&(i, j)| !f.solid[j][i] && !f.solid[N - 1 - j][i])
        .expect("a see-through shadow column exists");
    let r = shadow_ray_at(i, j);

    assert!(is_voxel_occluded(&world, &r, 10.0), "geometry alone blocks");
    let visibility = f.scene(&world, &[], 1.0, |s| light_visibility(s, &r, 10.0));
    assert!(approx(visibility, 1.0), "visibility = {visibility}");
}

#[test]
fn a_shadow_ray_is_blocked_by_an_opaque_texel() {
    let f = fixture();
    let world = shadow_world();
    // Empty top texel but opaque bottom texel, and the reverse.
    let (i, j) = (0..N)
        .flat_map(|j| (0..N).map(move |i| (i, j)))
        .find(|&(i, j)| f.solid[N - 1 - j][i])
        .expect("an opaque bottom texel exists");
    let r = shadow_ray_at(i, j);

    let visibility = f.scene(&world, &[], 1.0, |s| light_visibility(s, &r, 10.0));
    assert!(approx(visibility, 0.0), "visibility = {visibility}");
}

#[test]
fn a_leaf_casts_a_dappled_shadow_in_the_rendered_image() {
    let f = fixture();
    let world = shadow_world();
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Color::white(),
        1.0,
    ))];

    // Sample the floor top under the cube on a 16x16 grid: some points lit,
    // some in shadow, none corrupted.
    let mut lit = 0;
    let mut shadowed = 0;
    for j in 0..N {
        for i in 0..N {
            let x = (i as f32 + 0.5) / N as f32;
            let z = (j as f32 + 0.5) / N as f32;
            let color = f.scene(&world, &lights, 0.1, |s| {
                trace_ray(s, &ray((x, 6.0, z), (0.0, -1.0, 0.0)), 0)
            });
            assert!(color.r.is_finite());
            // The eye ray passes through the leaves too; only count floor hits.
            if color.r > 0.5 {
                lit += 1;
            } else {
                shadowed += 1;
            }
        }
    }
    assert!(lit > 0 && shadowed > 0, "lit {lit}, shadowed {shadowed}");
}

#[test]
fn a_back_face_seen_through_a_hole_is_lit_from_the_viewers_side() {
    let f = fixture();
    let world = leaves_cube_world();
    let (i, j) = hole_then_solid_texel(&f);
    let r = front_ray(i, j);
    // Light toward +Z (the viewer's side); no ambient at all.
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];

    let color = f.scene(&world, &lights, 0.0, |s| trace_ray(s, &r, 0));
    assert!(color.g > 0.0, "back face must be lit, got {color:?}");
}

#[test]
fn leaves_do_not_refract_or_blend_like_glass() {
    let f = fixture();
    let leaves = f.library.get(leaves_material_id()).unwrap();

    assert_eq!(leaves.alpha_mode, AlphaMode::Cutout);
    assert_eq!(leaves.transparency, 0.0);
    assert_eq!(leaves.reflectivity, 0.0);
    // Cutout never becomes continuous transmission, whatever the texel alpha.
    assert_eq!(leaves.effective_transparency(0.0), 0.0);
    assert_eq!(leaves.effective_transparency(1.0), 0.0);

    // A solid texel is fully opaque: no blending with what is behind it.
    let mut world = leaves_cube_world();
    world.insert(cell(0, 0, -3), block(BlockType::Stone, RED));
    let (i, j) = solid_front_texel(&f);
    let color = f.scene(&world, &[], 1.0, |s| trace_ray(s, &front_ray(i, j), 0));
    let expected = f.color[j][i];
    assert!(approx(color.r, expected.r) && approx(color.g, expected.g));
}

#[test]
fn only_cutout_materials_discard_texels() {
    let opaque = Material::matte(red());
    let blend = Material::matte(red()).with_alpha_mode(AlphaMode::Blend);
    let cutout = Material::matte(red()).with_alpha_mode(AlphaMode::Cutout);

    assert!(!opaque.is_cut_out(0.0));
    assert!(!blend.is_cut_out(0.0));
    assert!(cutout.is_cut_out(0.0));
    assert!(cutout.is_cut_out(CUTOUT_ALPHA_THRESHOLD - 0.01));
    assert!(!cutout.is_cut_out(CUTOUT_ALPHA_THRESHOLD));
    assert!(!cutout.is_cut_out(1.0));
}

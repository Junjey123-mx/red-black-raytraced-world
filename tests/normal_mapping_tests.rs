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
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::normal_mapping::{
    decode_normal, perturb_normal, sample_shading_normal, tangent_basis,
};
use renderer::raytracer::{VoxelScene, nearest_voxel_hit, trace_ray};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::{
    GalleryTextures, advanced_materials_library, deepslate_bricks_material_id,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 50.0;
const N: usize = 16;
const PLAIN: u32 = 91;

const FACES: [Face; 6] = [
    Face::PositiveX,
    Face::NegativeX,
    Face::PositiveY,
    Face::NegativeY,
    Face::PositiveZ,
    Face::NegativeZ,
];

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

fn approx_vec(a: Vec3, b: Vec3) -> bool {
    approx(a.x, b.x) && approx(a.y, b.y) && approx(a.z, b.z)
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

/// `n` as a normal-map texel: `rgb = n * 0.5 + 0.5`.
fn texel_of(n: Vec3) -> Color {
    Color::new(n.x * 0.5 + 0.5, n.y * 0.5 + 0.5, n.z * 0.5 + 0.5, 1.0)
}

#[test]
fn a_flat_normal_map_keeps_the_geometric_normal_on_every_face() {
    let flat = decode_normal(texel_of(Vec3::new(0.0, 0.0, 1.0)));
    for face in FACES {
        let shading = perturb_normal(face.normal(), face, flat);
        assert!(approx_vec(shading, face.normal()), "{face:?}: {shading:?}");
    }
}

#[test]
fn a_positive_x_tangent_perturbation_tilts_the_normal_toward_the_tangent() {
    let tilted = decode_normal(texel_of(Vec3::new(0.6, 0.0, 0.8)));

    for face in FACES {
        let basis = tangent_basis(face);
        let shading = perturb_normal(face.normal(), face, tilted);
        let expected = (basis.tangent * 0.6 + basis.normal * 0.8).normalize();

        assert!(approx_vec(shading, expected), "{face:?}: {shading:?}");
        assert!(shading.dot(basis.tangent) > 0.5, "{face:?} must lean +T");
        assert!(shading != face.normal());
    }
}

#[test]
fn a_positive_y_tangent_perturbation_tilts_the_normal_toward_the_bitangent() {
    let tilted = decode_normal(texel_of(Vec3::new(0.0, 0.6, 0.8)));

    for face in FACES {
        let basis = tangent_basis(face);
        let shading = perturb_normal(face.normal(), face, tilted);
        let expected = (basis.bitangent * 0.6 + basis.normal * 0.8).normalize();

        assert!(approx_vec(shading, expected), "{face:?}: {shading:?}");
        assert!(shading.dot(basis.bitangent) > 0.5, "{face:?} must lean +B");
    }
}

#[test]
fn the_perturbed_normal_is_always_finite_and_unit_length() {
    let texels = [
        Color::new(0.0, 0.0, 0.0, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
        Color::new(0.5, 0.5, 0.5, 1.0),
        Color::new(1.0, 0.5, 0.5, 1.0),
        Color::new(0.1, 0.9, 0.55, 1.0),
        Color::new(0.5, 0.5, 1.0, 1.0),
    ];

    for face in FACES {
        for texel in texels {
            let decoded = decode_normal(texel);
            assert!((decoded.length() - 1.0).abs() < 1e-4, "{texel:?}");
            assert!(decoded.z >= 0.0, "never into the surface");

            let shading = perturb_normal(face.normal(), face, decoded);
            assert!(shading.x.is_finite() && shading.y.is_finite() && shading.z.is_finite());
            assert!((shading.length() - 1.0).abs() < 1e-4, "{face:?} {texel:?}");
            assert!(shading.dot(face.normal()) >= -EPS, "faces outward");
        }
    }
}

#[test]
fn every_face_has_an_orthonormal_right_handed_frame() {
    for face in FACES {
        let b = tangent_basis(face);
        assert!(approx(b.tangent.length(), 1.0) && approx(b.bitangent.length(), 1.0));
        assert!(approx(b.tangent.dot(b.bitangent), 0.0));
        assert!(approx(b.tangent.dot(b.normal), 0.0));
        assert!(approx(b.bitangent.dot(b.normal), 0.0));
        assert!(
            approx_vec(b.tangent.cross(b.bitangent), b.normal),
            "{face:?}"
        );
        assert_eq!(b.normal, face.normal());
    }
}

#[test]
fn the_tangent_frame_follows_the_uv_direction_of_each_face() {
    // Walking +u along a face must move the hit point along `tangent`, and
    // walking +v must move it along `-bitangent` (v grows downward).
    let world = {
        let mut w = VoxelWorld::new();
        w.insert(
            cell(0, 0, 0),
            BlockInstance::new(BlockType::Stone, MaterialId::new(1), Orientation::Up),
        );
        w
    };
    let eyes = [
        (Face::PositiveZ, (0.5, 0.5, 5.0), (0.0, 0.0, -1.0)),
        (Face::NegativeZ, (0.5, 0.5, -5.0), (0.0, 0.0, 1.0)),
        (Face::PositiveX, (5.0, 0.5, 0.5), (-1.0, 0.0, 0.0)),
        (Face::NegativeX, (-5.0, 0.5, 0.5), (1.0, 0.0, 0.0)),
        (Face::PositiveY, (0.5, 5.0, 0.5), (0.0, -1.0, 0.0)),
        (Face::NegativeY, (0.5, -5.0, 0.5), (0.0, 1.0, 0.0)),
    ];

    for (face, origin, direction) in eyes {
        let basis = tangent_basis(face);
        let hit_at = |shift: Vec3| {
            let r = ray(
                (origin.0 + shift.x, origin.1 + shift.y, origin.2 + shift.z),
                direction,
            );
            nearest_voxel_hit(&world, &r, RANGE).expect("must hit").hit
        };

        let base = hit_at(Vec3::zero());
        assert_eq!(base.face, face);
        let step = 0.2;
        let moved_u = hit_at(basis.tangent * step);
        let moved_v = hit_at(basis.bitangent * -step);

        assert!(
            moved_u.uv.x > base.uv.x + 0.1 && approx(moved_u.uv.y, base.uv.y),
            "{face:?} u"
        );
        assert!(
            moved_v.uv.y > base.uv.y + 0.1 && approx(moved_v.uv.x, base.uv.x),
            "{face:?} v"
        );
    }
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

    // The same brick albedo without the normal map, for A/B comparisons.
    let mapped = library.get(deepslate_bricks_material_id()).unwrap().clone();
    let mut plain = mapped.clone();
    plain.normal_texture = None;
    library.insert(MaterialId::new(PLAIN), plain);

    Fixture { manager, library }
}

impl Fixture {
    fn scene_color(&self, world: &VoxelWorld, lights: &[Light], r: &Ray) -> Color {
        let scene = VoxelScene {
            world,
            materials: &self.library,
            camera_position: r.origin,
            lights,
            ambient_factor: 0.1,
            background: Color::new(0.0, 0.0, 0.0, 1.0),
            texture_manager: &self.manager,
            max_distance: RANGE,
        };
        trace_ray(&scene, r, 0)
    }
}

fn brick_world(id: MaterialId) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        BlockInstance::new(BlockType::DeepslateBricks, id, Orientation::Up),
    );
    world
}

fn front_ray(i: usize, j: usize) -> Ray {
    let lx = (i as f32 + 0.5) / N as f32;
    let ly = 1.0 - (j as f32 + 0.5) / N as f32;
    ray((lx, ly, 6.0), (0.0, 0.0, -1.0))
}

fn grazing_light() -> [Light; 1] {
    // From the upper right, well off the surface normal, so relief shows.
    [Light::Directional(DirectionalLight::new(
        Vec3::new(0.7, 0.5, 0.5),
        Color::white(),
        1.0,
    ))]
}

#[test]
fn the_normal_texture_is_sampled_with_the_same_uv_as_the_albedo() {
    let f = fixture();
    let material = f.library.get(deepslate_bricks_material_id()).unwrap();
    let normal_texture = f.manager.get(material.normal_texture.unwrap()).unwrap();

    for j in 0..N {
        for i in 0..N {
            let uv = Vec2::new((i as f32 + 0.5) / N as f32, (j as f32 + 0.5) / N as f32);
            let shading = sample_shading_normal(
                material,
                Face::PositiveZ,
                uv,
                Face::PositiveZ.normal(),
                &f.manager,
            );
            let expected = perturb_normal(
                Face::PositiveZ.normal(),
                Face::PositiveZ,
                decode_normal(normal_texture.texel(i, j).unwrap()),
            );
            assert!(approx_vec(shading, expected), "texel ({i},{j})");
        }
    }
}

#[test]
fn a_material_without_a_normal_texture_shades_with_the_geometric_normal() {
    let f = fixture();
    let plain = f.library.get(MaterialId::new(PLAIN)).unwrap();
    let uv = Vec2::new(0.3, 0.6);

    for face in FACES {
        assert_eq!(
            sample_shading_normal(plain, face, uv, face.normal(), &f.manager),
            face.normal()
        );
    }
}

#[test]
fn the_brick_normal_map_actually_relieves_the_surface() {
    let f = fixture();
    let material = f.library.get(deepslate_bricks_material_id()).unwrap();

    let tilted = (0..N * N)
        .filter(|n| {
            let uv = Vec2::new(((n % N) as f32 + 0.5) / 16.0, ((n / N) as f32 + 0.5) / 16.0);
            let s = sample_shading_normal(
                material,
                Face::PositiveZ,
                uv,
                Face::PositiveZ.normal(),
                &f.manager,
            );
            s.z < 0.97
        })
        .count();

    assert!(tilted > 30, "only {tilted} texels are perturbed");
    assert!(tilted < N * N, "some texels must stay (almost) flat");
}

#[test]
fn diffuse_and_specular_respond_to_the_normal_map() {
    let f = fixture();
    let mapped = brick_world(deepslate_bricks_material_id());
    let plain = brick_world(MaterialId::new(PLAIN));
    let lights = grazing_light();

    let mut differing = 0;
    let mut identical_where_flat = 0;
    for j in 0..N {
        for i in 0..N {
            let a = f.scene_color(&mapped, &lights, &front_ray(i, j));
            let b = f.scene_color(&plain, &lights, &front_ray(i, j));
            assert!(a.r.is_finite() && a.g.is_finite() && a.b.is_finite());
            if (a.r - b.r).abs() > 0.01 || (a.g - b.g).abs() > 0.01 {
                differing += 1;
            } else {
                identical_where_flat += 1;
            }
        }
    }

    assert!(
        differing > 30,
        "the relief must change the shading ({differing})"
    );
    assert!(
        identical_where_flat > 0,
        "flat texels must shade like the plain block"
    );
}

#[test]
fn normal_mapping_never_changes_the_silhouette_or_the_geometric_hit() {
    let mapped = brick_world(deepslate_bricks_material_id());
    let plain = brick_world(MaterialId::new(PLAIN));

    for j in 0..N {
        for i in 0..N {
            let r = front_ray(i, j);
            let a = nearest_voxel_hit(&mapped, &r, RANGE).unwrap();
            let b = nearest_voxel_hit(&plain, &r, RANGE).unwrap();

            // The traversal never sees materials at all, and the recorded
            // normal stays the exact axis-aligned face normal.
            assert_eq!(a.hit, b.hit);
            assert_eq!(a.hit.normal, Face::PositiveZ.normal());
        }
    }

    // Rays just outside the block still miss it: relief adds no geometry.
    let f = fixture();
    let miss = f.scene_color(
        &mapped,
        &grazing_light(),
        &ray((1.02, 0.5, 6.0), (0.0, 0.0, -1.0)),
    );
    assert_eq!(miss, Color::new(0.0, 0.0, 0.0, 1.0));
}

#[test]
fn a_perturbed_normal_cannot_light_a_surface_facing_away_from_the_light() {
    let f = fixture();
    let world = brick_world(deepslate_bricks_material_id());
    // The light is behind the +Z face: the face itself must stay in shadow
    // (ambient only) whatever the relief says.
    let behind = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, -1.0),
        Color::white(),
        1.0,
    ))];

    for j in 0..N {
        for i in 0..N {
            let lit = f.scene_color(&world, &behind, &front_ray(i, j));
            let ambient_only = f.scene_color(&world, &[], &front_ray(i, j));
            assert!((lit.r - ambient_only.r).abs() < EPS, "texel ({i},{j})");
        }
    }
}

#[test]
fn reflection_and_refraction_keep_using_the_geometric_normal() {
    // A reflective, normal-mapped brick block still reflects like a flat
    // mirror: the reflected ray direction depends only on the geometric
    // normal, so the mirrored scene color is identical across texels.
    let mut f = fixture();
    let id = deepslate_bricks_material_id();
    let mirror = f.library.get(id).unwrap().clone().with_reflectivity(1.0);
    f.library.insert(id, mirror);

    let mut world = brick_world(id);
    world.insert(
        cell(0, 0, 8),
        BlockInstance::new(
            BlockType::Stone,
            MaterialId::new(PLAIN + 1),
            Orientation::Up,
        ),
    );
    f.library.insert(
        MaterialId::new(PLAIN + 1),
        Material::matte(Color::new(1.0, 0.0, 0.0, 1.0)),
    );

    let first = f.scene_color(&world, &[], &front_ray(3, 3));
    let second = f.scene_color(&world, &[], &front_ray(11, 12));
    assert_eq!(first, second, "every texel mirrors the same red block");
    assert!(first.r > 0.09 && first.g < 0.01, "first = {first:?}");
}

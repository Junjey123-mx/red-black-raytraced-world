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
use core::hit::Face;
use core::material::MaterialId;
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, cast_ray_voxel};
use renderer::skybox::Background;
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::catalog::catalog_entries;
use scene::geometry_orientation::{functional_face, functional_face_uv, functional_uv};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::{log_material_id, mycelium_material_id, smooth_basalt_material_id};
use scene::scene::grass_material_id;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;
use scene::world::{WorldTextures, world_materials};

const ALL_FACES: [Face; 6] = [
    Face::PositiveX,
    Face::NegativeX,
    Face::PositiveY,
    Face::NegativeY,
    Face::PositiveZ,
    Face::NegativeZ,
];

struct Lab {
    manager: TextureManager,
    materials: MaterialLibrary,
}

impl Lab {
    fn new() -> Self {
        let mut manager = TextureManager::new();
        let textures = WorldTextures::load(
            &mut manager,
            "assets/textures/overworld",
            "assets/textures/overworld/grass",
            "assets/textures/portal",
        )
        .unwrap();
        let materials = world_materials(&textures);
        Self { manager, materials }
    }

    /// Albedo seen on one block (at the origin cell) placed with
    /// `orientation`, under ambient-only light: the sampled texel itself.
    fn sample(
        &self,
        block_type: BlockType,
        material: MaterialId,
        orientation: Orientation,
        origin: Vec3,
        direction: Vec3,
    ) -> Color {
        let mut world = VoxelWorld::new();
        world.insert(
            IVec3::zero(),
            BlockInstance::new(block_type, material, orientation),
        );
        let scene = VoxelScene {
            world: &world,
            materials: &self.materials,
            camera_position: origin,
            lights: &[],
            ambient_factor: 1.0,
            background: Background::Solid(Color::new(0.0, 1.0, 1.0, 1.0)),
            texture_manager: &self.manager,
            max_distance: 20.0,
        };
        cast_ray_voxel(&scene, &Ray::new(origin, direction))
    }
}

fn from_above(lx: f32, lz: f32) -> (Vec3, Vec3) {
    (Vec3::new(lx, 4.0, lz), Vec3::new(0.0, -1.0, 0.0))
}

fn from_below(lx: f32, lz: f32) -> (Vec3, Vec3) {
    (Vec3::new(lx, -4.0, lz), Vec3::new(0.0, 1.0, 0.0))
}

fn from_south(lx: f32, ly: f32) -> (Vec3, Vec3) {
    (Vec3::new(lx, ly, 4.0), Vec3::new(0.0, 0.0, -1.0))
}

fn close(a: Color, b: Color) -> bool {
    (a.r - b.r).abs() < 1e-4 && (a.g - b.g).abs() < 1e-4 && (a.b - b.b).abs() < 1e-4
}

#[test]
fn up_and_south_keep_every_face_and_uv() {
    for face in ALL_FACES {
        for (u, v) in [(0.0, 0.0), (0.25, 0.75), (1.0, 0.5)] {
            let uv = Vec2::new(u, v);
            assert_eq!(functional_face_uv(face, uv, Orientation::Up), (face, uv));
            assert_eq!(functional_face_uv(face, uv, Orientation::South), (face, uv));
        }
    }
}

#[test]
fn down_moves_the_top_to_negative_y_and_flips_v() {
    assert_eq!(
        functional_face(Face::PositiveY, Orientation::Down),
        Face::NegativeY
    );
    assert_eq!(
        functional_face(Face::NegativeY, Orientation::Down),
        Face::PositiveY
    );
    for face in [
        Face::PositiveX,
        Face::NegativeX,
        Face::PositiveZ,
        Face::NegativeZ,
    ] {
        assert_eq!(functional_face(face, Orientation::Down), face);
    }
    for face in ALL_FACES {
        let uv = functional_uv(face, Vec2::new(0.3, 0.2), Orientation::Down);
        assert!((uv.x - 0.3).abs() < 1e-6 && (uv.y - 0.8).abs() < 1e-6);
    }
}

#[test]
fn cardinal_rotations_permute_the_lateral_faces_and_keep_uv_finite() {
    use Face::*;
    assert_eq!(functional_face(PositiveX, Orientation::North), NegativeX);
    assert_eq!(functional_face(PositiveZ, Orientation::North), NegativeZ);
    assert_eq!(functional_face(PositiveX, Orientation::East), PositiveZ);
    assert_eq!(functional_face(PositiveZ, Orientation::East), NegativeX);
    assert_eq!(functional_face(PositiveX, Orientation::West), NegativeZ);
    assert_eq!(functional_face(PositiveZ, Orientation::West), PositiveX);
    for orientation in [Orientation::North, Orientation::East, Orientation::West] {
        assert_eq!(functional_face(PositiveY, orientation), PositiveY);
        assert_eq!(functional_face(NegativeY, orientation), NegativeY);
        // A bijection on the six faces.
        let mut images: Vec<Face> = ALL_FACES
            .iter()
            .map(|f| functional_face(*f, orientation))
            .collect();
        images.sort_by_key(|f| *f as u8);
        let mut all = ALL_FACES.to_vec();
        all.sort_by_key(|f| *f as u8);
        assert_eq!(images, all);
        for face in ALL_FACES {
            let uv = functional_uv(face, Vec2::new(0.9, 0.1), orientation);
            assert!(uv.x.is_finite() && uv.y.is_finite());
            assert!((0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y));
        }
    }
    // East turns the top texture a quarter turn; West the other way.
    let e = functional_uv(PositiveY, Vec2::new(1.0, 0.0), Orientation::East);
    let w = functional_uv(PositiveY, Vec2::new(1.0, 0.0), Orientation::West);
    assert_ne!(e, w);
}

#[test]
fn inverted_mycelium_shows_its_top_on_the_underside() {
    let lab = Lab::new();
    let (o, d) = from_above(0.5, 0.5);
    let top_up = lab.sample(
        BlockType::Mycelium,
        mycelium_material_id(),
        Orientation::Up,
        o,
        d,
    );
    let (o, d) = from_below(0.5, 0.5);
    let bottom_up = lab.sample(
        BlockType::Mycelium,
        mycelium_material_id(),
        Orientation::Up,
        o,
        d,
    );
    let bottom_down = lab.sample(
        BlockType::Mycelium,
        mycelium_material_id(),
        Orientation::Down,
        o,
        d,
    );
    let (o, d) = from_above(0.5, 0.5);
    let top_down = lab.sample(
        BlockType::Mycelium,
        mycelium_material_id(),
        Orientation::Down,
        o,
        d,
    );
    assert!(
        !close(top_up, bottom_up),
        "the mycelium top and its dirt bottom must differ"
    );
    assert!(
        close(bottom_down, top_up),
        "Down: -Y must show the mycelium top"
    );
    assert!(
        close(top_down, bottom_up),
        "Down: +Y must show the dirt bottom"
    );
}

#[test]
fn inverted_log_and_basalt_keep_their_ends_and_flip_their_sides() {
    let lab = Lab::new();
    for (block, material) in [
        (BlockType::Log, log_material_id()),
        (BlockType::SmoothBasalt, smooth_basalt_material_id()),
    ] {
        let (o, d) = from_above(0.3, 0.7);
        let top_up = lab.sample(block, material, Orientation::Up, o, d);
        let (o, d) = from_below(0.3, 0.7);
        let bottom_down = lab.sample(block, material, Orientation::Down, o, d);
        assert!(
            close(top_up, bottom_down),
            "{block:?}: the end texture must stay on the ends"
        );
        // Sides: the row near the bottom of the inverted block is the row
        // near the top of the upright one.
        let (o, d) = from_south(0.4, 0.9);
        let side_up_high = lab.sample(block, material, Orientation::Up, o, d);
        let (o, d) = from_south(0.4, 0.1);
        let side_down_low = lab.sample(block, material, Orientation::Down, o, d);
        assert!(
            close(side_up_high, side_down_low),
            "{block:?}: side rows must flip"
        );
    }
}

#[test]
fn inverted_grass_hangs_its_green_band_downward() {
    let lab = Lab::new();
    let (o, d) = from_south(0.5, 0.95);
    let up_top_row = lab.sample(BlockType::Grass, grass_material_id(), Orientation::Up, o, d);
    let (o, d) = from_south(0.5, 0.05);
    let up_bottom_row = lab.sample(BlockType::Grass, grass_material_id(), Orientation::Up, o, d);
    let down_bottom_row = lab.sample(
        BlockType::Grass,
        grass_material_id(),
        Orientation::Down,
        o,
        d,
    );
    assert!(
        !close(up_top_row, up_bottom_row),
        "grass sides have a green band on top"
    );
    assert!(close(down_bottom_row, up_top_row));
    // And the grass top itself faces down.
    let (o, d) = from_below(0.5, 0.5);
    let down_underside = lab.sample(
        BlockType::Grass,
        grass_material_id(),
        Orientation::Down,
        o,
        d,
    );
    let (o, d) = from_above(0.5, 0.5);
    let up_top = lab.sample(BlockType::Grass, grass_material_id(), Orientation::Up, o, d);
    assert!(close(down_underside, up_top));
}

#[test]
fn the_catalog_only_uses_identity_orientations_so_it_cannot_change() {
    for entry in catalog_entries() {
        assert!(
            matches!(entry.orientation, Orientation::Up | Orientation::South),
            "{}",
            entry.display_name
        );
        for (_, block) in &entry.extra_blocks {
            assert!(matches!(
                block.orientation(),
                Orientation::Up | Orientation::South
            ));
        }
    }
}

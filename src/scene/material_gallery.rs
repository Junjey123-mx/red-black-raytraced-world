// Diagnostic-scene stage: the isolated "optical gallery" that demonstrates the
// Gate 07 advanced material properties one specimen at a time. This is
// deliberately NOT the final Overworld or any final block art: only the
// materials needed to show each optical behavior exist, on a plain checker
// floor with a few backdrop cubes that make refraction legible.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::material::{AlphaMode, Material, MaterialId};
use crate::core::math::{IVec3, Vec3};
use crate::core::texture::TextureId;
use crate::scene::block::BlockInstance;
use crate::scene::block_type::BlockType;
use crate::scene::light::{DirectionalLight, Light, PointLight};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::orientation::Orientation;
use crate::scene::texture_manager::{TextureLoadError, TextureManager};
use crate::scene::voxel_world::VoxelWorld;

pub fn checker_light_material_id() -> MaterialId {
    MaterialId::new(10)
}

pub fn checker_dark_material_id() -> MaterialId {
    MaterialId::new(11)
}

pub fn matte_material_id() -> MaterialId {
    MaterialId::new(12)
}

pub fn mirror_material_id() -> MaterialId {
    MaterialId::new(13)
}

pub fn glass_material_id() -> MaterialId {
    MaterialId::new(14)
}

pub fn water_material_id() -> MaterialId {
    MaterialId::new(15)
}

pub fn backdrop_red_material_id() -> MaterialId {
    MaterialId::new(16)
}

pub fn backdrop_blue_material_id() -> MaterialId {
    MaterialId::new(17)
}

pub fn leaves_material_id() -> MaterialId {
    MaterialId::new(18)
}

pub fn redstone_lamp_material_id() -> MaterialId {
    MaterialId::new(19)
}

/// Floor of the gallery: `x` in `0..GALLERY_WIDTH`, `z` in `0..GALLERY_DEPTH`.
pub const GALLERY_WIDTH: i32 = 12;
pub const GALLERY_DEPTH: i32 = 7;

/// Front row (closest to the camera) of specimens, at `y = 1`.
const FRONT_Z: i32 = 5;
/// Backdrop cubes sit just behind the transmissive specimens.
const BACKDROP_Z: i32 = 3;

/// Back row of specimens (`z = 1`), with a red cube right behind the leaves
/// so the holes of the cut-out texture visibly show what lies beyond.
const BACK_Z: i32 = 1;

pub const LEAVES_X: i32 = 1;
pub const MATTE_X: i32 = 1;
pub const MIRROR_X: i32 = 4;
pub const GLASS_X: i32 = 7;
pub const WATER_X: i32 = 10;

/// Builds the optical gallery as a sparse `VoxelWorld`: a two-tone checker
/// floor, and one specimen per optical behavior. Front row (`z = 5`):
///
/// ```text
/// Matte control x=1   Mirror x=4   Glass x=7   Water x=10
/// ```
///
/// Back row (`z = 1`): a Leaves cube (alpha cutout) at `x=1`, with a red cube
/// directly behind it.
///
/// A red and a blue cube stand behind the glass and the water so the
/// refracted view of them is unmistakable.
pub fn advanced_materials_world() -> VoxelWorld {
    let mut world = VoxelWorld::new();

    for z in 0..GALLERY_DEPTH {
        for x in 0..GALLERY_WIDTH {
            let id = if (x + z) % 2 == 0 {
                checker_light_material_id()
            } else {
                checker_dark_material_id()
            };
            world.insert(
                IVec3::new(x, 0, z),
                BlockInstance::new(BlockType::Stone, id, Orientation::Up),
            );
        }
    }

    let mut place = |x: i32, z: i32, block_type: BlockType, id: MaterialId| {
        world.insert(
            IVec3::new(x, 1, z),
            BlockInstance::new(block_type, id, Orientation::Up),
        );
    };

    place(MATTE_X, FRONT_Z, BlockType::Stone, matte_material_id());
    place(MIRROR_X, FRONT_Z, BlockType::Stone, mirror_material_id());
    place(GLASS_X, FRONT_Z, BlockType::Glass, glass_material_id());
    place(WATER_X, FRONT_Z, BlockType::Water, water_material_id());

    place(LEAVES_X, BACK_Z, BlockType::Leaves, leaves_material_id());
    place(
        LEAVES_X,
        BACK_Z - 1,
        BlockType::Stone,
        backdrop_red_material_id(),
    );

    place(
        GLASS_X,
        BACKDROP_Z,
        BlockType::Stone,
        backdrop_red_material_id(),
    );
    place(
        WATER_X,
        BACKDROP_Z,
        BlockType::Stone,
        backdrop_blue_material_id(),
    );

    world
}

/// Face textures of the gallery specimens that use real pixel-art assets.
pub struct GalleryTextures {
    pub glass: FaceTextures,
    pub water: FaceTextures,
    pub leaves: FaceTextures,
    pub redstone_lamp: FaceTextures,
    pub redstone_lamp_emissive: TextureId,
}

impl GalleryTextures {
    /// Loads the gallery PNGs from `overworld_dir` (`glass.png`, `water.png`)
    /// once, through the shared `TextureManager`.
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let glass = manager.load(format!("{overworld_dir}/glass.png"))?;
        let water = manager.load(format!("{overworld_dir}/water.png"))?;
        let leaves = manager.load(format!("{overworld_dir}/leaves.png"))?;
        let lamp_albedo = manager.load(format!("{overworld_dir}/redstone_lamp/albedo.png"))?;
        let lamp_emissive = manager.load(format!("{overworld_dir}/redstone_lamp/emissive.png"))?;

        Ok(Self {
            glass: FaceTextures::uniform(glass),
            water: FaceTextures::uniform(water),
            leaves: FaceTextures::uniform(leaves),
            redstone_lamp: FaceTextures::uniform(lamp_albedo),
            redstone_lamp_emissive: lamp_emissive,
        })
    }
}

/// Materials of the optical gallery.
///
/// - Glass: high transparency, IOR 1.5, strong specular, low reflectivity.
/// - Water: lower transparency, IOR 1.33, strong specular, partial reflection.
/// - Mirror: a diagnostic polished control (not a catalog block) with a
///   deliberately strong `reflectivity` so the bounce reads at a glance.
/// - Leaves: alpha cutout foliage — opaque or air per texel, never refractive.
/// - Redstone Lamp (lit): albedo plus a separate emissive mask, so only the
///   amber panels are self-luminous (`emission_strength` 1.8, the low end of
///   the documented 1.8-3.0 range); the dark-brown frame does not emit.
/// - Everything else is a plain matte control.
pub fn advanced_materials_library(textures: &GalleryTextures) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    let matte = |r: f32, g: f32, b: f32| Material::matte(Color::new(r, g, b, 1.0));

    library.insert(checker_light_material_id(), matte(0.62, 0.58, 0.48));
    library.insert(checker_dark_material_id(), matte(0.20, 0.21, 0.26));
    library.insert(matte_material_id(), matte(0.55, 0.55, 0.58));
    library.insert(backdrop_red_material_id(), matte(0.85, 0.12, 0.10));
    library.insert(backdrop_blue_material_id(), matte(0.10, 0.25, 0.85));

    library.insert(
        mirror_material_id(),
        Material::new(Color::new(0.75, 0.78, 0.80, 1.0), 0.5, 60.0).with_reflectivity(0.65),
    );
    library.insert(
        leaves_material_id(),
        Material::new(Color::new(0.30, 0.45, 0.20, 1.0), 0.03, 4.0)
            .with_face_textures(textures.leaves)
            .with_alpha_mode(AlphaMode::Cutout),
    );
    library.insert(
        redstone_lamp_material_id(),
        Material::new(Color::new(0.45, 0.35, 0.25, 1.0), 0.10, 20.0)
            .with_face_textures(textures.redstone_lamp)
            .with_emissive_texture(textures.redstone_lamp_emissive)
            .with_emission_strength(1.8),
    );
    library.insert(
        glass_material_id(),
        Material::new(Color::new(0.92, 0.97, 1.0, 1.0), 0.75, 140.0)
            .with_face_textures(textures.glass)
            .with_alpha_mode(AlphaMode::Blend)
            .with_reflectivity(0.10)
            .with_transparency(0.92)
            .with_refractive_index(1.5),
    );
    library.insert(
        water_material_id(),
        Material::new(Color::new(0.20, 0.38, 0.75, 1.0), 0.55, 80.0)
            .with_face_textures(textures.water)
            .with_reflectivity(0.18)
            .with_transparency(0.78)
            .with_refractive_index(1.33),
    );

    library
}

/// Scene range for primary rays and directional shadow rays.
pub const GALLERY_MAX_DISTANCE: f32 = 40.0;

/// Sky/background color of the gallery.
pub fn gallery_background() -> Color {
    Color::new(0.05, 0.05, 0.08, 1.0)
}

/// Camera framing the whole gallery from the front, slightly elevated.
pub fn gallery_camera(aspect_ratio: f32) -> Camera {
    Camera::new(
        Vec3::new(6.0, 5.6, 14.0),
        Vec3::new(6.0, 0.6, 2.6),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    )
}

/// A soft overhead directional fill plus a stronger point light above the
/// front row, casting hard shadows and driving the specular highlights.
pub fn gallery_lights() -> Vec<Light> {
    vec![
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.0, 1.0, 0.0),
            Color::white(),
            0.3,
        )),
        Light::Point(PointLight::new(
            Vec3::new(2.0, 8.0, 9.0),
            Color::white(),
            1.0,
        )),
    ]
}

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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
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

use core::color::Color;
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{VoxelScene, cast_ray_voxel};
use scene::light::Light;
use scene::portal::{PORTAL_LIGHT_INTENSITY, portal_light_color};
use scene::red_black_maze::{FAMILY_LIGHT_INTENSITY, Family, family_light_position};
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_background,
    world_camera, world_lights, world_materials, world_sky,
};

fn luminance(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

fn points() -> Vec<scene::light::PointLight> {
    world_lights()
        .into_iter()
        .filter_map(|l| match l {
            Light::Point(p) => Some(p),
            Light::Directional(_) => None,
        })
        .collect()
}

#[test]
fn the_portal_light_exists_and_the_point_lights_are_bounded() {
    let points = points();
    assert!(
        points
            .iter()
            .any(|p| p.color == portal_light_color() && p.intensity == PORTAL_LIGHT_INTENSITY)
    );
    assert_eq!(points.len(), 4, "portal + one accent per family");
    assert!(points.len() <= 4);
    for p in &points {
        assert!(p.intensity <= 0.4, "{}", p.intensity);
        assert!(p.position.x.is_finite() && p.position.y.is_finite() && p.position.z.is_finite());
    }
}

#[test]
fn there_is_no_light_per_emissive_block() {
    let scene = WorldScene::new();
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    let r = scene.rhombus();
    let mut emissive_blocks = 0;
    for z in -4..28 {
        for x in -4..28 {
            for y in r.lower_tip_y - 3..=24 {
                if let Some(b) = scene.world().get(IVec3::new(x, y, z)) {
                    if materials
                        .get(b.material_id())
                        .map(|m| m.emission_strength > 0.0)
                        .unwrap_or(false)
                    {
                        emissive_blocks += 1;
                    }
                }
            }
        }
    }
    assert!(emissive_blocks > 100, "{emissive_blocks} emissive blocks");
    assert!(points().len() * 20 < emissive_blocks);
}

#[test]
fn accent_light_colors_match_their_families_and_hang_under_the_tip() {
    let scene = WorldScene::new();
    let (t, r) = (scene.config(), scene.rhombus());
    let points = points();
    for family in Family::ALL {
        let expected = family_light_position(t, r, family);
        let light = points
            .iter()
            .find(|p| p.color == family.light_color())
            .unwrap_or_else(|| panic!("{family:?} light"));
        assert_eq!(light.position, expected);
        assert_eq!(light.intensity, FAMILY_LIGHT_INTENSITY);
        assert!(light.position.y < r.lower_tip_y as f32);
        assert!(light.position.x > 0.0 && light.position.x < t.width as f32);
        assert!(light.position.z > 0.0 && light.position.z < t.depth as f32);
    }
    let crimson = Family::Crimson.light_color();
    assert!(crimson.r > crimson.g * 3.0 && crimson.r > crimson.b * 3.0);
    let orange = Family::Orange.light_color();
    assert!(orange.r > orange.g && orange.g > orange.b + 0.3);
    let violet = Family::Violet.light_color();
    assert!(violet.b > violet.r && violet.r > violet.g);
    // Three distinct centroids.
    let mut positions: Vec<(i32, i32)> = Family::ALL
        .iter()
        .map(|f| {
            let p = family_light_position(t, r, *f);
            (p.x as i32, p.z as i32)
        })
        .collect();
    positions.dedup();
    assert_eq!(positions.len(), 3);
}

#[test]
fn the_overworld_sky_and_sun_are_preserved() {
    let lights = world_lights();
    let suns: Vec<_> = lights
        .iter()
        .filter_map(|l| match l {
            Light::Directional(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(suns.len(), 1);
    assert!(suns[0].direction.y > 0.6 && suns[0].intensity == 1.0);
    assert!(world_background().is_sky());
    assert_eq!(
        world_background().color(Vec3::new(0.0, 1.0, 0.0)),
        world_sky().zenith
    );
    assert_eq!(WORLD_AMBIENT_FACTOR, 0.30);
}

#[test]
fn the_lower_world_is_dark_but_readable_and_the_upper_world_is_not_overexposed() {
    let scene = WorldScene::new();
    let mut manager = TextureManager::new();
    let textures = WorldTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/overworld/grass",
        "assets/textures/portal",
    )
    .unwrap();
    let materials = world_materials(&textures);
    let lights = world_lights();
    let camera = world_camera(4.0 / 3.0);
    let view = VoxelScene {
        world: scene.world(),
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: WORLD_AMBIENT_FACTOR,
        background: world_background(),
        texture_manager: &manager,
        max_distance: WORLD_MAX_DISTANCE,
    };
    // Upward rays from under the diamond: the surface they hit is lit.
    let mut lit = 0.0;
    let mut hits = 0;
    for z in (2..22).step_by(2) {
        for x in (2..22).step_by(2) {
            let ray = Ray::new(
                Vec3::new(x as f32 + 0.5, -40.0, z as f32 + 0.5),
                Vec3::new(0.0, 1.0, 0.0),
            );
            let c = cast_ray_voxel(&view, &ray);
            if c != world_background().color(ray.direction) {
                lit += luminance(c);
                hits += 1;
            }
        }
    }
    assert!(hits > 40);
    let mean = lit / hits as f32;
    assert!(mean > 0.06, "the underside is black: mean luminance {mean}");
    assert!(mean < 0.5, "the underside is too bright: {mean}");
    // A sunlit grass top stays below saturation.
    let base = scene.trees()[0].base;
    let ray = Ray::new(
        Vec3::new(base.x as f32 + 0.5, 30.0, base.z as f32 + 2.5),
        Vec3::new(0.0, -1.0, 0.0),
    );
    let c = cast_ray_voxel(&view, &ray);
    assert!(luminance(c) < 0.95, "{c:?}");
}

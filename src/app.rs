use std::collections::HashMap;

use raylib::prelude::*;

use crate::camera::camera::Camera;
use crate::camera::projection::primary_ray;
use crate::config;
use crate::core::color::Color as CpuColor;
use crate::core::material::{Material, MaterialId};
use crate::core::math::Vec3;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::raytracer::cast_ray_voxel_lit;
use crate::renderer::shading::DEFAULT_AMBIENT_FACTOR;
use crate::scene::light::{DirectionalLight, Light, PointLight};
use crate::scene::scene::{
    PartialSceneTextures, diagnostic_partial_materials, diagnostic_partial_voxel_world,
};
use crate::scene::texture_manager::TextureManager;
use crate::scene::voxel_world::VoxelWorld;

const GRASS_TEXTURES_DIR: &str = "assets/textures/overworld/grass";
const PARTIAL_TEXTURES_DIR: &str = "assets/textures/diagnostic/partial";

/// Scene range for primary rays and directional shadow rays. The diagnostic
/// terrain spans only a few cells, so a modest finite range keeps every
/// DDA traversal short while still covering the whole scene from the camera.
const SCENE_MAX_DISTANCE: f32 = 24.0;

/// Loads every gallery PNG once (grass, stone, planks, door, portal core)
/// through a single `TextureManager` and returns it with the per-face
/// texture mappings. Called exactly once during scene setup, never per
/// pixel/frame.
fn load_scene_textures() -> (TextureManager, PartialSceneTextures) {
    let mut manager = TextureManager::new();
    let textures =
        PartialSceneTextures::load(&mut manager, GRASS_TEXTURES_DIR, PARTIAL_TEXTURES_DIR)
            .expect("missing gallery texture");

    (manager, textures)
}

/// Casts one primary ray per pixel into the sparse `VoxelWorld` (3D DDA +
/// local block-geometry intersection) and writes the fully shaded color
/// (ambient + visible diffuse/specular per light, hard shadows queried
/// against the same world, or `background` on a miss) into the framebuffer.
fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    world: &VoxelWorld,
    materials: &HashMap<MaterialId, Material>,
    lights: &[Light],
    background: CpuColor,
    texture_manager: &TextureManager,
) {
    let width = framebuffer.width();
    let height = framebuffer.height();

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(camera, x, y, width, height);
            let color = cast_ray_voxel_lit(
                world,
                materials,
                &ray,
                camera.position,
                lights,
                DEFAULT_AMBIENT_FACTOR,
                background,
                texture_manager,
                SCENE_MAX_DISTANCE,
            );
            framebuffer.set_pixel(x, y, color);
        }
    }
}

/// Copies already-computed framebuffer pixels into a Raylib Image, the only
/// place where `core::Color` is converted to a Raylib-facing type.
fn framebuffer_to_image(framebuffer: &Framebuffer) -> Image {
    let mut image = Image::gen_image_color(
        framebuffer.width() as i32,
        framebuffer.height() as i32,
        Color::BLACK,
    );

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            if let Some(pixel) = framebuffer.get_pixel(x, y) {
                let [r, g, b, a] = pixel.to_rgba8();
                image.draw_pixel(x as i32, y as i32, Color::new(r, g, b, a));
            }
        }
    }

    image
}

pub fn run() {
    let (mut rl, thread) = raylib::init()
        .size(config::WINDOW_WIDTH, config::WINDOW_HEIGHT)
        .title(config::WINDOW_TITLE)
        .build();

    let mut framebuffer = Framebuffer::new(
        config::WINDOW_WIDTH as usize,
        config::WINDOW_HEIGHT as usize,
    );

    let aspect_ratio = config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32;
    let camera = Camera::new(
        Vec3::new(7.4, 4.3, 9.6),
        Vec3::new(4.2, 1.4, 2.6),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    );

    // Grass Block textures are loaded once here, before any per-pixel work
    // starts; the pixel loop only ever calls `TextureManager::get` through
    // an immutable reference, so no texture can be loaded mid-render.
    let (texture_manager, scene_textures) = load_scene_textures();

    // The visible scene is the mixed partial-geometry diagnostic VoxelWorld
    // (stairs, fence, door, portal core, amethyst cluster); blocks reference
    // their material by `MaterialId`, resolved through this minimal
    // diagnostic map.
    let world = diagnostic_partial_voxel_world();
    let materials = diagnostic_partial_materials(&scene_textures);

    // A soft overhead directional fill plus a stronger side point light: the
    // point light drives the visible diffuse gradient, specular highlight,
    // and the hard shadows the taller columns cast onto lower ones.
    let lights = [
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.0, 1.0, 0.0),
            CpuColor::white(),
            0.4,
        )),
        Light::Point(PointLight::new(
            Vec3::new(-2.0, 7.0, 9.0),
            CpuColor::white(),
            1.5,
        )),
    ];

    let background = CpuColor::new(0.05, 0.05, 0.08, 1.0);

    render(
        &mut framebuffer,
        &camera,
        &world,
        &materials,
        &lights,
        background,
        &texture_manager,
    );

    let image = framebuffer_to_image(&framebuffer);
    let texture = rl
        .load_texture_from_image(&thread, &image)
        .expect("failed to upload the CPU framebuffer to a Raylib texture");

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
    }
}

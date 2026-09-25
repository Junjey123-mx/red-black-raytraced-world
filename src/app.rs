use raylib::prelude::*;

use crate::camera::camera::Camera;
use crate::camera::controls::{OrbitInput, apply_orbit_input};
use crate::camera::diagnostic::DiagnosticCameraState;
use crate::camera::projection::primary_ray;
use crate::config;
use crate::core::color::Color as CpuColor;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::raytracer::cast_ray_voxel_lit;
use crate::renderer::shading::DEFAULT_AMBIENT_FACTOR;
use crate::scene::light::Light;
use crate::scene::material_gallery::{
    GALLERY_MAX_DISTANCE, GalleryTextures, advanced_materials_library, advanced_materials_world,
    gallery_background, gallery_camera, gallery_lights,
};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::texture_manager::TextureManager;
use crate::scene::voxel_world::VoxelWorld;

/// While the camera is being moved the scene is traced at `1 / PREVIEW_DOWNSCALE`
/// of the window resolution (each preview pixel is stretched to fill its
/// block) so interaction stays responsive on a single CPU thread; as soon as
/// the input stops, one full-resolution frame is traced.
const PREVIEW_DOWNSCALE: usize = 4;

const OVERWORLD_TEXTURES_DIR: &str = "assets/textures/overworld";
const PORTAL_TEXTURES_DIR: &str = "assets/textures/portal";

/// Loads every gallery PNG once through a single `TextureManager` and
/// returns it with the per-face texture mappings. Called exactly once during
/// scene setup, never per pixel/frame.
fn load_scene_textures() -> (TextureManager, GalleryTextures) {
    let mut manager = TextureManager::new();
    let textures = GalleryTextures::load(&mut manager, OVERWORLD_TEXTURES_DIR, PORTAL_TEXTURES_DIR)
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
    materials: &MaterialLibrary,
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
                GALLERY_MAX_DISTANCE,
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

/// Polls the mouse and keyboard into one frame of `OrbitInput`: left-button
/// drag orbits, the wheel zooms, arrows are an orbit fallback, `R` resets.
fn poll_orbit_input(rl: &RaylibHandle) -> OrbitInput {
    let dragging = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
    let delta = rl.get_mouse_delta();
    let axis = |positive: KeyboardKey, negative: KeyboardKey| {
        (rl.is_key_down(positive) as i32 - rl.is_key_down(negative) as i32) as f32
    };

    OrbitInput {
        drag_dx: if dragging { delta.x } else { 0.0 },
        drag_dy: if dragging { delta.y } else { 0.0 },
        wheel: rl.get_mouse_wheel_move(),
        key_yaw: axis(KeyboardKey::KEY_RIGHT, KeyboardKey::KEY_LEFT),
        key_pitch: axis(KeyboardKey::KEY_UP, KeyboardKey::KEY_DOWN),
        delta_time: rl.get_frame_time(),
        reset: rl.is_key_pressed(KeyboardKey::KEY_R),
    }
}

/// Traces the scene from `view` into a framebuffer of `width x height` and
/// uploads it to a Raylib texture for presentation.
#[allow(clippy::too_many_arguments)]
fn trace_to_texture(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    view: &DiagnosticCameraState,
    width: usize,
    height: usize,
    world: &VoxelWorld,
    materials: &MaterialLibrary,
    lights: &[Light],
    background: CpuColor,
    texture_manager: &TextureManager,
) -> Texture2D {
    let mut framebuffer = Framebuffer::new(width, height);
    let camera = view.build_camera(config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32);

    render(
        &mut framebuffer,
        &camera,
        world,
        materials,
        lights,
        background,
        texture_manager,
    );

    let image = framebuffer_to_image(&framebuffer);
    rl.load_texture_from_image(thread, &image)
        .expect("failed to upload the CPU framebuffer to a Raylib texture")
}

pub fn run() {
    let (mut rl, thread) = raylib::init()
        .size(config::WINDOW_WIDTH, config::WINDOW_HEIGHT)
        .title(config::WINDOW_TITLE)
        .build();

    let full_width = config::WINDOW_WIDTH as usize;
    let full_height = config::WINDOW_HEIGHT as usize;

    // The diagnostic orbit camera starts on the gallery's framing, and `R`
    // restores exactly that view.
    let aspect_ratio = config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32;
    let home = gallery_camera(aspect_ratio);
    let mut view = DiagnosticCameraState::from_pose(home.position, home.target);

    // Gallery textures are loaded once here, before any per-pixel work
    // starts; the pixel loop only ever calls `TextureManager::get` through
    // an immutable reference, so no texture can be loaded mid-render.
    let (texture_manager, scene_textures) = load_scene_textures();

    // The visible scene is the advanced-materials optical gallery
    // (`advanced_materials_world`); blocks reference their material by
    // `MaterialId`, resolved through the scene `MaterialLibrary`.
    let world = advanced_materials_world();
    let materials = advanced_materials_library(&scene_textures);

    let lights = gallery_lights();
    let background = gallery_background();

    let mut texture_scale = 1;
    let mut texture = trace_to_texture(
        &mut rl,
        &thread,
        &view,
        full_width,
        full_height,
        &world,
        &materials,
        &lights,
        background,
        &texture_manager,
    );
    // A full-resolution frame is up to date until the camera moves again.
    let mut needs_full_frame = false;

    while !rl.window_should_close() {
        let input = poll_orbit_input(&rl);

        if input.is_active() {
            apply_orbit_input(&mut view, &input);
            texture_scale = PREVIEW_DOWNSCALE;
            texture = trace_to_texture(
                &mut rl,
                &thread,
                &view,
                full_width / PREVIEW_DOWNSCALE,
                full_height / PREVIEW_DOWNSCALE,
                &world,
                &materials,
                &lights,
                background,
                &texture_manager,
            );
            needs_full_frame = true;
        } else if needs_full_frame {
            texture_scale = 1;
            texture = trace_to_texture(
                &mut rl,
                &thread,
                &view,
                full_width,
                full_height,
                &world,
                &materials,
                &lights,
                background,
                &texture_manager,
            );
            needs_full_frame = false;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_ex(
            &texture,
            Vector2::new(0.0, 0.0),
            0.0,
            texture_scale as f32,
            Color::WHITE,
        );
    }
}

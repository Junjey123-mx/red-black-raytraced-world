use raylib::prelude::*;

use crate::camera::camera::Camera;
use crate::camera::projection::primary_ray;
use crate::config;
use crate::core::color::Color as CpuColor;
use crate::core::cube::Cube;
use crate::core::face_textures::FaceTextures;
use crate::core::material::Material;
use crate::core::math::Vec3;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::raytracer::cast_ray_lit;
use crate::renderer::shading::DEFAULT_AMBIENT_FACTOR;
use crate::scene::light::{DirectionalLight, Light, PointLight};
use crate::scene::texture_manager::TextureManager;

const GRASS_TEXTURES_DIR: &str = "assets/textures/overworld/grass";

/// Loads the three Grass Block PNGs once — `top` and `bottom` are unique,
/// while `side` is loaded a single time and its `TextureId` is reused for
/// all four lateral faces, demonstrating that `FaceTextures` can share ids
/// — and returns the manager plus a `FaceTextures` mapping ready to attach
/// to the diagnostic cube's material. Called exactly once during scene
/// setup, never per pixel/frame.
fn load_grass_face_textures() -> (TextureManager, FaceTextures) {
    let mut manager = TextureManager::new();
    let path = |name: &str| format!("{GRASS_TEXTURES_DIR}/{name}.png");

    let top = manager
        .load(path("top"))
        .expect("missing grass texture: top.png");
    let side = manager
        .load(path("side"))
        .expect("missing grass texture: side.png");
    let bottom = manager
        .load(path("bottom"))
        .expect("missing grass texture: bottom.png");

    let face_textures = FaceTextures::new(side, side, top, bottom, side, side);

    (manager, face_textures)
}

/// Casts one primary ray per pixel against the diagnostic lighting scene
/// (`objects` + `lights`) and writes the fully shaded color (ambient +
/// visible diffuse/specular per light, or `background` on a miss) into the
/// framebuffer. This is the Category 3 basic-lighting checkpoint image.
fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    objects: &[(Cube, Material)],
    lights: &[Light],
    background: CpuColor,
    texture_manager: &TextureManager,
) {
    let width = framebuffer.width();
    let height = framebuffer.height();

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(camera, x, y, width, height);
            let color = cast_ray_lit(
                objects,
                &ray,
                camera.position,
                lights,
                DEFAULT_AMBIENT_FACTOR,
                background,
                texture_manager,
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
        Vec3::new(6.0, 4.0, 8.0),
        Vec3::new(0.0, -0.3, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    );

    // Grass Block textures are loaded once here, before any per-pixel work
    // starts; the pixel loop only ever calls `TextureManager::get` through
    // an immutable reference, so no texture can be loaded mid-render.
    let (texture_manager, face_textures) = load_grass_face_textures();

    // Main diagnostic cube now represents a Grass Block: albedo is a
    // fallback only (every face has a texture assigned, so it is never
    // actually sampled), and specular/shininess follow the project's Grass
    // material spec — mostly matte, with a light, subtle highlight.
    let main_cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    let main_material = Material::new(CpuColor::new(0.45, 0.55, 0.25, 1.0), 0.04, 8.0)
        .with_face_textures(face_textures);

    // Flat receptor floor built from a squashed Cube, wide enough to catch
    // the main cube's projected hard shadow.
    let floor_cube = Cube::new(Vec3::new(-5.0, -2.0, -5.0), Vec3::new(5.0, -1.5, 5.0));
    let floor_material = Material::matte(CpuColor::new(0.6, 0.6, 0.65, 1.0));

    let objects = [(main_cube, main_material), (floor_cube, floor_material)];

    // A soft overhead directional fill plus a stronger side point light: the
    // point light drives the visible diffuse gradient, specular highlight,
    // and the shadow cast onto the floor.
    let lights = [
        Light::Directional(DirectionalLight::new(
            Vec3::new(0.0, 1.0, 0.0),
            CpuColor::white(),
            0.4,
        )),
        Light::Point(PointLight::new(
            Vec3::new(-4.0, 6.0, 5.0),
            CpuColor::white(),
            1.5,
        )),
    ];

    let background = CpuColor::new(0.05, 0.05, 0.08, 1.0);

    render(
        &mut framebuffer,
        &camera,
        &objects,
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

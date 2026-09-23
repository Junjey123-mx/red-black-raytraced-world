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

const DIAGNOSTIC_FACE_TEXTURES_DIR: &str = "assets/textures/diagnostic/faces";

/// Loads the six diagnostic face PNGs once and returns the manager plus a
/// `FaceTextures` mapping ready to attach to the diagnostic cube's
/// material. Called exactly once during scene setup, never per pixel/frame.
fn load_diagnostic_face_textures() -> (TextureManager, FaceTextures) {
    let mut manager = TextureManager::new();
    let path = |name: &str| format!("{DIAGNOSTIC_FACE_TEXTURES_DIR}/{name}.png");

    let positive_x = manager
        .load(path("positive_x"))
        .expect("missing diagnostic texture: positive_x.png");
    let negative_x = manager
        .load(path("negative_x"))
        .expect("missing diagnostic texture: negative_x.png");
    let positive_y = manager
        .load(path("positive_y"))
        .expect("missing diagnostic texture: positive_y.png");
    let negative_y = manager
        .load(path("negative_y"))
        .expect("missing diagnostic texture: negative_y.png");
    let positive_z = manager
        .load(path("positive_z"))
        .expect("missing diagnostic texture: positive_z.png");
    let negative_z = manager
        .load(path("negative_z"))
        .expect("missing diagnostic texture: negative_z.png");

    let face_textures = FaceTextures::new(
        positive_x, negative_x, positive_y, negative_y, positive_z, negative_z,
    );

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

    // Diagnostic face textures are loaded once here, before any per-pixel
    // work starts; the pixel loop only ever calls `TextureManager::get`
    // through an immutable reference, so no texture can be loaded mid-render.
    let (texture_manager, face_textures) = load_diagnostic_face_textures();

    // Main diagnostic cube: warm-colored fallback albedo, now overridden per
    // face by the loaded diagnostic textures; specular/shininess stay
    // independent of texturing so the highlight remains visible.
    let main_cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
    let main_material = Material::new(CpuColor::new(0.9, 0.5, 0.2, 1.0), 3.0, 2.0)
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

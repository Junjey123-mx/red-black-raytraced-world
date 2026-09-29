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
use crate::scene::catalog::{CatalogScene, CatalogTextures, catalog_materials};
use crate::scene::light::Light;
use crate::scene::material_gallery::{
    GALLERY_MAX_DISTANCE, gallery_background, gallery_camera, gallery_lights,
};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::texture_manager::TextureManager;
use crate::scene::voxel_world::VoxelWorld;
use crate::scene::world::{
    WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_background, world_camera, world_lights,
    world_materials,
};

/// While the camera is being moved the scene is traced at `1 / PREVIEW_DOWNSCALE`
/// of the window resolution (each preview pixel is stretched to fill its
/// block) so interaction stays responsive on a single CPU thread; as soon as
/// the input stops, one full-resolution frame is traced.
const PREVIEW_DOWNSCALE: usize = 4;

const OVERWORLD_TEXTURES_DIR: &str = "assets/textures/overworld";
const PORTAL_TEXTURES_DIR: &str = "assets/textures/portal";
const GRASS_TEXTURES_DIR: &str = "assets/textures/overworld/grass";
const SHAPE_TEXTURES_DIR: &str = "assets/textures/diagnostic/partial";

/// Everything that differs between the project's executables: which scene is
/// traced and its mode-specific camera, controls and label. The window,
/// texture loading, CPU tracing, orbit/zoom/reset input and presentation are
/// the shared runtime (`run_viewer`), identical for every mode.
trait ViewerMode {
    fn world(&self) -> &VoxelWorld;
    fn materials(&self) -> &MaterialLibrary;
    fn lights(&self) -> &[Light];
    fn background(&self) -> CpuColor;
    /// Scene range: bounds primary traversal and directional shadow rays.
    fn max_distance(&self) -> f32;
    /// The orbit camera's starting (and `R` reset) pose.
    fn home_camera(&self, aspect_ratio: f32) -> Camera;
    /// Mode-specific keys, polled once per frame. Returns `true` when they
    /// changed the view, so a new frame must be traced.
    fn handle_keys(&mut self, rl: &RaylibHandle, view: &mut DiagnosticCameraState) -> bool;
    /// One-line status text drawn in the top-left corner.
    fn label(&self) -> String;
    /// Controls hint drawn along the bottom edge.
    fn help(&self) -> &'static str;
}

/// The block catalog: the definitive 39-block `CatalogScene` plus the optical
/// diagnostics, lit by the gallery lights, with selection and focus keys.
struct CatalogMode {
    catalog: CatalogScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
}

impl CatalogMode {
    /// Loads every catalog PNG once through the shared `TextureManager`
    /// (never per pixel/frame) and builds the catalog scene and materials.
    fn new(manager: &mut TextureManager) -> Self {
        let textures = CatalogTextures::load(
            manager,
            OVERWORLD_TEXTURES_DIR,
            PORTAL_TEXTURES_DIR,
            GRASS_TEXTURES_DIR,
            SHAPE_TEXTURES_DIR,
        )
        .expect("missing catalog texture");

        // The visible scene is the persistent block catalog (`CatalogScene`).
        // Blocks reference their material by `MaterialId`, resolved through
        // the `MaterialLibrary`.
        Self {
            catalog: CatalogScene::new(),
            materials: catalog_materials(&textures),
            lights: gallery_lights(),
        }
    }
}

impl ViewerMode for CatalogMode {
    fn world(&self) -> &VoxelWorld {
        self.catalog.world()
    }

    fn materials(&self) -> &MaterialLibrary {
        &self.materials
    }

    fn lights(&self) -> &[Light] {
        &self.lights
    }

    fn background(&self) -> CpuColor {
        gallery_background()
    }

    fn max_distance(&self) -> f32 {
        GALLERY_MAX_DISTANCE
    }

    fn home_camera(&self, aspect_ratio: f32) -> Camera {
        gallery_camera(aspect_ratio)
    }

    /// Catalog selection: [ / Q previous, ] / E next (wrapping), F focus.
    fn handle_keys(&mut self, rl: &RaylibHandle, view: &mut DiagnosticCameraState) -> bool {
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) || rl.is_key_pressed(KeyboardKey::KEY_Q)
        {
            self.catalog.select_previous();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET)
            || rl.is_key_pressed(KeyboardKey::KEY_E)
        {
            self.catalog.select_next();
        }
        let focus = rl.is_key_pressed(KeyboardKey::KEY_F);
        if focus {
            self.catalog.focus_camera(view);
        }
        focus
    }

    fn label(&self) -> String {
        self.catalog.label()
    }

    fn help(&self) -> &'static str {
        "Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset"
    }
}

/// The main world: the Overworld `WorldScene`. It never builds `CatalogScene`.
struct WorldMode {
    scene: WorldScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
}

impl WorldMode {
    /// Loads the world's textures once through the shared `TextureManager`
    /// and builds the Overworld.
    fn new(manager: &mut TextureManager) -> Self {
        let textures = WorldTextures::load(manager, OVERWORLD_TEXTURES_DIR, GRASS_TEXTURES_DIR)
            .expect("missing world texture");
        Self {
            scene: WorldScene::new(),
            materials: world_materials(&textures),
            lights: world_lights(),
        }
    }
}

impl ViewerMode for WorldMode {
    fn world(&self) -> &VoxelWorld {
        self.scene.world()
    }

    fn materials(&self) -> &MaterialLibrary {
        &self.materials
    }

    fn lights(&self) -> &[Light] {
        &self.lights
    }

    fn background(&self) -> CpuColor {
        world_background()
    }

    fn max_distance(&self) -> f32 {
        WORLD_MAX_DISTANCE
    }

    fn home_camera(&self, aspect_ratio: f32) -> Camera {
        world_camera(aspect_ratio)
    }

    /// The world has no mode-specific keys yet.
    fn handle_keys(&mut self, _rl: &RaylibHandle, _view: &mut DiagnosticCameraState) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Overworld | {} blocks", self.scene.world().len())
    }

    fn help(&self) -> &'static str {
        "Drag: orbit | Wheel: zoom | R: reset | Block catalog: cargo run --bin catalog"
    }
}

/// Casts one primary ray per pixel into the sparse `VoxelWorld` (3D DDA +
/// local block-geometry intersection) and writes the fully shaded color
/// (ambient + visible diffuse/specular per light, hard shadows queried
/// against the same world, or the background on a miss) into the framebuffer.
fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    mode: &dyn ViewerMode,
    texture_manager: &TextureManager,
) {
    let width = framebuffer.width();
    let height = framebuffer.height();

    for y in 0..height {
        for x in 0..width {
            let ray = primary_ray(camera, x, y, width, height);
            let color = cast_ray_voxel_lit(
                mode.world(),
                mode.materials(),
                &ray,
                camera.position,
                mode.lights(),
                DEFAULT_AMBIENT_FACTOR,
                mode.background(),
                texture_manager,
                mode.max_distance(),
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
fn trace_to_texture(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    view: &DiagnosticCameraState,
    width: usize,
    height: usize,
    mode: &dyn ViewerMode,
    texture_manager: &TextureManager,
) -> Texture2D {
    let mut framebuffer = Framebuffer::new(width, height);
    let camera = view.build_camera(config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32);

    render(&mut framebuffer, &camera, mode, texture_manager);

    let image = framebuffer_to_image(&framebuffer);
    rl.load_texture_from_image(thread, &image)
        .expect("failed to upload the CPU framebuffer to a Raylib texture")
}

/// The shared runtime of every executable: opens the window, creates the one
/// `TextureManager`, lets `build_mode` load its textures and scene through
/// it, and runs the interactive loop that traces each frame on the CPU and
/// presents it through Raylib.
fn run_viewer<M: ViewerMode>(build_mode: impl FnOnce(&mut TextureManager) -> M) {
    let (mut rl, thread) = raylib::init()
        .size(config::WINDOW_WIDTH, config::WINDOW_HEIGHT)
        .title(config::WINDOW_TITLE)
        .build();

    let full_width = config::WINDOW_WIDTH as usize;
    let full_height = config::WINDOW_HEIGHT as usize;

    // Textures are loaded once here, before any per-pixel work starts; the
    // pixel loop only ever calls `TextureManager::get` through an immutable
    // reference, so no texture can be loaded mid-render.
    let mut texture_manager = TextureManager::new();
    let mut mode = build_mode(&mut texture_manager);

    // The orbit camera starts on the mode's framing, and `R` restores
    // exactly that view.
    let aspect_ratio = config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32;
    let home = mode.home_camera(aspect_ratio);
    let mut view = DiagnosticCameraState::from_pose(home.position, home.target);

    let mut texture_scale = 1;
    let mut texture = trace_to_texture(
        &mut rl,
        &thread,
        &view,
        full_width,
        full_height,
        &mode,
        &texture_manager,
    );
    // A full-resolution frame is up to date until the camera moves again.
    let mut needs_full_frame = false;

    while !rl.window_should_close() {
        let input = poll_orbit_input(&rl);
        let mode_changed_view = mode.handle_keys(&rl, &mut view);

        if input.is_active() || mode_changed_view {
            apply_orbit_input(&mut view, &input);
            texture_scale = PREVIEW_DOWNSCALE;
            texture = trace_to_texture(
                &mut rl,
                &thread,
                &view,
                full_width / PREVIEW_DOWNSCALE,
                full_height / PREVIEW_DOWNSCALE,
                &mode,
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
                &mode,
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
        d.draw_text(&mode.label(), 12, 10, 20, Color::WHITE);
        d.draw_text(
            mode.help(),
            12,
            config::WINDOW_HEIGHT - 26,
            16,
            Color::LIGHTGRAY,
        );
    }
}

/// The catalog application: the shared runtime showing the definitive
/// 39-block `CatalogScene` and its diagnostics, with selection and focus.
pub fn run_catalog_app() {
    run_viewer(CatalogMode::new);
}

/// The world application (`cargo run`): the shared runtime showing the main
/// `WorldScene`, never the catalog.
pub fn run_world_app() {
    run_viewer(WorldMode::new);
}

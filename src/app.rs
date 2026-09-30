use std::time::{Duration, Instant};

use raylib::prelude::*;

use crate::camera::camera::Camera;
use crate::camera::controls::{OrbitInput, apply_orbit_input};
use crate::camera::diagnostic::DiagnosticCameraState;
use crate::camera::portal_crossing::{PortalCrossingDetector, PortalVolume};
use crate::camera::world_free_fly::{FreeFlyInput, WorldFreeFlyCameraState, apply_free_fly_input};
use crate::config;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::parallel::{ParallelRenderConfig, render_parallel};
use crate::renderer::perf::{FramePerfStats, PerfReporter, TracedFrameKind};
use crate::renderer::preview::AdaptivePreview;
use crate::renderer::raytracer::{RenderQuality, VoxelScene};
use crate::renderer::shading::DEFAULT_AMBIENT_FACTOR;
use crate::renderer::skybox::Background;
use crate::scene::catalog::{CatalogScene, CatalogTextures, catalog_materials};
use crate::scene::light::Light;
use crate::scene::material_gallery::{
    GALLERY_MAX_DISTANCE, gallery_background, gallery_camera, gallery_lights,
};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::texture_manager::TextureManager;
use crate::scene::voxel_world::VoxelWorld;
use crate::scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};

/// While the camera is being moved the scene is traced at a reduced scale
/// (each preview pixel is stretched to fill its block) and at Interactive
/// quality; as soon as the input stops, one full-resolution Full frame is
/// traced. The World picks its scale adaptively (`AdaptivePreview`); the
/// Catalog keeps this fixed downscale, unchanged.
const PREVIEW_DOWNSCALE: usize = 4;

const OVERWORLD_TEXTURES_DIR: &str = "assets/textures/overworld";
const PORTAL_TEXTURES_DIR: &str = "assets/textures/portal";
const GRASS_TEXTURES_DIR: &str = "assets/textures/overworld/grass";
const SHAPE_TEXTURES_DIR: &str = "assets/textures/diagnostic/partial";

/// Everything that differs between the project's executables: which scene is
/// traced, and its own camera, controls and label. The window, texture
/// loading, CPU tracing and presentation are the shared runtime
/// (`run_viewer`), identical for every mode. The catalog keeps the orbit
/// (diagnostic) camera; the world flies free.
trait ViewerMode {
    fn world(&self) -> &VoxelWorld;
    fn materials(&self) -> &MaterialLibrary;
    fn lights(&self) -> &[Light];
    /// What a ray sees on a miss: a flat backdrop or a directional sky.
    fn background(&self) -> Background;
    /// Ambient term of the local shading.
    fn ambient_factor(&self) -> f32;
    /// Scene range: bounds primary traversal and directional shadow rays.
    fn max_distance(&self) -> f32;
    /// One-time window setup (e.g. capturing the cursor for mouse look).
    fn setup(&self, _rl: &mut RaylibHandle) {}
    /// Polls this frame's input and advances the mode's camera. Returns
    /// `true` when the view changed, so a new frame must be traced.
    fn update(&mut self, rl: &RaylibHandle) -> bool;
    /// The raytracer camera for the current view.
    fn camera(&self, aspect_ratio: f32) -> Camera;
    /// One-line status text drawn in the top-left corner.
    fn label(&self) -> String;
    /// Controls hint drawn along the bottom edge.
    fn help(&self) -> &'static str;
    /// Whether moving frames pick their preview scale from measured render
    /// time (`AdaptivePreview`) instead of the fixed `PREVIEW_DOWNSCALE`.
    fn adaptive_preview(&self) -> bool {
        false
    }
}

/// The block catalog: the definitive 39-block `CatalogScene` plus the optical
/// diagnostics, lit by the gallery lights, with selection and focus keys.
struct CatalogMode {
    catalog: CatalogScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
    /// The diagnostic orbit camera, starting on the gallery framing.
    view: DiagnosticCameraState,
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
        let home = gallery_camera(config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32);
        Self {
            catalog: CatalogScene::new(),
            materials: catalog_materials(&textures),
            lights: gallery_lights(),
            view: DiagnosticCameraState::from_pose(home.position, home.target),
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

    fn background(&self) -> Background {
        Background::Solid(gallery_background())
    }

    fn ambient_factor(&self) -> f32 {
        DEFAULT_AMBIENT_FACTOR
    }

    fn max_distance(&self) -> f32 {
        GALLERY_MAX_DISTANCE
    }

    /// Orbit/zoom/reset input plus the catalog keys: [ / Q previous,
    /// ] / E next (wrapping), F focus. Unchanged catalog behavior.
    fn update(&mut self, rl: &RaylibHandle) -> bool {
        let input = poll_orbit_input(rl);
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
            self.catalog.focus_camera(&mut self.view);
        }
        if input.is_active() {
            apply_orbit_input(&mut self.view, &input);
        }
        input.is_active() || focus
    }

    fn camera(&self, aspect_ratio: f32) -> Camera {
        self.view.build_camera(aspect_ratio)
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
    /// The World's own free-fly camera (the catalog keeps the orbit camera).
    free_fly: WorldFreeFlyCameraState,
    /// Fires when the camera's movement pierces the portal membrane.
    portal_detector: PortalCrossingDetector,
}

impl WorldMode {
    /// Loads the world's textures once through the shared `TextureManager`
    /// and builds the Overworld.
    fn new(manager: &mut TextureManager) -> Self {
        let textures = WorldTextures::load(
            manager,
            OVERWORLD_TEXTURES_DIR,
            GRASS_TEXTURES_DIR,
            PORTAL_TEXTURES_DIR,
        )
        .expect("missing world texture");
        let scene = WorldScene::new();
        let portal_detector =
            PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
        Self {
            scene,
            materials: world_materials(&textures),
            lights: world_lights(),
            free_fly: world_free_fly_camera(),
            portal_detector,
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

    /// The realm's sky (blended through the portal transition), hung
    /// along the camera's current vertical.
    fn background(&self) -> Background {
        self.free_fly.background()
    }

    fn ambient_factor(&self) -> f32 {
        WORLD_AMBIENT_FACTOR * self.free_fly.environment().ambient_scale
    }

    fn max_distance(&self) -> f32 {
        WORLD_MAX_DISTANCE
    }

    /// Mouse look needs a captured cursor: relative motion, no edges. The
    /// default ESC exit key is disabled so a stray key event cannot end a
    /// recording; the window's close button still quits.
    fn setup(&self, rl: &mut RaylibHandle) {
        rl.set_exit_key(None);
        rl.disable_cursor();
    }

    /// WASD / Space / Shift flight, mouse look and R, all through the
    /// free-fly state (never the orbit camera).
    fn update(&mut self, rl: &RaylibHandle) -> bool {
        let input = poll_free_fly_input(rl);
        // An active roll keeps changing the view even without input.
        let rolling = self.free_fly.advance_transition(rl.get_frame_time());
        if input.is_active() {
            let previous = self.free_fly.position;
            apply_free_fly_input(&mut self.free_fly, &input);
            if input.reset {
                self.portal_detector.reset();
            } else if self.free_fly.transition().is_none() {
                if let Some(event) = self.portal_detector.detect(
                    previous,
                    self.free_fly.position,
                    self.free_fly.realm,
                ) {
                    self.free_fly.begin_transition(event);
                }
            }
        }
        input.is_active() || rolling
    }

    fn camera(&self, aspect_ratio: f32) -> Camera {
        self.free_fly.camera(aspect_ratio)
    }

    fn label(&self) -> String {
        format!(
            "{} | {} blocks",
            self.free_fly.realm.label(),
            self.scene.world().len()
        )
    }

    fn help(&self) -> &'static str {
        "WASD move | Mouse look | Space/Shift up/down | R reset"
    }

    /// The World prefers the sharper 400x300 preview whenever it fits the
    /// interactive budget.
    fn adaptive_preview(&self) -> bool {
        true
    }
}

/// Casts one primary ray per pixel into the sparse `VoxelWorld` (3D DDA +
/// local block-geometry intersection) and writes the fully shaded color
/// (ambient + visible diffuse/specular per light, hard shadows queried
/// against the same world, or the background on a miss) into the framebuffer.
/// The pixels are traced by `render_parallel` across the configured worker
/// threads; every pixel still goes through `cast_ray_voxel_at`.
fn render(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    mode: &dyn ViewerMode,
    texture_manager: &TextureManager,
    quality: RenderQuality,
    threads: &ParallelRenderConfig,
) {
    // Assembled once per frame; every pixel traces through the same scene.
    let scene = VoxelScene {
        world: mode.world(),
        materials: mode.materials(),
        camera_position: camera.position,
        lights: mode.lights(),
        ambient_factor: mode.ambient_factor(),
        background: mode.background(),
        texture_manager,
        max_distance: mode.max_distance(),
    };

    render_parallel(framebuffer, camera, &scene, quality, threads);
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

/// Polls the keyboard and mouse into one frame of `FreeFlyInput`: WASD
/// along the view frame, Space / Left Shift along the local up, the mouse
/// delta for looking, `R` to reset.
fn poll_free_fly_input(rl: &RaylibHandle) -> FreeFlyInput {
    let axis = |positive: KeyboardKey, negative: KeyboardKey| {
        (rl.is_key_down(positive) as i32 - rl.is_key_down(negative) as i32) as f32
    };
    let delta = rl.get_mouse_delta();
    FreeFlyInput {
        move_forward: axis(KeyboardKey::KEY_W, KeyboardKey::KEY_S),
        move_right: axis(KeyboardKey::KEY_D, KeyboardKey::KEY_A),
        move_up: axis(KeyboardKey::KEY_SPACE, KeyboardKey::KEY_LEFT_SHIFT),
        look_dx: delta.x,
        look_dy: delta.y,
        delta_time: rl.get_frame_time(),
        reset: rl.is_key_pressed(KeyboardKey::KEY_R),
    }
}

/// How long the two halves of a traced frame took: the CPU render and the
/// framebuffer-to-texture upload.
struct TraceTiming {
    render: Duration,
    upload: Duration,
}

/// Traces the scene from the mode's camera into a framebuffer of
/// `width x height` and uploads it to a Raylib texture for presentation.
fn trace_to_texture(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    width: usize,
    height: usize,
    mode: &dyn ViewerMode,
    texture_manager: &TextureManager,
    quality: RenderQuality,
    threads: &ParallelRenderConfig,
) -> (Texture2D, TraceTiming) {
    let started = Instant::now();
    let mut framebuffer = Framebuffer::new(width, height);
    let camera = mode.camera(config::WINDOW_WIDTH as f32 / config::WINDOW_HEIGHT as f32);

    render(
        &mut framebuffer,
        &camera,
        mode,
        texture_manager,
        quality,
        threads,
    );
    let rendered = Instant::now();

    let image = framebuffer_to_image(&framebuffer);
    let texture = rl
        .load_texture_from_image(thread, &image)
        .expect("failed to upload the CPU framebuffer to a Raylib texture");
    let timing = TraceTiming {
        render: rendered - started,
        upload: rendered.elapsed(),
    };
    (texture, timing)
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
    mode.setup(&mut rl);

    // Frame timing is reported only when `RBRW_PERF` asks for it.
    let perf = PerfReporter::from_env();
    // Worker threads: one per hardware thread unless `RBRW_THREADS` says otherwise.
    let threads = ParallelRenderConfig::detect();
    // Preview scale chosen from measured Interactive render times.
    let mut preview = AdaptivePreview::new();

    let mut texture_scale = 1;
    let (mut texture, _) = trace_to_texture(
        &mut rl,
        &thread,
        full_width,
        full_height,
        &mode,
        &texture_manager,
        RenderQuality::Full,
        &threads,
    );
    // A full-resolution frame is up to date until the camera moves again.
    let mut needs_full_frame = false;

    while !rl.window_should_close() {
        // Each mode owns and advances its own camera.
        let mut traced: Option<(TracedFrameKind, usize, usize, TraceTiming)> = None;
        if mode.update(&rl) {
            let scale = if mode.adaptive_preview() {
                preview.scale()
            } else {
                PREVIEW_DOWNSCALE
            };
            texture_scale = scale;
            let (width, height) = (full_width / scale, full_height / scale);
            // A moving view is traced at Interactive quality.
            let (new_texture, timing) = trace_to_texture(
                &mut rl,
                &thread,
                width,
                height,
                &mode,
                &texture_manager,
                RenderQuality::Interactive,
                &threads,
            );
            texture = new_texture;
            if mode.adaptive_preview() {
                preview.record(timing.render);
            }
            traced = Some((TracedFrameKind::Interactive, width, height, timing));
            needs_full_frame = true;
        } else if needs_full_frame {
            texture_scale = 1;
            let (new_texture, timing) = trace_to_texture(
                &mut rl,
                &thread,
                full_width,
                full_height,
                &mode,
                &texture_manager,
                RenderQuality::Full,
                &threads,
            );
            texture = new_texture;
            traced = Some((TracedFrameKind::Full, full_width, full_height, timing));
            needs_full_frame = false;
        }

        let drawing_started = Instant::now();
        {
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

        if let Some((kind, width, height, timing)) = traced {
            let presentation = timing.upload + drawing_started.elapsed();
            perf.report(&FramePerfStats::new(
                kind,
                width,
                height,
                timing.render,
                presentation,
            ));
        }
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

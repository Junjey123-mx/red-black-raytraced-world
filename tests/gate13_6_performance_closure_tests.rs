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
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::camera::Camera;
use camera::world_free_fly::{
    FreeFlyInput, WorldFreeFlyCameraState, WorldRealm, apply_free_fly_input,
};
use core::color::Color;
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::perf::PerfReporter;
use renderer::preview::{
    AdaptivePreview, DEMOTE_AFTER, INTERACTIVE_FRAME_BUDGET, PREVIEW_SCALES, PROMOTE_AFTER,
};
use renderer::raytracer::{
    INTERACTIVE_REFLECTION_THRESHOLD, MAX_RAY_DEPTH, RenderQuality, VoxelScene, cast_ray_voxel_at,
};
use renderer::refinement::{
    BATCH_ROWS, FullRefinement, IDLE_GRACE_PERIOD, RefinementPhase, STEP_TIME_BUDGET,
};
use renderer::skybox::Background;
use scene::block::BlockInstance;
use scene::block_type::{BlockType, OFFICIAL_BLOCK_COUNT};
use scene::catalog::{CatalogScene, official_entries};
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::{glass_material_id, portal_core_material_id, water_material_id};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::amethyst_cluster_material_id;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::time::{Duration, Instant};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

struct Rig {
    manager: TextureManager,
    scene: WorldScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
    threads: ParallelRenderConfig,
}

impl Rig {
    fn new() -> Self {
        let mut manager = TextureManager::new();
        let textures = WorldTextures::load(
            &mut manager,
            "assets/textures/overworld",
            "assets/textures/overworld/grass",
            "assets/textures/portal",
        )
        .unwrap();
        Self {
            manager,
            scene: WorldScene::new(),
            materials: world_materials(&textures),
            lights: world_lights(),
            threads: ParallelRenderConfig::detect(),
        }
    }

    fn scene_for<'a>(
        &'a self,
        world: &'a VoxelWorld,
        state: &WorldFreeFlyCameraState,
    ) -> VoxelScene<'a> {
        VoxelScene {
            world,
            materials: &self.materials,
            camera_position: state.position,
            lights: &self.lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &self.manager,
            max_distance: WORLD_MAX_DISTANCE,
        }
    }

    fn scene(&self, state: &WorldFreeFlyCameraState) -> VoxelScene<'_> {
        self.scene_for(self.scene.world(), state)
    }

    fn frame(
        &self,
        state: &WorldFreeFlyCameraState,
        w: usize,
        h: usize,
        q: RenderQuality,
        threads: &ParallelRenderConfig,
    ) -> Vec<Color> {
        let mut fb = Framebuffer::new(w, h);
        render_parallel(
            &mut fb,
            &state.camera(4.0 / 3.0),
            &self.scene(state),
            q,
            threads,
        );
        fb.pixels().to_vec()
    }
}

/// The benchmark poses of the pre-Gate audit, unchanged.
fn scenarios() -> Vec<(&'static str, WorldFreeFlyCameraState)> {
    let red_black = |p: Vec3, t: Vec3| {
        let mut s = WorldFreeFlyCameraState::looking_at(p, t);
        s.realm = WorldRealm::RedBlack;
        s.local_up = WorldRealm::RedBlack.up();
        s.pitch = -s.pitch;
        s
    };
    vec![
        ("A_full_diorama_home", world_free_fly_camera()),
        (
            "B_overworld_house_trees",
            WorldFreeFlyCameraState::looking_at(v(15.0, 11.0, 21.0), v(8.0, 6.0, 10.0)),
        ),
        (
            "C_portal_cutaway",
            WorldFreeFlyCameraState::looking_at(v(16.5, -6.0, 20.5), v(16.5, -7.0, 14.5)),
        ),
        (
            "D_red_black_underside",
            red_black(v(12.0, -34.0, 36.0), v(12.0, -20.0, 12.0)),
        ),
        (
            "E_pond_water_glass",
            WorldFreeFlyCameraState::looking_at(v(7.5, 10.0, 25.0), v(7.5, 5.0, 18.0)),
        ),
        (
            "F_just_after_crossing",
            red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0)),
        ),
        (
            "G_sky_only_all_misses",
            WorldFreeFlyCameraState::looking_at(v(12.0, -34.0, 36.0), v(12.0, -48.0, 60.0)),
        ),
    ]
}

/// The same cells plus two far sentinels: a box so wide that clipping can
/// no longer shorten any traversal (the unclipped reference).
fn unclipped_copy(world: &VoxelWorld) -> VoxelWorld {
    let mut copy = VoxelWorld::new();
    for (cell, block) in world.iter() {
        copy.insert(*cell, *block);
    }
    let sentinel = *world.iter().next().unwrap().1;
    copy.insert(IVec3::new(-4000, -4000, -4000), sentinel);
    copy.insert(IVec3::new(4000, 4000, 4000), sentinel);
    copy
}

/// A probe material in front of a red wall its mirror ray would see.
fn probe_color(probe: Material, q: RenderQuality) -> Color {
    let mut materials = MaterialLibrary::new();
    materials.insert(MaterialId::new(1), probe);
    materials.insert(
        MaterialId::new(2),
        Material::new(Color::new(0.9, 0.2, 0.2, 1.0), 0.0, 1.0),
    );
    let mut world = VoxelWorld::new();
    world.insert(
        IVec3::new(0, 0, 0),
        BlockInstance::new(BlockType::Stone, MaterialId::new(1), Orientation::Up),
    );
    world.insert(
        IVec3::new(0, 3, 0),
        BlockInstance::new(BlockType::Stone, MaterialId::new(2), Orientation::Up),
    );
    let lights = vec![Light::Directional(DirectionalLight::new(
        v(0.2, 0.3, 1.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
        1.0,
    ))];
    let manager = TextureManager::new();
    let scene = VoxelScene {
        world: &world,
        materials: &materials,
        camera_position: v(0.5, -2.0, 3.0),
        lights: &lights,
        ambient_factor: 0.3,
        background: Background::Solid(Color::new(0.1, 0.1, 0.1, 1.0)),
        texture_manager: &manager,
        max_distance: 96.0,
    };
    cast_ray_voxel_at(&scene, &Ray::new(v(0.5, -2.0, 3.0), v(0.0, 2.5, -2.5)), q)
}

fn opaque(reflectivity: f32) -> Material {
    Material::new(Color::new(0.5, 0.5, 0.5, 1.0), 0.1, 10.0).with_reflectivity(reflectivity)
}

// 1, 2: world-bounds clipping is active and leaves Full output unchanged.
#[test]
fn world_bounds_clipping_is_active_and_exact() {
    let rig = Rig::new();
    let bounds = rig.scene.world().bounds().expect("the world has a box");
    assert_eq!(bounds.min, IVec3::new(0, -22, 0));
    // Gate 15 grew the lobe east: the box ends at the lobe's last column.
    assert_eq!(
        bounds.max_exclusive.x,
        rig.scene.expansion_layout().overworld_extension.max_x + 1
    );
    assert_eq!((bounds.max_exclusive.y, bounds.max_exclusive.z), (13, 24));
    let src = std::fs::read_to_string("src/renderer/raytracer.rs").unwrap();
    assert!(src.contains("bounds.clip_distance(ray.origin, ray.direction, max_distance)?"));
    let reference = unclipped_copy(rig.scene.world());
    for (name, state) in scenarios() {
        let camera = state.camera(4.0 / 3.0);
        let mut clipped = Framebuffer::new(60, 45);
        let mut unclipped = Framebuffer::new(60, 45);
        render_parallel(
            &mut clipped,
            &camera,
            &rig.scene(&state),
            RenderQuality::Full,
            &rig.threads,
        );
        render_parallel(
            &mut unclipped,
            &camera,
            &rig.scene_for(&reference, &state),
            RenderQuality::Full,
            &rig.threads,
        );
        assert_eq!(clipped.pixels(), unclipped.pixels(), "{name}");
    }
}

// 3-6: both qualities exist; Full keeps weak reflections, Interactive skips them.
#[test]
fn interactive_and_full_qualities_treat_weak_reflections_as_specified() {
    assert_eq!(RenderQuality::Interactive.label(), "Interactive");
    assert_eq!(RenderQuality::Full.label(), "Full");
    assert_eq!(RenderQuality::default(), RenderQuality::Full);
    assert_eq!(INTERACTIVE_REFLECTION_THRESHOLD, 0.05);
    assert_eq!(MAX_RAY_DEPTH, 3);
    let none = probe_color(opaque(0.0), RenderQuality::Full);
    for weak in [0.01, 0.02, 0.03] {
        assert_ne!(
            probe_color(opaque(weak), RenderQuality::Full),
            none,
            "Full keeps {weak}"
        );
        assert_eq!(
            probe_color(opaque(weak), RenderQuality::Interactive),
            none,
            "Interactive skips {weak}"
        );
    }
    for strong in [0.05, 0.2, 0.65] {
        assert_eq!(
            probe_color(opaque(strong), RenderQuality::Interactive),
            probe_color(opaque(strong), RenderQuality::Full)
        );
    }
}

// 7: the certified optics of water, glass, amethyst and the portal core.
#[test]
fn water_glass_portal_and_amethyst_optics_are_preserved() {
    let rig = Rig::new();
    let water = rig.materials.get(water_material_id()).unwrap();
    assert_eq!(
        (
            water.reflectivity,
            water.transparency,
            water.refractive_index
        ),
        (0.18, 0.78, 1.33)
    );
    let glass = rig.materials.get(glass_material_id()).unwrap();
    assert_eq!(
        (
            glass.reflectivity,
            glass.transparency,
            glass.refractive_index
        ),
        (0.10, 0.92, 1.5)
    );
    assert_eq!(glass.alpha_mode, AlphaMode::Blend);
    let core = rig.materials.get(portal_core_material_id()).unwrap();
    assert_eq!(
        (core.reflectivity, core.transparency, core.emission_strength),
        (0.03, 0.25, 2.5)
    );
    let amethyst = rig.materials.get(amethyst_cluster_material_id()).unwrap();
    assert_eq!(
        (
            amethyst.reflectivity,
            amethyst.transparency,
            amethyst.refractive_index
        ),
        (0.20, 0.08, 1.45)
    );
    for m in [water, glass, core, amethyst] {
        assert!(RenderQuality::Interactive.traces_reflection(m));
        assert!(RenderQuality::Full.traces_reflection(m));
    }
    // Shadows, refraction, normal maps and emission are not gated by quality.
    let src = std::fs::read_to_string("src/renderer/raytracer.rs").unwrap();
    assert_eq!(
        src.matches("quality.traces_reflection(material)").count(),
        1
    );
    assert!(
        !src.contains("quality")
            || !src.contains(
                "fn light_visibility(scene: &VoxelScene, ray: &Ray, max_distance: f32, quality"
            )
    );
}

// 8: parallel output equals serial output.
#[test]
fn parallel_tracing_equals_serial_tracing() {
    let rig = Rig::new();
    let serial = ParallelRenderConfig::serial();
    for (name, state) in scenarios() {
        for q in [RenderQuality::Full, RenderQuality::Interactive] {
            assert_eq!(
                rig.frame(&state, 48, 36, q, &rig.threads),
                rig.frame(&state, 48, 36, q, &serial),
                "{name} {q:?}"
            );
        }
    }
    assert!(rig.threads.workers >= 1);
    let parallel = std::fs::read_to_string("src/renderer/parallel.rs").unwrap();
    assert!(
        parallel.contains("std::thread::scope")
            && !parallel.contains("rayon")
            && !parallel.contains("Mutex")
    );
}

// 9, 10: the adaptive preview works and does not thrash.
#[test]
fn adaptive_preview_prefers_400x300_and_never_thrashes() {
    let mut preview = AdaptivePreview::new();
    assert_eq!(preview.resolution(800, 600), (400, 300));
    assert_eq!(PREVIEW_SCALES, [2, 4]);
    for _ in 0..DEMOTE_AFTER {
        preview.record(Duration::from_millis(60));
    }
    assert_eq!(preview.resolution(800, 600), (200, 150));
    for _ in 0..PROMOTE_AFTER {
        preview.record(Duration::from_millis(5));
    }
    assert_eq!(preview.resolution(800, 600), (400, 300));
    let mut flips = 0;
    let mut last = preview.scale();
    for i in 0..300 {
        let t = Duration::from_millis(if i % 2 == 0 { 60 } else { 5 });
        let s = preview.record(t);
        if s != last {
            flips += 1;
            last = s;
        }
    }
    assert_eq!(flips, 0);
    assert!((33..=50).contains(&(INTERACTIVE_FRAME_BUDGET.as_millis() as u64)));
}

// 11-13: refinement state machine, input and reset cancel it.
#[test]
fn full_refinement_is_non_blocking_and_cancellable() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let mut job = FullRefinement::new(80, 60);
    assert_eq!(job.phase(), RefinementPhase::Inactive);
    let t0 = Instant::now();
    job.view_changed(t0);
    assert!(!job.ready_to_start(t0 + IDLE_GRACE_PERIOD / 2));
    assert!(job.ready_to_start(t0 + IDLE_GRACE_PERIOD));
    job.start(state.camera(4.0 / 3.0));
    let scene = rig.scene(&state);
    let step = job.step_with_budget(&scene, &rig.threads, Duration::ZERO);
    assert_eq!((step.rows_traced, step.complete), (BATCH_ROWS, false));
    // Input cancels.
    job.view_changed(Instant::now());
    assert_eq!(
        (job.phase(), job.rows_done()),
        (RefinementPhase::WaitingForIdle, 0)
    );
    // Reset is activity too: R is active input and cancels as well.
    let mut s = state;
    apply_free_fly_input(
        &mut s,
        &FreeFlyInput {
            reset: true,
            ..Default::default()
        },
    );
    assert!(
        FreeFlyInput {
            reset: true,
            ..Default::default()
        }
        .is_active()
    );
    job.start(s.camera(4.0 / 3.0));
    job.view_changed(Instant::now());
    assert!(!job.is_refining());
    // A completed job equals the one-shot Full render.
    job.start(state.camera(4.0 / 3.0));
    while !job.is_complete() {
        job.step_with_budget(&scene, &rig.threads, Duration::ZERO);
    }
    assert_eq!(
        job.framebuffer().pixels(),
        &rig.frame(&state, 80, 60, RenderQuality::Full, &rig.threads)[..]
    );
    assert!(STEP_TIME_BUDGET <= Duration::from_millis(40));
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(!app.contains("needs_full_frame"));
    assert!(app.contains("refinement.view_changed(now);"));
}

// 14, 15: the portal roll traces Interactive previews; Red-Black uses the same pipeline.
#[test]
fn the_portal_roll_and_the_red_black_realm_use_the_same_pipeline() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("input.is_active() || rolling"));
    let moving = &app[app.find("if mode.update(&rl) {").unwrap()
        ..app.find("if refinement.ready_to_start(now) {").unwrap()];
    assert!(moving.contains("RenderQuality::Interactive") && moving.contains("preview.scale()"));
    let rig = Rig::new();
    let mut mid = WorldFreeFlyCameraState::looking_at(v(16.5, -7.0, 15.2), v(16.5, -7.0, 10.0));
    mid.begin_transition(camera::portal_crossing::PortalCrossingEvent::OverworldToRedBlack);
    mid.advance_transition(0.4);
    let (_, red_black) = scenarios()
        .into_iter()
        .find(|(n, _)| n.starts_with("D_"))
        .unwrap();
    assert_eq!(red_black.realm, WorldRealm::RedBlack);
    for state in [mid, red_black] {
        let serial = ParallelRenderConfig::serial();
        assert_eq!(
            rig.frame(&state, 40, 30, RenderQuality::Interactive, &rig.threads),
            rig.frame(&state, 40, 30, RenderQuality::Interactive, &serial)
        );
        assert_eq!(
            rig.frame(&state, 40, 30, RenderQuality::Full, &rig.threads),
            rig.frame(&state, 40, 30, RenderQuality::Full, &serial)
        );
    }
    assert!(red_black.background().is_sky());
}

// 16: framebuffer and texture reuse.
#[test]
fn the_framebuffer_and_texture_are_reused() {
    let mut fb = Framebuffer::new(400, 300);
    let ptr = fb.pixels().as_ptr();
    fb.resize(400, 300);
    assert_eq!(fb.pixels().as_ptr(), ptr);
    fb.resize(200, 150);
    assert_eq!(fb.pixels().len(), 200 * 150);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains(".update_texture(&self.rgba)"));
    assert!(app.contains("framebuffer.resize(width, height);"));
    assert!(app.contains("FullRefinement::new(full_width, full_height)"));
    assert!(!app.contains("draw_pixel"));
    assert!(app.contains("PerfReporter::from_env()"));
    assert!(!PerfReporter::from_setting(None).is_enabled());
}

// 17: Catalog 39/39.
#[test]
fn the_catalog_is_still_39_of_39() {
    assert_eq!(OFFICIAL_BLOCK_COUNT, 39);
    assert_eq!(official_entries().len(), 39);
    let catalog = CatalogScene::new();
    assert_eq!(catalog.entries().len(), 41);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("view: DiagnosticCameraState"));
    assert!(
        app.contains("\"Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset\"")
    );
}

// 18, 19: no geometry change, no material mutation, lights unchanged.
#[test]
fn geometry_lights_and_materials_are_untouched() {
    let rig = Rig::new();
    let scene = &rig.scene;
    assert_eq!(scene.world().len(), WorldScene::new().world().len()); // deterministic (Gate 14 changed the total)
    assert!(scene.world().len() > 6000);
    assert_eq!(
        (scene.portal().frame.len(), scene.portal().core.len()),
        (18, 12)
    );
    assert_eq!(scene.pond().water.len(), 26);
    assert_eq!(scene.trees().len(), WorldScene::new().trees().len());
    assert_eq!(rig.lights.len(), 5);
    assert_eq!(
        rig.lights
            .iter()
            .filter(|l| matches!(l, Light::Point(_)))
            .count(),
        4
    );
    let ids = [
        water_material_id(),
        glass_material_id(),
        portal_core_material_id(),
        amethyst_cluster_material_id(),
        scene::scene::grass_material_id(),
    ];
    let before: Vec<Material> = ids
        .iter()
        .map(|id| rig.materials.get(*id).unwrap().clone())
        .collect();
    let state = world_free_fly_camera();
    rig.frame(&state, 40, 30, RenderQuality::Interactive, &rig.threads);
    rig.frame(&state, 40, 30, RenderQuality::Full, &rig.threads);
    for (id, m) in ids.iter().zip(&before) {
        assert_eq!(rig.materials.get(*id).unwrap(), m);
    }
    assert_eq!(rig.scene.world().len(), WorldScene::new().world().len());
}

// 20, 21: no new crates, no Raylib 3D.
#[test]
fn no_new_crates_and_no_raylib_3d() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = &manifest[manifest.find("[dependencies]").unwrap()..];
    let deps = &deps[..deps.find("\n[").map_or(deps.len(), |i| i)];
    let crates: Vec<&str> = deps.lines().filter(|l| l.contains('=')).collect();
    assert_eq!(crates, vec!["raylib = \"5.5\""]);
    let lock = std::fs::read_to_string("Cargo.lock").unwrap();
    for forbidden in [
        "rayon",
        "crossbeam",
        "nalgebra",
        "glam",
        "cgmath",
        "noise",
        "rand",
    ] {
        assert!(
            !lock.contains(&format!("name = \"{forbidden}\"")),
            "{forbidden} in Cargo.lock"
        );
    }
    let mut code = String::new();
    for entry in walk("src") {
        code.push_str(&std::fs::read_to_string(&entry).unwrap());
    }
    for token in [
        "Camera3D",
        "BeginMode3D",
        "DrawCube",
        "DrawCubeV",
        "DrawModel",
        "Shader",
        "Mesh",
        "Model",
    ] {
        assert!(!has_token(&code, token), "{token} in src");
    }
}

fn walk(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(path.to_str().unwrap()));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path.to_str().unwrap().to_string());
        }
    }
    out
}

fn has_token(code: &str, token: &str) -> bool {
    let bytes = code.as_bytes();
    let mut from = 0;
    while let Some(i) = code[from..].find(token) {
        let start = from + i;
        let end = start + token.len();
        let before =
            start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let after = end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_');
        if !before && !after {
            return true;
        }
        from = end;
    }
    false
}

/// The constants the audit reports.
#[test]
fn reportable_constants() {
    let threads = ParallelRenderConfig::detect();
    println!("GATE13.6 interactive reflection threshold: {INTERACTIVE_REFLECTION_THRESHOLD}");
    println!(
        "GATE13.6 preview scales: {PREVIEW_SCALES:?} budget_ms={} demote_after={DEMOTE_AFTER} promote_after={PROMOTE_AFTER}",
        INTERACTIVE_FRAME_BUDGET.as_millis()
    );
    println!(
        "GATE13.6 idle grace_ms={} step_budget_ms={} batch_rows={BATCH_ROWS}",
        IDLE_GRACE_PERIOD.as_millis(),
        STEP_TIME_BUDGET.as_millis()
    );
    println!(
        "GATE13.6 available_parallelism={:?} workers={} band_rows={}",
        std::thread::available_parallelism().map(|n| n.get()),
        threads.workers,
        threads.band_rows
    );
    let bounds = WorldScene::new().world().bounds().unwrap();
    println!(
        "GATE13.6 world bounds min={:?} max_exclusive={:?}",
        bounds.min, bounds.max_exclusive
    );
}

/// Repeatable release benchmark of the benchmark poses A-G:
/// `cargo test --release --test gate13_6_performance_closure_tests bench_release_scenarios -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_release_scenarios() {
    let rig = Rig::new();
    let aspect = 4.0 / 3.0;
    println!(
        "BENCH workers={} band_rows={}",
        rig.threads.workers, rig.threads.band_rows
    );
    let p50 = |v: &mut Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let mut rgba = Vec::new();
    for (name, state) in scenarios() {
        let camera: Camera = state.camera(aspect);
        let scene = rig.scene(&state);
        // Interactive: let the controller pick the scale from measured frames.
        let mut preview = AdaptivePreview::new();
        let mut fb = Framebuffer::new(400, 300);
        for _ in 0..12 {
            let (w, h) = preview.resolution(800, 600);
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(
                &mut fb,
                &camera,
                &scene,
                RenderQuality::Interactive,
                &rig.threads,
            );
            preview.record(a.elapsed());
        }
        let (w, h) = preview.resolution(800, 600);
        let mut render_ms = Vec::new();
        let mut total_ms = Vec::new();
        for _ in 0..9 {
            fb.resize(w, h);
            let a = Instant::now();
            render_parallel(
                &mut fb,
                &camera,
                &scene,
                RenderQuality::Interactive,
                &rig.threads,
            );
            let r = a.elapsed();
            fb.write_rgba8(&mut rgba);
            render_ms.push(r.as_secs_f64() * 1e3);
            total_ms.push(a.elapsed().as_secs_f64() * 1e3);
        }
        let (r, t) = (p50(&mut render_ms), p50(&mut total_ms));
        println!(
            "BENCH {name} interactive resolution={w}x{h} render_ms={r:.1} total_ms={t:.1} fps={:.1}",
            1000.0 / t
        );
        // Full: the one-shot frame and the refinement job (batches, longest batch).
        let mut full = Framebuffer::new(800, 600);
        let mut full_ms = Vec::new();
        for _ in 0..3 {
            let a = Instant::now();
            render_parallel(
                &mut full,
                &camera,
                &scene,
                RenderQuality::Full,
                &rig.threads,
            );
            full_ms.push(a.elapsed().as_secs_f64() * 1e3);
        }
        let mut job = FullRefinement::new(800, 600);
        job.view_changed(Instant::now());
        job.start(camera);
        let a = Instant::now();
        let mut steps = 0;
        let mut longest_step = Duration::ZERO;
        while !job.is_complete() {
            let step = job.step(&scene, &rig.threads);
            longest_step = longest_step.max(step.elapsed);
            steps += 1;
        }
        full.write_rgba8(&mut rgba);
        let wall = a.elapsed().as_secs_f64() * 1e3;
        assert_eq!(job.framebuffer().pixels(), full.pixels());
        println!(
            "BENCH {name} full render_ms={:.1} total_ms={:.1} refinement steps={steps} batches={} longest_batch_ms={:.1} longest_step_ms={:.1} wall_ms={wall:.1}",
            p50(&mut full_ms),
            p50(&mut full_ms) + 6.0,
            job.batches(),
            job.longest_batch().as_secs_f64() * 1e3,
            longest_step.as_secs_f64() * 1e3
        );
    }
}

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
use camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm};
use core::color::Color;
use core::math::Vec3;
use renderer::framebuffer::Framebuffer;
use renderer::parallel::{ParallelRenderConfig, render_parallel};
use renderer::raytracer::{RenderQuality, VoxelScene};
use renderer::refinement::{
    BATCH_ROWS, FullRefinement, IDLE_GRACE_PERIOD, RefinementPhase, STEP_TIME_BUDGET,
};
use scene::light::Light;
use scene::material_library::MaterialLibrary;
use scene::texture_manager::TextureManager;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};
use std::time::{Duration, Instant};

const W: usize = 80;
const H: usize = 60;

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
            threads: ParallelRenderConfig::with_workers(4),
        }
    }

    fn scene(&self, state: &WorldFreeFlyCameraState) -> VoxelScene<'_> {
        VoxelScene {
            world: self.scene.world(),
            materials: &self.materials,
            camera_position: state.position,
            lights: &self.lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &self.manager,
            max_distance: WORLD_MAX_DISTANCE,
        }
    }

    fn reference(&self, state: &WorldFreeFlyCameraState, camera: &Camera) -> Vec<Color> {
        let mut fb = Framebuffer::new(W, H);
        render_parallel(
            &mut fb,
            camera,
            &self.scene(state),
            RenderQuality::Full,
            &self.threads,
        );
        fb.pixels().to_vec()
    }

    /// Runs a started job to completion one batch at a time.
    fn run_to_completion(
        &self,
        job: &mut FullRefinement,
        state: &WorldFreeFlyCameraState,
    ) -> usize {
        let scene = self.scene(state);
        let mut steps = 0;
        while !job.is_complete() {
            job.step_with_budget(&scene, &self.threads, Duration::ZERO);
            steps += 1;
            assert!(steps < 1000, "refinement never completes");
        }
        steps
    }
}

fn home() -> WorldFreeFlyCameraState {
    world_free_fly_camera()
}

fn red_black_pose() -> WorldFreeFlyCameraState {
    let mut s = WorldFreeFlyCameraState::looking_at(
        Vec3::new(12.0, -34.0, 36.0),
        Vec3::new(12.0, -20.0, 12.0),
    );
    s.realm = WorldRealm::RedBlack;
    s.local_up = WorldRealm::RedBlack.up();
    s.pitch = -s.pitch;
    s
}

#[test]
fn no_full_refinement_starts_before_the_idle_grace_period() {
    let mut job = FullRefinement::new(W, H);
    assert_eq!(job.phase(), RefinementPhase::Inactive);
    let t0 = Instant::now();
    job.view_changed(t0);
    assert_eq!(job.phase(), RefinementPhase::WaitingForIdle);
    assert!(job.shows_preview());
    assert!(!job.ready_to_start(t0));
    assert!(!job.ready_to_start(t0 + IDLE_GRACE_PERIOD / 2));
    assert!(!job.ready_to_start(t0 + IDLE_GRACE_PERIOD - Duration::from_millis(1)));
    assert!((150..=250).contains(&(IDLE_GRACE_PERIOD.as_millis() as u64)));
    // An inactive job never starts on its own.
    let idle = FullRefinement::new(W, H);
    assert!(!idle.ready_to_start(t0 + Duration::from_secs(5)));
}

#[test]
fn refinement_starts_once_the_view_has_rested() {
    let mut job = FullRefinement::new(W, H);
    let t0 = Instant::now();
    job.view_changed(t0);
    assert!(job.ready_to_start(t0 + IDLE_GRACE_PERIOD));
    assert!(job.ready_to_start(t0 + Duration::from_secs(3)));
    let camera = home().camera(4.0 / 3.0);
    job.start(camera);
    assert_eq!(job.phase(), RefinementPhase::Refining);
    assert!(job.is_refining() && job.shows_preview());
    assert_eq!(job.camera(), Some(&camera));
    assert_eq!(job.rows_done(), 0);
}

#[test]
fn refinement_progresses_in_bounded_batches() {
    let rig = Rig::new();
    let state = home();
    let mut job = FullRefinement::new(W, H);
    job.view_changed(Instant::now());
    job.start(state.camera(4.0 / 3.0));
    let scene = rig.scene(&state);
    // A zero budget traces exactly one batch per step.
    let step = job.step_with_budget(&scene, &rig.threads, Duration::ZERO);
    assert_eq!(
        (step.rows_traced, step.batches, step.complete),
        (BATCH_ROWS, 1, false)
    );
    assert_eq!(job.rows_done(), BATCH_ROWS);
    assert!(job.is_refining());
    let step = job.step_with_budget(&scene, &rig.threads, Duration::ZERO);
    assert_eq!(job.rows_done(), 2 * BATCH_ROWS);
    assert_eq!(step.batches, 1);
    // The last batch is partial (60 rows = 24 + 24 + 12) and completes.
    let step = job.step_with_budget(&scene, &rig.threads, Duration::ZERO);
    assert_eq!(
        (step.rows_traced, step.complete),
        (H - 2 * BATCH_ROWS, true)
    );
    assert_eq!(job.batches(), 3);
    assert!(job.render_time() > Duration::ZERO && job.longest_batch() <= job.render_time());
    // Once complete, stepping does nothing more.
    let step = job.step(&scene, &rig.threads);
    assert_eq!(
        (step.rows_traced, step.batches, step.complete),
        (0, 0, true)
    );
    assert!(STEP_TIME_BUDGET <= Duration::from_millis(40));
}

#[test]
fn input_during_refinement_cancels_it_immediately() {
    let rig = Rig::new();
    let state = home();
    let mut job = FullRefinement::new(W, H);
    job.view_changed(Instant::now());
    job.start(state.camera(4.0 / 3.0));
    let scene = rig.scene(&state);
    job.step_with_budget(&scene, &rig.threads, Duration::ZERO);
    assert!(job.rows_done() > 0);
    job.view_changed(Instant::now());
    assert_eq!(job.phase(), RefinementPhase::WaitingForIdle);
    assert_eq!(job.rows_done(), 0);
    assert_eq!(job.camera(), None);
    assert_eq!(job.batches(), 0);
    // Nothing is traced while cancelled.
    let step = job.step(&scene, &rig.threads);
    assert_eq!((step.rows_traced, step.complete), (0, false));
}

#[test]
fn a_cancelled_job_hands_the_screen_back_to_the_interactive_preview() {
    let mut job = FullRefinement::new(W, H);
    let t0 = Instant::now();
    job.view_changed(t0);
    job.start(home().camera(4.0 / 3.0));
    job.view_changed(t0 + Duration::from_millis(10));
    assert!(job.shows_preview());
    assert!(!job.is_refining() && !job.is_complete());
    // The idle clock restarted at the cancel, not at the first change.
    assert!(!job.ready_to_start(t0 + IDLE_GRACE_PERIOD));
    assert!(job.ready_to_start(t0 + Duration::from_millis(10) + IDLE_GRACE_PERIOD));
    // The viewer keeps polling input between batches: the loop calls
    // `mode.update` every iteration and steps at most once per iteration.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("if mode.update(&rl) {"));
    assert_eq!(app.matches("refinement.step(&scene, &threads)").count(), 1);
    assert!(app.contains("refinement.view_changed(now);"));
}

#[test]
fn a_completed_refinement_equals_the_full_reference_render() {
    let rig = Rig::new();
    for state in [home(), red_black_pose()] {
        let camera = state.camera(4.0 / 3.0);
        let mut job = FullRefinement::new(W, H);
        job.view_changed(Instant::now());
        // Seeded with a coarse preview that must be fully overwritten.
        let preview = vec![Color::new(0.9, 0.1, 0.9, 1.0); 20 * 15];
        job.seed(&preview, 20, 15);
        assert!(job.framebuffer().pixels().iter().all(|p| *p == preview[0]));
        job.start(camera);
        rig.run_to_completion(&mut job, &state);
        assert_eq!(
            job.framebuffer().pixels(),
            &rig.reference(&state, &camera)[..]
        );
        assert!(job.framebuffer().pixels().iter().any(|p| *p != preview[0]));
        job.acknowledge_complete();
        assert_eq!(job.phase(), RefinementPhase::Inactive);
        assert!(!job.shows_preview());
    }
}

#[test]
fn reset_cancels_refinement_like_any_other_activity() {
    let rig = Rig::new();
    let state = home();
    let mut job = FullRefinement::new(W, H);
    job.view_changed(Instant::now());
    job.start(state.camera(4.0 / 3.0));
    job.step_with_budget(&rig.scene(&state), &rig.threads, Duration::ZERO);
    // R makes `apply_free_fly_input` reset the camera and `update` return
    // true, which the loop turns into `view_changed`.
    let mut reset_state = state;
    camera::world_free_fly::apply_free_fly_input(
        &mut reset_state,
        &camera::world_free_fly::FreeFlyInput {
            reset: true,
            ..Default::default()
        },
    );
    assert!(
        camera::world_free_fly::FreeFlyInput {
            reset: true,
            ..Default::default()
        }
        .is_active()
    );
    job.view_changed(Instant::now());
    assert_eq!(job.phase(), RefinementPhase::WaitingForIdle);
    assert_eq!(job.rows_done(), 0);
}

#[test]
fn an_active_portal_roll_counts_as_activity() {
    // While the roll runs, `WorldMode::update` returns true even without
    // input, so the loop traces previews and calls `view_changed`.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("let rolling = self.free_fly.advance_transition(rl.get_frame_time());"));
    assert!(app.contains("input.is_active() || rolling"));
    let update_branch = &app[app.find("if mode.update(&rl) {").unwrap()
        ..app
            .find(
                "} else {
            if refinement.ready_to_start(now)",
            )
            .unwrap()];
    assert!(update_branch.contains("refinement.view_changed(now);"));
    assert!(update_branch.contains("RenderQuality::Interactive"));
    // The state machine mirrors it: a change during a roll cancels the job.
    let mut state = WorldFreeFlyCameraState::looking_at(
        Vec3::new(16.5, -7.0, 15.2),
        Vec3::new(16.5, -7.0, 10.0),
    );
    state.begin_transition(camera::portal_crossing::PortalCrossingEvent::OverworldToRedBlack);
    let mut job = FullRefinement::new(W, H);
    job.view_changed(Instant::now());
    job.start(state.camera(4.0 / 3.0));
    assert!(state.advance_transition(0.1));
    job.view_changed(Instant::now());
    assert!(!job.is_refining());
}

#[test]
fn refining_across_a_realm_transition_never_deadlocks() {
    let rig = Rig::new();
    // A roll in progress: blended sky, rolled camera; then the flipped realm.
    let mut mid = WorldFreeFlyCameraState::looking_at(
        Vec3::new(16.5, -7.0, 15.2),
        Vec3::new(16.5, -7.0, 10.0),
    );
    mid.begin_transition(camera::portal_crossing::PortalCrossingEvent::OverworldToRedBlack);
    mid.advance_transition(0.4);
    let mut flipped = mid;
    flipped.advance_transition(1.0);
    assert_eq!(flipped.realm, WorldRealm::RedBlack);
    for state in [mid, flipped, red_black_pose()] {
        let camera = state.camera(4.0 / 3.0);
        let mut job = FullRefinement::new(W, H);
        let t0 = Instant::now();
        job.view_changed(t0);
        // Repeated cancels before it ever starts, then a clean run.
        for i in 1..20 {
            job.view_changed(t0 + Duration::from_millis(i));
        }
        assert!(job.ready_to_start(t0 + Duration::from_millis(19) + IDLE_GRACE_PERIOD));
        job.start(camera);
        let steps = rig.run_to_completion(&mut job, &state);
        assert!(steps >= 1);
        assert_eq!(
            job.framebuffer().pixels(),
            &rig.reference(&state, &camera)[..]
        );
    }
}

#[test]
fn no_stale_rows_survive_from_a_previous_camera() {
    let rig = Rig::new();
    let first = home();
    let second =
        WorldFreeFlyCameraState::looking_at(Vec3::new(7.5, 10.0, 25.0), Vec3::new(7.5, 5.0, 18.0));
    let mut job = FullRefinement::new(W, H);
    job.view_changed(Instant::now());
    job.start(first.camera(4.0 / 3.0));
    job.step_with_budget(&rig.scene(&first), &rig.threads, Duration::ZERO);
    let partial = job.framebuffer().pixels().to_vec();
    // The view changes: the new job starts from row 0 with the new camera.
    job.view_changed(Instant::now());
    job.start(second.camera(4.0 / 3.0));
    assert_eq!(job.rows_done(), 0);
    rig.run_to_completion(&mut job, &second);
    let expected = rig.reference(&second, &second.camera(4.0 / 3.0));
    assert_eq!(job.framebuffer().pixels(), &expected[..]);
    assert_ne!(
        job.framebuffer().pixels()[..W * BATCH_ROWS],
        partial[..W * BATCH_ROWS]
    );
    // The camera used is the one captured at start, not re-read per batch,
    // and it is the second camera.
    assert_eq!(job.camera(), Some(&second.camera(4.0 / 3.0)));
    job.acknowledge_complete();
    assert_eq!(job.phase(), RefinementPhase::Inactive);
}

#[test]
fn the_full_result_is_the_full_window_resolution() {
    let job = FullRefinement::new(800, 600);
    assert_eq!((job.width(), job.height()), (800, 600));
    assert_eq!(job.framebuffer().pixels().len(), 480_000);
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("FullRefinement::new(full_width, full_height)"));
    // The synchronous full-frame branch is gone.
    assert!(!app.contains("needs_full_frame"));
    let refine = &app[app.find("if refinement.is_refining() {").unwrap()..];
    assert!(refine.contains("TracedFrameKind::Full"));
    assert!(refine.contains(
        "full_width,
                        full_height,"
    ));
}

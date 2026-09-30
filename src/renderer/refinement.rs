//! Cancellable, non-blocking refinement of the Full-quality frame.
//!
//! When the view stops changing, the viewer no longer traces the whole
//! 800x600 Full frame in one blocking call. Instead this job waits for a
//! short idle grace period (so a pause between two mouse movements does not
//! start it), then traces the frame a bounded batch of rows per event-loop
//! iteration, returning control to the loop (and its input polling) between
//! batches. Any activity cancels the job at once; the preview stays on
//! screen until a Full result is complete. A completed frame is exactly the
//! frame `render_parallel` would have traced in one go: the same camera,
//! captured when the job started, and the same per-pixel trace.
//!
//! Progress is shown as it happens: the frame starts as the last preview
//! stretched to full size, and every finished batch replaces its rows.

// `seed` and the accessors are the test-facing API of the job.
#![allow(dead_code)]

use std::time::{Duration, Instant};

use crate::camera::camera::Camera;
use crate::core::color::Color;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::parallel::{ParallelRenderConfig, render_rows_parallel};
use crate::renderer::raytracer::{RenderQuality, VoxelScene};

/// How long the view must rest before Full refinement begins.
pub const IDLE_GRACE_PERIOD: Duration = Duration::from_millis(200);

/// Time budget of one `step`: rows are traced in batches until this much
/// time has elapsed, then the loop gets control back. At most one batch
/// (`BATCH_ROWS` rows) can overshoot it.
pub const STEP_TIME_BUDGET: Duration = Duration::from_millis(25);

/// Rows traced per batch (a multiple of the parallel band size so every
/// worker gets a share of each batch).
pub const BATCH_ROWS: usize = 24;

/// Where the job is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefinementPhase {
    /// The current Full frame is up to date; nothing to do.
    Inactive,
    /// The view changed; waiting for `IDLE_GRACE_PERIOD` of rest.
    WaitingForIdle,
    /// Tracing the Full frame batch by batch.
    Refining,
    /// Every row is traced; the frame is ready to present.
    Complete,
}

/// What one `step` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefinementStep {
    pub rows_traced: usize,
    pub batches: usize,
    pub elapsed: Duration,
    pub complete: bool,
}

/// The Full-frame refinement job (see the module docs).
pub struct FullRefinement {
    phase: RefinementPhase,
    framebuffer: Framebuffer,
    camera: Option<Camera>,
    idle_since: Option<Instant>,
    next_row: usize,
    batches: usize,
    render_time: Duration,
    longest_batch: Duration,
}

impl FullRefinement {
    /// A job for a `width x height` Full frame, initially `Inactive`.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            phase: RefinementPhase::Inactive,
            framebuffer: Framebuffer::new(width, height),
            camera: None,
            idle_since: None,
            next_row: 0,
            batches: 0,
            render_time: Duration::ZERO,
            longest_batch: Duration::ZERO,
        }
    }

    pub fn phase(&self) -> RefinementPhase {
        self.phase
    }

    pub fn is_refining(&self) -> bool {
        self.phase == RefinementPhase::Refining
    }

    pub fn is_complete(&self) -> bool {
        self.phase == RefinementPhase::Complete
    }

    /// Whether the viewer should keep showing the Interactive preview
    /// (anything but an up-to-date Full frame).
    pub fn shows_preview(&self) -> bool {
        matches!(
            self.phase,
            RefinementPhase::WaitingForIdle | RefinementPhase::Refining
        )
    }

    pub fn width(&self) -> usize {
        self.framebuffer.width()
    }

    pub fn height(&self) -> usize {
        self.framebuffer.height()
    }

    /// Rows already traced by the current job.
    pub fn rows_done(&self) -> usize {
        self.next_row
    }

    /// The frame being refined (or the completed frame).
    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    /// The camera the job is tracing with, once started.
    pub fn camera(&self) -> Option<&Camera> {
        self.camera.as_ref()
    }

    /// Batches traced by the current job so far.
    pub fn batches(&self) -> usize {
        self.batches
    }

    /// CPU render time accumulated by the current job so far.
    pub fn render_time(&self) -> Duration {
        self.render_time
    }

    /// The longest single batch of the current job (the most the event
    /// loop was blocked by one `step` beyond its budget).
    pub fn longest_batch(&self) -> Duration {
        self.longest_batch
    }

    /// The view changed at `now` (camera input, reset, or an active portal
    /// roll): any refinement in progress is discarded and the idle clock
    /// restarts.
    pub fn view_changed(&mut self, now: Instant) {
        self.phase = RefinementPhase::WaitingForIdle;
        self.idle_since = Some(now);
        self.camera = None;
        self.next_row = 0;
        self.batches = 0;
        self.render_time = Duration::ZERO;
        self.longest_batch = Duration::ZERO;
    }

    /// Whether the grace period has elapsed since the last change.
    pub fn ready_to_start(&self, now: Instant) -> bool {
        self.phase == RefinementPhase::WaitingForIdle
            && self
                .idle_since
                .is_some_and(|since| now.duration_since(since) >= IDLE_GRACE_PERIOD)
    }

    /// Begins refining with `camera` (captured for the whole job).
    pub fn start(&mut self, camera: Camera) {
        self.phase = RefinementPhase::Refining;
        self.camera = Some(camera);
        self.next_row = 0;
        self.batches = 0;
        self.render_time = Duration::ZERO;
        self.longest_batch = Duration::ZERO;
    }

    /// Fills the frame with `preview` (a `width x height` image) stretched
    /// by nearest sampling, so progress can be shown over the last preview
    /// while rows are still being traced.
    pub fn seed(&mut self, preview: &[Color], width: usize, height: usize) {
        let (full_width, full_height) = (self.framebuffer.width(), self.framebuffer.height());
        if width == 0 || height == 0 || preview.len() != width * height {
            return;
        }
        let pixels = self.framebuffer.pixels_mut();
        for y in 0..full_height {
            let sy = (y * height / full_height).min(height - 1);
            for x in 0..full_width {
                let sx = (x * width / full_width).min(width - 1);
                pixels[y * full_width + x] = preview[sy * width + sx];
            }
        }
    }

    /// Traces batches of rows of the Full frame until `STEP_TIME_BUDGET`
    /// has elapsed (or the frame is complete). Does nothing unless refining.
    pub fn step(&mut self, scene: &VoxelScene, config: &ParallelRenderConfig) -> RefinementStep {
        self.step_with_budget(scene, config, STEP_TIME_BUDGET)
    }

    /// `step` with an explicit time budget (`Duration::ZERO` traces exactly
    /// one batch).
    pub fn step_with_budget(
        &mut self,
        scene: &VoxelScene,
        config: &ParallelRenderConfig,
        budget: Duration,
    ) -> RefinementStep {
        let started = Instant::now();
        let mut traced = 0;
        let mut batches = 0;
        let (Some(camera), RefinementPhase::Refining) = (self.camera, self.phase) else {
            return RefinementStep {
                rows_traced: 0,
                batches: 0,
                elapsed: Duration::ZERO,
                complete: self.phase == RefinementPhase::Complete,
            };
        };
        let (width, height) = (self.framebuffer.width(), self.framebuffer.height());

        while self.next_row < height {
            let batch_started = Instant::now();
            let rows = self.next_row..(self.next_row + BATCH_ROWS).min(height);
            let pixels = &mut self.framebuffer.pixels_mut()[rows.start * width..rows.end * width];
            render_rows_parallel(
                pixels,
                rows.clone(),
                width,
                height,
                &camera,
                scene,
                RenderQuality::Full,
                config,
            );
            let batch_time = batch_started.elapsed();
            self.longest_batch = self.longest_batch.max(batch_time);
            self.render_time += batch_time;
            self.next_row = rows.end;
            traced += rows.len();
            batches += 1;
            self.batches += 1;
            if started.elapsed() >= budget {
                break;
            }
        }

        if self.next_row >= height {
            self.phase = RefinementPhase::Complete;
        }
        RefinementStep {
            rows_traced: traced,
            batches,
            elapsed: started.elapsed(),
            complete: self.phase == RefinementPhase::Complete,
        }
    }

    /// The completed frame has been presented: nothing is pending until
    /// the view changes again.
    pub fn acknowledge_complete(&mut self) {
        if self.phase == RefinementPhase::Complete {
            self.phase = RefinementPhase::Inactive;
        }
    }
}

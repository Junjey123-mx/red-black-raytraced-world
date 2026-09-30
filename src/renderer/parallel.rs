//! Parallel CPU tracing of a frame with scoped standard-library threads.
//!
//! The framebuffer is split into horizontal bands of rows (full-width
//! tiles). Bands are dealt to the workers round-robin, every worker owns its
//! bands' pixels exclusively (`chunks_mut`), and all of them read the one
//! `VoxelScene` through shared references: nothing is cloned, nothing is
//! locked, no pixel is written twice. Each pixel is traced by exactly the
//! same `cast_ray_voxel_at` call the serial loop uses, so the parallel image
//! is identical to the serial one, bit for bit, for any worker count.

// `serial`, `with_band_rows` and `assign_bands` are the test-facing API of
// the band partition; the viewer only detects a config and renders.
#![allow(dead_code)]

use std::num::NonZeroUsize;

use crate::camera::camera::Camera;
use crate::camera::projection::primary_ray;
use crate::core::color::Color;
use crate::renderer::framebuffer::Framebuffer;
use crate::renderer::raytracer::{RenderQuality, VoxelScene, cast_ray_voxel_at};

/// Environment variable that overrides the detected worker count.
pub const THREADS_ENV_VAR: &str = "RBRW_THREADS";

/// Rows per band: small enough that round-robin dealing balances the
/// expensive and cheap regions of a frame across workers, large enough that
/// the per-band bookkeeping is negligible (600 rows = 150 bands).
pub const DEFAULT_BAND_ROWS: usize = 4;

/// How a frame is divided among threads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParallelRenderConfig {
    /// Upper bound on worker threads (the real count is also bounded by
    /// the number of bands). `1` traces on the calling thread.
    pub workers: usize,
    /// Rows per band.
    pub band_rows: usize,
}

impl ParallelRenderConfig {
    /// Tracing on the calling thread only.
    pub fn serial() -> Self {
        Self {
            workers: 1,
            band_rows: DEFAULT_BAND_ROWS,
        }
    }

    pub fn with_workers(workers: usize) -> Self {
        Self {
            workers: workers.max(1),
            band_rows: DEFAULT_BAND_ROWS,
        }
    }

    pub fn with_band_rows(self, band_rows: usize) -> Self {
        Self {
            band_rows: band_rows.max(1),
            ..self
        }
    }

    /// One worker per available hardware thread, unless `RBRW_THREADS`
    /// names a positive count.
    pub fn detect() -> Self {
        let available = std::thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1);
        let requested = std::env::var(THREADS_ENV_VAR)
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|&n| n > 0);
        Self::with_workers(requested.unwrap_or(available))
    }

    /// Workers that will actually run for a frame of `height` rows: never
    /// more than there are bands.
    pub fn effective_workers(&self, height: usize) -> usize {
        self.workers
            .max(1)
            .min(band_count(height, self.band_rows).max(1))
    }
}

/// Number of bands a frame of `height` rows is cut into.
pub fn band_count(height: usize, band_rows: usize) -> usize {
    height.div_ceil(band_rows.max(1))
}

/// The bands (as `start_row..end_row`) dealt to each worker, round-robin:
/// band `i` goes to worker `i % workers`. Together the bands of all workers
/// cover every row exactly once.
pub fn assign_bands(height: usize, band_rows: usize, workers: usize) -> Vec<Vec<(usize, usize)>> {
    let band_rows = band_rows.max(1);
    let workers = workers.max(1).min(band_count(height, band_rows).max(1));
    let mut assignment = vec![Vec::new(); workers];
    for (i, start) in (0..height).step_by(band_rows).enumerate() {
        assignment[i % workers].push((start, (start + band_rows).min(height)));
    }
    assignment
}

/// Traces the rows `rows` of a `width`-wide frame into `pixels` (exactly
/// `rows.len() * width` colors, row-major).
fn trace_rows(
    pixels: &mut [Color],
    rows: std::ops::Range<usize>,
    width: usize,
    height: usize,
    camera: &Camera,
    scene: &VoxelScene,
    quality: RenderQuality,
) {
    debug_assert_eq!(pixels.len(), rows.len() * width);
    for (row_index, y) in rows.enumerate() {
        let row = &mut pixels[row_index * width..(row_index + 1) * width];
        for (x, pixel) in row.iter_mut().enumerate() {
            let ray = primary_ray(camera, x, y, width, height);
            *pixel = cast_ray_voxel_at(scene, &ray, quality);
        }
    }
}

/// Traces every pixel of `framebuffer` from `camera` through `scene` at
/// `quality`, on `config.effective_workers` scoped threads (or inline for a
/// single worker). The scene, materials, textures, lights, camera and
/// background are shared immutably; each worker writes only its own bands.
pub fn render_parallel(
    framebuffer: &mut Framebuffer,
    camera: &Camera,
    scene: &VoxelScene,
    quality: RenderQuality,
    config: &ParallelRenderConfig,
) {
    let width = framebuffer.width();
    let height = framebuffer.height();
    let pixels = framebuffer.pixels_mut();
    render_rows_parallel(
        pixels,
        0..height,
        width,
        height,
        camera,
        scene,
        quality,
        config,
    );
}

/// Traces the rows `rows` of a `width x height` frame into `pixels` (the
/// storage of exactly those rows, row-major) with the same band dealing as
/// `render_parallel`. This is how a Full frame is refined a batch of rows
/// at a time without changing a single pixel's value.
#[allow(clippy::too_many_arguments)]
pub fn render_rows_parallel(
    pixels: &mut [Color],
    rows: std::ops::Range<usize>,
    width: usize,
    height: usize,
    camera: &Camera,
    scene: &VoxelScene,
    quality: RenderQuality,
    config: &ParallelRenderConfig,
) {
    let row_count = rows.len();
    assert_eq!(
        pixels.len(),
        row_count * width,
        "pixel storage must match the rows"
    );
    if width == 0 || row_count == 0 {
        return;
    }
    let band_rows = config.band_rows.max(1);
    let workers = config.effective_workers(row_count);

    if workers <= 1 {
        trace_rows(pixels, rows, width, height, camera, scene, quality);
        return;
    }

    // Deal the bands round-robin: worker `w` gets bands w, w + workers, ...
    let mut per_worker: Vec<Vec<(usize, &mut [Color])>> =
        (0..workers).map(|_| Vec::new()).collect();
    for (i, band) in pixels.chunks_mut(band_rows * width).enumerate() {
        per_worker[i % workers].push((rows.start + i * band_rows, band));
    }

    std::thread::scope(|scope| {
        for bands in per_worker {
            scope.spawn(move || {
                for (start_row, band) in bands {
                    let band_range = start_row..(start_row + band.len() / width);
                    trace_rows(band, band_range, width, height, camera, scene, quality);
                }
            });
        }
    });
}

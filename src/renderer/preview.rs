//! Adaptive preview resolution for interactive frames.
//!
//! While the view changes, frames are traced at a reduced scale and
//! stretched to the window. The controller prefers the sharper `x2` scale
//! (400x300 for an 800x600 window) and falls back to `x4` (200x150) only
//! when recent Interactive render times exceed the frame budget. Both
//! directions need several consecutive frames of evidence, so the scale
//! cannot flip every frame. Full-quality frames ignore it entirely.

// `reset` and the streak accessors are the test-facing API.
#![allow(dead_code)]

use std::time::Duration;

/// The preview scales the controller chooses between, sharpest first.
pub const PREVIEW_SCALES: [usize; 2] = [2, 4];

/// Time an Interactive render may take per frame: 40 ms is 25 frames per
/// second, the middle of the 33-50 ms (20-30 FPS) target.
pub const INTERACTIVE_FRAME_BUDGET: Duration = Duration::from_millis(40);

/// Consecutive over-budget frames at the sharper scale before dropping to
/// the coarser one.
pub const DEMOTE_AFTER: usize = 3;

/// Consecutive frames at the coarser scale whose predicted sharper-scale
/// cost fits the budget (with `PROMOTE_MARGIN`) before going back up.
pub const PROMOTE_AFTER: usize = 8;

/// Fraction of the budget the predicted sharper-scale cost must fit in
/// before promoting (a promotion that immediately fails is wasted work).
pub const PROMOTE_MARGIN: f64 = 0.8;

/// Picks the preview scale from measured Interactive render times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptivePreview {
    scale: usize,
    over_budget_streak: usize,
    under_budget_streak: usize,
}

impl Default for AdaptivePreview {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptivePreview {
    /// Starts at the sharper scale.
    pub fn new() -> Self {
        Self {
            scale: PREVIEW_SCALES[0],
            over_budget_streak: 0,
            under_budget_streak: 0,
        }
    }

    /// Current preview downscale factor (one of `PREVIEW_SCALES`).
    pub fn scale(&self) -> usize {
        self.scale
    }

    pub fn min_scale() -> usize {
        PREVIEW_SCALES[0]
    }

    pub fn max_scale() -> usize {
        PREVIEW_SCALES[PREVIEW_SCALES.len() - 1]
    }

    /// The preview resolution for a `full_width x full_height` window.
    pub fn resolution(&self, full_width: usize, full_height: usize) -> (usize, usize) {
        (
            (full_width / self.scale).max(1),
            (full_height / self.scale).max(1),
        )
    }

    pub fn over_budget_streak(&self) -> usize {
        self.over_budget_streak
    }

    pub fn under_budget_streak(&self) -> usize {
        self.under_budget_streak
    }

    /// Back to the initial state (sharper scale, no history).
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Feeds the render time of an Interactive frame traced at the current
    /// scale and returns the scale to use for the next one.
    pub fn record(&mut self, render_time: Duration) -> usize {
        let budget = INTERACTIVE_FRAME_BUDGET.as_secs_f64();
        let measured = render_time.as_secs_f64();
        if self.scale == Self::min_scale() {
            if measured > budget {
                self.over_budget_streak += 1;
                if self.over_budget_streak >= DEMOTE_AFTER {
                    self.scale = Self::max_scale();
                    self.over_budget_streak = 0;
                    self.under_budget_streak = 0;
                }
            } else {
                self.over_budget_streak = 0;
            }
        } else {
            // Pixel count grows with the square of the scale ratio.
            let ratio = (self.scale / Self::min_scale()) as f64;
            let predicted = measured * ratio * ratio;
            if predicted <= budget * PROMOTE_MARGIN {
                self.under_budget_streak += 1;
                if self.under_budget_streak >= PROMOTE_AFTER {
                    self.scale = Self::min_scale();
                    self.over_budget_streak = 0;
                    self.under_budget_streak = 0;
                }
            } else {
                self.under_budget_streak = 0;
            }
        }
        self.scale
    }
}

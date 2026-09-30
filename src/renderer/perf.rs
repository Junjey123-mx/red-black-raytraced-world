//! Opt-in, low-overhead timing of every traced frame: which quality was
//! traced (Interactive preview or Full), at what resolution, how long the
//! CPU render took, how long presentation (image conversion, texture upload
//! and the draw call) took, the total, and the equivalent frame rate.
//!
//! Nothing here runs per pixel. Reporting is disabled unless the
//! `RBRW_PERF` environment variable enables it, and an enabled reporter only
//! prints one line per traced frame to stderr, so the rendered image is the
//! same either way.

// The constructors and accessors below are the test-facing API of the
// stats; the viewer only builds and reports them.
#![allow(dead_code)]

use std::time::Duration;

/// The environment variable that turns performance reporting on.
pub const PERF_ENV_VAR: &str = "RBRW_PERF";

/// What kind of frame was traced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracedFrameKind {
    /// A reduced preview traced while the view is changing.
    Interactive,
    /// The full-resolution, full-quality frame traced once the view rests.
    Full,
}

impl TracedFrameKind {
    pub fn label(self) -> &'static str {
        match self {
            TracedFrameKind::Interactive => "Interactive",
            TracedFrameKind::Full => "Full",
        }
    }
}

/// Timing of the CPU render of one frame at a given resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderPerfStats {
    pub width: usize,
    pub height: usize,
    pub render: Duration,
}

impl RenderPerfStats {
    pub fn new(width: usize, height: usize, render: Duration) -> Self {
        Self {
            width,
            height,
            render,
        }
    }

    /// Primary rays traced (one per pixel).
    pub fn pixels(&self) -> usize {
        self.width * self.height
    }

    pub fn render_ms(&self) -> f64 {
        millis(self.render)
    }
}

/// Everything measured for one traced frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FramePerfStats {
    pub kind: TracedFrameKind,
    pub render: RenderPerfStats,
    /// Framebuffer-to-image conversion, GPU upload and the draw call.
    pub presentation: Duration,
}

impl FramePerfStats {
    pub fn new(
        kind: TracedFrameKind,
        width: usize,
        height: usize,
        render: Duration,
        presentation: Duration,
    ) -> Self {
        Self {
            kind,
            render: RenderPerfStats::new(width, height, render),
            presentation,
        }
    }

    pub fn width(&self) -> usize {
        self.render.width
    }

    pub fn height(&self) -> usize {
        self.render.height
    }

    /// Render plus presentation.
    pub fn total(&self) -> Duration {
        self.render.render + self.presentation
    }

    pub fn render_ms(&self) -> f64 {
        self.render.render_ms()
    }

    pub fn presentation_ms(&self) -> f64 {
        millis(self.presentation)
    }

    pub fn total_ms(&self) -> f64 {
        millis(self.total())
    }

    /// Frames per second this frame's total time would sustain; `0` for a
    /// zero-length frame (nothing meaningful to report).
    pub fn fps(&self) -> f64 {
        let total = self.total().as_secs_f64();
        if total > 0.0 { 1.0 / total } else { 0.0 }
    }

    /// The one-line report printed per traced frame.
    pub fn report_line(&self) -> String {
        format!(
            "{PERF_ENV_VAR} kind={} resolution={}x{} render_ms={:.2} presentation_ms={:.2} total_ms={:.2} fps={:.1}",
            self.kind.label(),
            self.width(),
            self.height(),
            self.render_ms(),
            self.presentation_ms(),
            self.total_ms(),
            self.fps()
        )
    }
}

/// Prints frame statistics when enabled; a no-op otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PerfReporter {
    enabled: bool,
}

impl PerfReporter {
    /// Enabled by `RBRW_PERF` (see `from_setting` for the accepted values).
    pub fn from_env() -> Self {
        Self::from_setting(std::env::var(PERF_ENV_VAR).ok().as_deref())
    }

    /// `None`, an empty value, `0`, `false` or `off` (any case) leave
    /// reporting disabled; every other value enables it.
    pub fn from_setting(value: Option<&str>) -> Self {
        let enabled = match value {
            None => false,
            Some(v) => !matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "" | "0" | "false" | "off" | "no"
            ),
        };
        Self { enabled }
    }

    pub fn enabled() -> Self {
        Self { enabled: true }
    }

    pub fn disabled() -> Self {
        Self { enabled: false }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Prints one line to stderr when enabled.
    pub fn report(&self, stats: &FramePerfStats) {
        if self.enabled {
            eprintln!("{}", stats.report_line());
        }
    }

    /// Prints an extra line (prefixed like `report`) when enabled.
    pub fn note(&self, message: &str) {
        if self.enabled {
            eprintln!("{PERF_ENV_VAR} {message}");
        }
    }
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

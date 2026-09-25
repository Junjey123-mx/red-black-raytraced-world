// Inspection-stage controls: turns already-polled input (mouse drag, wheel,
// keys) into changes of the `DiagnosticCameraState`. Nothing here talks to
// Raylib, so it can be tested without a window; `app.rs` only polls the
// devices and fills an `OrbitInput`.
#![allow(dead_code)]

use crate::camera::diagnostic::DiagnosticCameraState;

/// Radians of yaw/pitch per pixel of left-button mouse drag. Mouse-delta
/// based, so the feel does not depend on the frame rate.
pub const ORBIT_SENSITIVITY: f32 = 0.006;

/// Fraction of the current distance one wheel notch adds/removes. Zoom is
/// multiplicative, so it stays proportionate from close-ups to overviews.
pub const ZOOM_SENSITIVITY: f32 = 0.12;

/// Radians per second of the optional arrow-key orbit fallback.
pub const KEY_ORBIT_SPEED: f32 = 1.6;

/// Largest single-frame mouse delta honored, in pixels, so a lost-focus
/// spike cannot whip the camera around.
pub const MAX_DRAG_PIXELS_PER_FRAME: f32 = 240.0;

/// A single wheel report is limited to this many notches per frame, and the
/// resulting distance factor is kept inside a sane band.
const MAX_WHEEL_NOTCHES: f32 = 4.0;
const MIN_ZOOM_FACTOR: f32 = 0.2;
const MAX_ZOOM_FACTOR: f32 = 5.0;

/// One frame of polled input. All fields default to "nothing happened".
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct OrbitInput {
    /// Mouse movement this frame (pixels) while the left button is held;
    /// `(0, 0)` when it is not.
    pub drag_dx: f32,
    pub drag_dy: f32,
    /// Wheel notches this frame; positive zooms in.
    pub wheel: f32,
    /// Arrow-key orbit axes in `[-1, 1]`: `key_yaw` (right positive),
    /// `key_pitch` (up positive).
    pub key_yaw: f32,
    pub key_pitch: f32,
    /// Seconds since the previous frame (only used by the key fallback).
    pub delta_time: f32,
    /// `R`: restore the initial view.
    pub reset: bool,
}

fn sane(value: f32, limit: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-limit, limit)
    } else {
        0.0
    }
}

impl OrbitInput {
    /// `true` when this frame changes the camera at all (used to decide
    /// whether a re-render is needed).
    pub fn is_active(&self) -> bool {
        let moves = |v: f32| v.is_finite() && v != 0.0;
        self.reset
            || moves(self.drag_dx)
            || moves(self.drag_dy)
            || moves(self.wheel)
            || (moves(self.delta_time) && (moves(self.key_yaw) || moves(self.key_pitch)))
    }
}

/// Applies one frame of input to `state`.
///
/// - Dragging right turns the view right (the eye moves left around the
///   target); dragging down raises the eye.
/// - The wheel scales the distance by `1 - wheel * ZOOM_SENSITIVITY`.
/// - `reset` restores the initial pose and ignores the rest of the frame.
///
/// Orbiting never touches `target`, and zooming never touches `yaw`/`pitch`.
/// Pitch and distance stay inside the state's own limits, and non-finite
/// input is treated as no input.
pub fn apply_orbit_input(state: &mut DiagnosticCameraState, input: &OrbitInput) {
    if input.reset {
        state.reset();
        return;
    }

    let dx = sane(input.drag_dx, MAX_DRAG_PIXELS_PER_FRAME);
    let dy = sane(input.drag_dy, MAX_DRAG_PIXELS_PER_FRAME);
    let dt = sane(input.delta_time, 1.0).max(0.0);
    let key_yaw = sane(input.key_yaw, 1.0);
    let key_pitch = sane(input.key_pitch, 1.0);

    let yaw_delta = -dx * ORBIT_SENSITIVITY + key_yaw * KEY_ORBIT_SPEED * dt;
    let pitch_delta = dy * ORBIT_SENSITIVITY + key_pitch * KEY_ORBIT_SPEED * dt;
    if yaw_delta != 0.0 || pitch_delta != 0.0 {
        state.set_angles(state.yaw + yaw_delta, state.pitch + pitch_delta);
    }

    let wheel = sane(input.wheel, MAX_WHEEL_NOTCHES);
    if wheel != 0.0 {
        let factor = (1.0 - wheel * ZOOM_SENSITIVITY).clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
        state.set_distance(state.distance * factor);
    }
}

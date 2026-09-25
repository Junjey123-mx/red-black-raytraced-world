// Inspection-stage state: a project-owned orbit camera description used by
// the diagnostic catalog viewer. It only *describes* a camera; reading input
// is `controls.rs`'s job and rendering stays with the CPU raytracer.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::core::math::Vec3;

/// Pitch is kept strictly inside +-90 degrees so the view direction never
/// becomes parallel to the world up axis (no gimbal flip, no degenerate basis).
pub const MIN_PITCH: f32 = -80.0 * std::f32::consts::PI / 180.0;
pub const MAX_PITCH: f32 = 80.0 * std::f32::consts::PI / 180.0;

/// The eye can never reach the target (`> 0`) nor drift out of the scene.
pub const MIN_DISTANCE: f32 = 0.75;
pub const MAX_DISTANCE: f32 = 80.0;

/// Vertical field of view of the diagnostic camera, in degrees.
pub const DIAGNOSTIC_FOV_DEGREES: f32 = 60.0;

const TAU: f32 = std::f32::consts::TAU;
const PI: f32 = std::f32::consts::PI;

/// The four numbers that define an orbit view.
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrbitPose {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

/// Orbit camera around an explicit `target`.
///
/// Convention (right-handed, +Y up, the same axes as the world):
///
/// ```text
/// offset.x = distance * cos(pitch) * sin(yaw)
/// offset.y = distance * sin(pitch)
/// offset.z = distance * cos(pitch) * cos(yaw)
/// position = target + offset
/// ```
///
/// `yaw = 0` puts the eye on the target's `+Z` side looking toward `-Z`;
/// growing `yaw` swings the eye toward `+X`. Positive `pitch` raises the eye.
/// `yaw` is wrapped to `(-pi, pi]`; `pitch` and `distance` are clamped, and
/// non-finite inputs are ignored, so the state is always finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiagnosticCameraState {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    home: OrbitPose,
}

fn wrap_yaw(yaw: f32) -> f32 {
    let wrapped = yaw.rem_euclid(TAU);
    if wrapped > PI { wrapped - TAU } else { wrapped }
}

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

impl DiagnosticCameraState {
    /// A state that also becomes its own `reset` pose. Invalid (non-finite)
    /// components fall back to the default view; the rest is clamped.
    pub fn new(target: Vec3, yaw: f32, pitch: f32, distance: f32) -> Self {
        let fallback = Self::default();
        let pose = OrbitPose {
            target: if finite(target) {
                target
            } else {
                fallback.target
            },
            yaw: if yaw.is_finite() {
                wrap_yaw(yaw)
            } else {
                fallback.yaw
            },
            pitch: if pitch.is_finite() {
                pitch.clamp(MIN_PITCH, MAX_PITCH)
            } else {
                fallback.pitch
            },
            distance: if distance.is_finite() {
                distance.clamp(MIN_DISTANCE, MAX_DISTANCE)
            } else {
                fallback.distance
            },
        };

        Self {
            target: pose.target,
            yaw: pose.yaw,
            pitch: pose.pitch,
            distance: pose.distance,
            home: pose,
        }
    }

    /// Builds the orbit state that looks from `position` at `target` (the
    /// inverse of `position()`), used to start from an existing framing.
    pub fn from_pose(position: Vec3, target: Vec3) -> Self {
        let offset = position - target;
        let distance = offset.length();
        if !distance.is_finite() || distance <= f32::EPSILON {
            return Self::default();
        }

        let pitch = (offset.y / distance).clamp(-1.0, 1.0).asin();
        let yaw = offset.x.atan2(offset.z);
        Self::new(target, yaw, pitch, distance)
    }

    /// Restores exactly the pose this state was created with.
    pub fn reset(&mut self) {
        self.target = self.home.target;
        self.yaw = self.home.yaw;
        self.pitch = self.home.pitch;
        self.distance = self.home.distance;
    }

    /// Eye position derived from the orbit parameters.
    pub fn position(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();

        self.target
            + Vec3::new(
                self.distance * cos_pitch * sin_yaw,
                self.distance * sin_pitch,
                self.distance * cos_pitch * cos_yaw,
            )
    }

    /// Moves the orbit center. A non-finite target is ignored.
    pub fn set_target(&mut self, target: Vec3) {
        if finite(target) {
            self.target = target;
        }
    }

    /// Sets yaw (wrapped) and pitch (clamped); non-finite values are ignored.
    pub fn set_angles(&mut self, yaw: f32, pitch: f32) {
        if yaw.is_finite() {
            self.yaw = wrap_yaw(yaw);
        }
        if pitch.is_finite() {
            self.pitch = pitch.clamp(MIN_PITCH, MAX_PITCH);
        }
    }

    /// Sets the eye-to-target distance (clamped); non-finite is ignored.
    pub fn set_distance(&mut self, distance: f32) {
        if distance.is_finite() {
            self.distance = distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
        }
    }

    /// Re-applies every invariant (wrapped yaw, clamped pitch/distance).
    pub fn clamp(&mut self) {
        self.set_angles(self.yaw, self.pitch);
        self.set_distance(self.distance);
    }

    /// The raytracer's `Camera` for this view (world up = `+Y`).
    pub fn build_camera(&self, aspect_ratio: f32) -> Camera {
        Camera::new(
            self.position(),
            self.target,
            Vec3::new(0.0, 1.0, 0.0),
            DIAGNOSTIC_FOV_DEGREES,
            aspect_ratio,
        )
    }
}

impl Default for DiagnosticCameraState {
    /// A neutral inspection view: 20 degrees above the horizon, 12 units out,
    /// looking at the origin.
    fn default() -> Self {
        let pose = OrbitPose {
            target: Vec3::zero(),
            yaw: 0.0,
            pitch: 20.0 * PI / 180.0,
            distance: 12.0,
        };
        Self {
            target: pose.target,
            yaw: pose.yaw,
            pitch: pose.pitch,
            distance: pose.distance,
            home: pose,
        }
    }
}

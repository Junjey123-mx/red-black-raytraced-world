// World-only free-fly camera: a noclip, gravity-free filming camera with an
// explicit local frame. Its vertical axis (`local_up`) belongs to the realm
// the camera is in (Overworld: +Y, Red-Black: -Y), so the same yaw/pitch
// controls stay natural after the portal inverts the world. Nothing here
// touches Raylib or voxel geometry; `app.rs` only feeds it polled input.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::core::math::Vec3;

/// Vertical field of view of the free-fly camera, in degrees.
pub const FREE_FLY_FOV_DEGREES: f32 = 60.0;

/// Blocks per second of free flight: slow enough to appreciate detail.
pub const FREE_FLY_SPEED: f32 = 4.0;

/// Radians of yaw/pitch per pixel of mouse motion.
pub const FREE_FLY_MOUSE_SENSITIVITY: f32 = 0.0025;

/// Pitch limit (radians) either side of the local horizon, so `forward`
/// can never become collinear with `local_up`.
pub const FREE_FLY_MAX_PITCH: f32 = 89.0 * std::f32::consts::PI / 180.0;

/// The two worlds of the diorama, as the camera experiences them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldRealm {
    Overworld,
    RedBlack,
}

impl WorldRealm {
    /// The realm's functional up: +Y above the portal, -Y below it.
    pub fn up(self) -> Vec3 {
        match self {
            WorldRealm::Overworld => Vec3::new(0.0, 1.0, 0.0),
            WorldRealm::RedBlack => Vec3::new(0.0, -1.0, 0.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WorldRealm::Overworld => "Overworld",
            WorldRealm::RedBlack => "Red-Black",
        }
    }
}

/// A pose the camera can return to: position and look angles in the
/// Overworld frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FreeFlyPose {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

/// Horizontal heading for `yaw` (radians): `0` looks toward -Z, growing
/// yaw turns toward +X. Shared by both realms, so a realm change never
/// turns the camera around.
pub fn heading(yaw: f32) -> Vec3 {
    Vec3::new(yaw.sin(), 0.0, -yaw.cos())
}

/// Viewing direction for `yaw`/`pitch` in a frame whose vertical is `up`.
pub fn look_direction(yaw: f32, pitch: f32, up: Vec3) -> Vec3 {
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    (heading(yaw) * cos_pitch + up * sin_pitch).normalize()
}

/// The camera's right for a viewing direction and a vertical reference
/// (`forward x up`, the same rule `Camera::basis` uses).
pub fn right_of(forward: Vec3, up: Vec3) -> Vec3 {
    forward.cross(up).normalize()
}

/// The true up of a frame (perpendicular to `forward`, on the side of
/// `reference_up`).
pub fn frame_up(forward: Vec3, reference_up: Vec3) -> Vec3 {
    right_of(forward, reference_up).cross(forward).normalize()
}

/// Yaw and pitch that reproduce `forward` in a frame whose vertical is `up`.
pub fn angles_for(forward: Vec3, up: Vec3) -> (f32, f32) {
    let f = forward.normalize();
    let pitch = f.dot(up).clamp(-1.0, 1.0).asin();
    let horizontal = (f - up * f.dot(up)).normalize();
    let yaw = if horizontal.length_squared() > 1e-8 {
        horizontal.x.atan2(-horizontal.z)
    } else {
        0.0
    };
    (yaw, pitch)
}

/// The World's free-fly camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldFreeFlyCameraState {
    pub position: Vec3,
    /// Heading angle in the local frame (see `heading`).
    pub yaw: f32,
    /// Elevation angle above the local horizon, clamped to
    /// `+-FREE_FLY_MAX_PITCH`.
    pub pitch: f32,
    /// The realm's vertical axis the controls are relative to.
    pub local_up: Vec3,
    pub realm: WorldRealm,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    home: FreeFlyPose,
    transition: Option<PortalTransitionState>,
}

impl WorldFreeFlyCameraState {
    /// A camera at `position` looking at `target`, in the Overworld, that
    /// also becomes its own reset pose.
    pub fn looking_at(position: Vec3, target: Vec3) -> Self {
        let realm = WorldRealm::Overworld;
        let (yaw, pitch) = angles_for(target - position, realm.up());
        Self::from_pose(FreeFlyPose {
            position,
            yaw,
            pitch: pitch.clamp(-FREE_FLY_MAX_PITCH, FREE_FLY_MAX_PITCH),
        })
    }

    /// A camera at `pose` in the Overworld; `pose` is the reset pose.
    pub fn from_pose(pose: FreeFlyPose) -> Self {
        Self {
            position: pose.position,
            yaw: pose.yaw,
            pitch: pose.pitch,
            local_up: WorldRealm::Overworld.up(),
            realm: WorldRealm::Overworld,
            movement_speed: FREE_FLY_SPEED,
            mouse_sensitivity: FREE_FLY_MOUSE_SENSITIVITY,
            home: pose,
            transition: None,
        }
    }

    pub fn home(&self) -> FreeFlyPose {
        self.home
    }

    /// Unit viewing direction.
    pub fn forward(&self) -> Vec3 {
        look_direction(self.yaw, self.pitch, self.local_up)
    }

    /// Unit right (`forward x up`), following the roll of a transition.
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up()).normalize()
    }

    /// Unit up of the view frame (perpendicular to `forward`), following
    /// the roll of a transition.
    pub fn up(&self) -> Vec3 {
        self.rolled_up()
    }

    /// The raytracer camera for this state.
    pub fn camera(&self, aspect_ratio: f32) -> Camera {
        Camera::new(
            self.position,
            self.position + self.forward(),
            self.up(),
            FREE_FLY_FOV_DEGREES,
            aspect_ratio,
        )
    }

    /// Restores everything: the reset pose, the Overworld realm, +Y and no
    /// transition in progress.
    pub fn reset(&mut self) {
        self.position = self.home.position;
        self.yaw = self.home.yaw;
        self.pitch = self.home.pitch;
        self.realm = WorldRealm::Overworld;
        self.local_up = WorldRealm::Overworld.up();
        self.transition = None;
    }
}

// ---------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------

/// Largest mouse delta honored per frame (pixels), so a focus spike cannot
/// whip the view around.
pub const FREE_FLY_MAX_LOOK_PIXELS: f32 = 240.0;

/// Largest frame time honored (seconds): a stall never teleports the camera.
pub const FREE_FLY_MAX_DELTA_TIME: f32 = 0.25;

/// One frame of polled free-fly input. Axes are in `[-1, 1]`; everything
/// defaults to "nothing happened".
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FreeFlyInput {
    /// W (+1) / S (-1): along `forward`.
    pub move_forward: f32,
    /// D (+1) / A (-1): along `right`.
    pub move_right: f32,
    /// Space (+1) / Left Shift (-1): along the local `up`.
    pub move_up: f32,
    /// Mouse motion this frame, in pixels (right and down positive).
    pub look_dx: f32,
    pub look_dy: f32,
    /// Seconds since the previous frame.
    pub delta_time: f32,
    /// `R`: restore everything.
    pub reset: bool,
}

fn sane(value: f32, limit: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-limit, limit)
    } else {
        0.0
    }
}

impl FreeFlyInput {
    /// `true` when this frame moves or turns the camera (or resets it).
    pub fn is_active(&self) -> bool {
        let moves = |v: f32| v.is_finite() && v != 0.0;
        let dt = self.delta_time.is_finite() && self.delta_time > 0.0;
        self.reset
            || moves(self.look_dx)
            || moves(self.look_dy)
            || (dt && (moves(self.move_forward) || moves(self.move_right) || moves(self.move_up)))
    }
}

fn wrap_yaw(yaw: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let wrapped = yaw.rem_euclid(TAU);
    if wrapped > PI { wrapped - TAU } else { wrapped }
}

impl WorldFreeFlyCameraState {
    /// Turns the view: `dx` pixels to the right yaw right, `dy` pixels down
    /// pitch down. Pitch is clamped so `forward` never reaches `local_up`.
    pub fn look(&mut self, dx: f32, dy: f32) {
        let dx = sane(dx, FREE_FLY_MAX_LOOK_PIXELS);
        let dy = sane(dy, FREE_FLY_MAX_LOOK_PIXELS);
        self.yaw = wrap_yaw(self.yaw + dx * self.mouse_sensitivity);
        self.pitch = (self.pitch - dy * self.mouse_sensitivity)
            .clamp(-FREE_FLY_MAX_PITCH, FREE_FLY_MAX_PITCH);
    }

    /// Flies along the current frame: `forward`, `right` and `up` axes in
    /// `[-1, 1]`, scaled by `movement_speed` blocks per second and
    /// `delta_time` seconds. Frame-rate independent by construction.
    pub fn fly(&mut self, forward: f32, right: f32, up: f32, delta_time: f32) {
        let dt = if delta_time.is_finite() {
            delta_time.clamp(0.0, FREE_FLY_MAX_DELTA_TIME)
        } else {
            0.0
        };
        let velocity = self.forward() * sane(forward, 1.0)
            + self.right() * sane(right, 1.0)
            + self.up() * sane(up, 1.0);
        self.position = self.position + velocity * (self.movement_speed * dt);
    }
}

/// Applies one frame of input: `reset` restores everything and ignores the
/// rest of the frame; otherwise the look comes first (so the movement uses
/// the direction the user is now facing) and then the flight.
pub fn apply_free_fly_input(state: &mut WorldFreeFlyCameraState, input: &FreeFlyInput) {
    if input.reset {
        state.reset();
        return;
    }
    state.look(input.look_dx, input.look_dy);
    state.fly(
        input.move_forward,
        input.move_right,
        input.move_up,
        input.delta_time,
    );
}

// ---------------------------------------------------------------------
// Portal transition: the 180-degree roll between realms
// ---------------------------------------------------------------------

use crate::camera::portal_crossing::PortalCrossingEvent;

/// Seconds the roll takes.
pub const PORTAL_TRANSITION_DURATION: f32 = 0.8;

/// Hermite smoothstep `t * t * (3 - 2t)`.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Rotates `v` (perpendicular to the unit `axis`) by `angle` radians around
/// it (Rodrigues, reduced to the perpendicular case).
pub fn rotate_around(v: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    v * cos + axis.cross(v) * sin
}

/// Rotates any `v` by `angle` radians around the unit `axis` (full
/// Rodrigues formula).
pub fn rotate_general(v: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    v * cos + axis.cross(v) * sin + axis * (axis.dot(v) * (1.0 - cos))
}

/// An in-progress traversal: which realm the camera leaves and enters and
/// how far the roll has come.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalTransitionState {
    pub from_realm: WorldRealm,
    pub to_realm: WorldRealm,
    pub elapsed: f32,
    pub duration: f32,
}

impl PortalTransitionState {
    pub fn start(event: PortalCrossingEvent) -> Self {
        Self {
            from_realm: event.from_realm(),
            to_realm: event.to_realm(),
            elapsed: 0.0,
            duration: PORTAL_TRANSITION_DURATION,
        }
    }

    /// Linear progress in `[0, 1]`.
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Eased progress in `[0, 1]`.
    pub fn eased(&self) -> f32 {
        smoothstep(self.progress())
    }

    /// Roll angle so far, in radians (`0` to `pi`).
    pub fn roll_angle(&self) -> f32 {
        std::f32::consts::PI * self.eased()
    }

    pub fn is_complete(&self) -> bool {
        self.progress() >= 1.0
    }

    /// Advances the clock (non-finite or negative steps count as zero; a
    /// stall simply completes the roll).
    pub fn advance(&mut self, delta_time: f32) {
        if delta_time.is_finite() && delta_time > 0.0 {
            self.elapsed += delta_time;
        }
    }
}

impl WorldFreeFlyCameraState {
    /// The active transition, if any.
    pub fn transition(&self) -> Option<&PortalTransitionState> {
        self.transition.as_ref()
    }

    /// Starts the roll for `event`. Returns `false` (and does nothing) while
    /// another transition is active or the event does not leave the current
    /// realm.
    pub fn begin_transition(&mut self, event: PortalCrossingEvent) -> bool {
        if self.transition.is_some() || event.from_realm() != self.realm {
            return false;
        }
        self.transition = Some(PortalTransitionState::start(event));
        true
    }

    /// Advances an active transition by `delta_time`; on completion the
    /// camera adopts the destination realm: `local_up` flips and the pitch
    /// is mirrored so `forward` is exactly preserved. Returns `true` while a
    /// transition was active this frame (the view keeps changing).
    pub fn advance_transition(&mut self, delta_time: f32) -> bool {
        let Some(transition) = self.transition.as_mut() else {
            return false;
        };
        transition.advance(delta_time);
        if transition.is_complete() {
            let to = transition.to_realm;
            self.transition = None;
            self.realm = to;
            self.local_up = to.up();
            self.pitch = -self.pitch;
        }
        true
    }

    /// The view frame's up, including the roll of an active transition:
    /// the frame up of the realm being left, rotated around `forward` by
    /// the eased angle; at the end of the roll it coincides with the
    /// destination realm's frame up.
    pub fn rolled_up(&self) -> Vec3 {
        let base = frame_up(self.forward(), self.local_up);
        match &self.transition {
            Some(t) => rotate_around(base, self.forward(), t.roll_angle()).normalize(),
            None => base,
        }
    }
}

// ---------------------------------------------------------------------
// Environment seen by the camera
// ---------------------------------------------------------------------

use crate::scene::environment::WorldEnvironmentProfile;

impl WorldRealm {
    /// The realm's environment profile.
    pub fn environment(self) -> WorldEnvironmentProfile {
        match self {
            WorldRealm::Overworld => WorldEnvironmentProfile::DAY,
            WorldRealm::RedBlack => WorldEnvironmentProfile::RED_BLACK,
        }
    }
}

impl WorldFreeFlyCameraState {
    /// The environment the camera currently sees: its realm's profile, or,
    /// during a transition, the blend of the realm being left toward the
    /// one being entered at the roll's eased progress.
    pub fn environment(&self) -> WorldEnvironmentProfile {
        match &self.transition {
            Some(t) => t
                .from_realm
                .environment()
                .blend(&t.to_realm.environment(), t.eased()),
            None => self.realm.environment(),
        }
    }

    /// The vertical the sky is hung along: the realm's up, or, during a
    /// transition, the realm-of-origin's up rotated continuously around the
    /// camera's horizontal axis by the roll angle, so the sky turns over
    /// with the view and arrives exactly at the destination realm's up.
    pub fn sky_up(&self) -> Vec3 {
        match &self.transition {
            Some(t) => {
                let from = t.from_realm.up();
                let mut axis = self.forward().cross(from);
                if axis.length_squared() < 1e-6 {
                    axis = Vec3::new(1.0, 0.0, 0.0);
                }
                rotate_general(from, axis.normalize(), t.roll_angle()).normalize()
            }
            None => self.local_up,
        }
    }

    /// The background the World traces against this frame.
    pub fn background(&self) -> crate::renderer::skybox::Background {
        self.environment().background(self.sky_up())
    }
}

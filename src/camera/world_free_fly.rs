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
        }
    }

    pub fn home(&self) -> FreeFlyPose {
        self.home
    }

    /// Unit viewing direction.
    pub fn forward(&self) -> Vec3 {
        look_direction(self.yaw, self.pitch, self.local_up)
    }

    /// Unit right (`forward x up`).
    pub fn right(&self) -> Vec3 {
        right_of(self.forward(), self.local_up)
    }

    /// Unit up of the view frame (perpendicular to `forward`).
    pub fn up(&self) -> Vec3 {
        frame_up(self.forward(), self.local_up)
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

    /// Restores everything: the reset pose, the Overworld realm and +Y.
    pub fn reset(&mut self) {
        self.position = self.home.position;
        self.yaw = self.home.yaw;
        self.pitch = self.home.pitch;
        self.realm = WorldRealm::Overworld;
        self.local_up = WorldRealm::Overworld.up();
    }
}

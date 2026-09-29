// Segment-versus-portal crossing detection for the free-fly camera: the
// realm changes only when the camera's movement segment pierces the portal
// membrane inside the frame's opening. Derived from the diorama's portal
// contract (`PortalAnchor`), never from a height or a coordinate threshold.
#![allow(dead_code)]

use crate::camera::world_free_fly::WorldRealm;
use crate::core::math::Vec3;
use crate::scene::orientation::Orientation;
use crate::scene::rhombus::PortalAnchor;

/// Shrink of the opening (blocks) applied to the crossing test, so a path
/// that grazes the frame's inner edge does not count as a traversal.
pub const PORTAL_CROSSING_EPSILON: f32 = 0.05;

/// Distance from the membrane plane (blocks) the camera must reach on
/// either side before the detector arms again after a crossing.
pub const PORTAL_REARM_DISTANCE: f32 = 1.5;

/// Where the portal membrane is and how big its opening is, in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalVolume {
    /// A point on the membrane plane (its center).
    pub plane_point: Vec3,
    /// Unit normal of the plane, pointing toward the side the portal faces.
    pub normal: Vec3,
    /// World-space extent of the opening along `x` and `y`.
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl PortalVolume {
    /// The volume of the diorama's portal: the membrane sits halfway through
    /// the frame's wall row and the opening is the frame's interior.
    pub fn from_anchor(anchor: &PortalAnchor) -> Self {
        let normal = anchor.facing.axis_direction().normalize();
        let min_x = (anchor.min_x + 1) as f32;
        let max_x = anchor.max_x as f32;
        let min_y = (anchor.min_y + 1) as f32;
        let max_y = anchor.max_y as f32;
        let plane_point = Vec3::new(
            (min_x + max_x) / 2.0,
            (min_y + max_y) / 2.0,
            anchor.wall_z as f32 + 0.5,
        );
        Self {
            plane_point,
            normal,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Signed distance of `p` from the membrane plane (positive on the
    /// side the portal faces).
    pub fn signed_distance(&self, p: Vec3) -> f32 {
        (p - self.plane_point).dot(self.normal)
    }

    /// `true` when `p`, projected onto the plane, lies inside the opening
    /// (shrunk by `PORTAL_CROSSING_EPSILON`).
    pub fn inside_aperture(&self, p: Vec3) -> bool {
        let e = PORTAL_CROSSING_EPSILON;
        p.x > self.min_x + e && p.x < self.max_x - e && p.y > self.min_y + e && p.y < self.max_y - e
    }

    /// The point where the segment `from -> to` pierces the plane, if it
    /// crosses from one side to the other (touching the plane or moving
    /// along it does not count).
    pub fn segment_intersection(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let d0 = self.signed_distance(from);
        let d1 = self.signed_distance(to);
        if !d0.is_finite() || !d1.is_finite() {
            return None;
        }
        // Strictly opposite sides: a zero distance is "touching", not crossing.
        if d0 == 0.0 || d1 == 0.0 || (d0 > 0.0) == (d1 > 0.0) {
            return None;
        }
        let t = d0 / (d0 - d1);
        Some(from + (to - from) * t)
    }
}

/// A traversal of the portal, named by the realm it leaves and the one it
/// enters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalCrossingEvent {
    OverworldToRedBlack,
    RedBlackToOverworld,
}

impl PortalCrossingEvent {
    pub fn from_realm(&self) -> WorldRealm {
        match self {
            PortalCrossingEvent::OverworldToRedBlack => WorldRealm::Overworld,
            PortalCrossingEvent::RedBlackToOverworld => WorldRealm::RedBlack,
        }
    }

    pub fn to_realm(&self) -> WorldRealm {
        match self {
            PortalCrossingEvent::OverworldToRedBlack => WorldRealm::RedBlack,
            PortalCrossingEvent::RedBlackToOverworld => WorldRealm::Overworld,
        }
    }
}

/// Pure crossing test: the segment pierces the plane inside the opening.
/// The event's direction is the realm the camera is currently in, so the
/// same membrane serves both trips.
pub fn detect_portal_crossing(
    previous: Vec3,
    current: Vec3,
    volume: &PortalVolume,
    realm: WorldRealm,
) -> Option<PortalCrossingEvent> {
    let hit = volume.segment_intersection(previous, current)?;
    if !volume.inside_aperture(hit) {
        return None;
    }
    Some(match realm {
        WorldRealm::Overworld => PortalCrossingEvent::OverworldToRedBlack,
        WorldRealm::RedBlack => PortalCrossingEvent::RedBlackToOverworld,
    })
}

/// The stateful detector: fires once per traversal and stays disarmed
/// until the camera moves `PORTAL_REARM_DISTANCE` away from the membrane
/// on either side, so hovering around the plane after a crossing cannot
/// re-trigger it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalCrossingDetector {
    pub volume: PortalVolume,
    armed: bool,
}

impl PortalCrossingDetector {
    pub fn new(volume: PortalVolume) -> Self {
        Self {
            volume,
            armed: true,
        }
    }

    pub fn is_armed(&self) -> bool {
        self.armed
    }

    /// Rearms immediately (used by the full reset).
    pub fn reset(&mut self) {
        self.armed = true;
    }

    /// Checks this frame's movement. A disarmed detector first looks
    /// whether the camera has left the membrane's neighborhood; only an
    /// armed detector can report a crossing, and reporting one disarms it.
    pub fn detect(
        &mut self,
        previous: Vec3,
        current: Vec3,
        realm: WorldRealm,
    ) -> Option<PortalCrossingEvent> {
        if !self.armed {
            if self.volume.signed_distance(current).abs() >= PORTAL_REARM_DISTANCE {
                self.armed = true;
            }
            return None;
        }
        let event = detect_portal_crossing(previous, current, &self.volume, realm)?;
        self.armed = false;
        Some(event)
    }
}

/// `true` when the portal faces along a horizontal axis (the only layouts
/// the membrane plane derivation supports).
pub fn faces_horizontally(anchor: &PortalAnchor) -> bool {
    !matches!(anchor.facing, Orientation::Up | Orientation::Down)
}

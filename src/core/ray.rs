use crate::core::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// `direction` is normalized on construction; a (near-)zero direction
    /// normalizes to the zero vector rather than producing NaN/Infinity.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Evaluates P(t) = origin + t * direction.
    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

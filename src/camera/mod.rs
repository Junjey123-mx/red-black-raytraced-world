// Camera is re-exported ahead of the projection/render code that will use
// it, so the bin target does not consume this name yet.
#![allow(unused_imports)]

pub mod camera;
pub mod diagnostic;
pub mod projection;

pub use camera::Camera;
pub use projection::primary_ray;

// Camera is re-exported ahead of the projection/render code that will use
// it, so the bin target does not consume this name yet.
#![allow(unused_imports)]

pub mod camera;

pub use camera::Camera;

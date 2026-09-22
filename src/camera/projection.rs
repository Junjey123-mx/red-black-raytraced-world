// Foundation-stage primitive: lands ahead of the scene-cast/render work
// that will consume primary rays.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::core::ray::Ray;

/// Maps a framebuffer pixel center to a project-owned primary `Ray`,
/// applying aspect-ratio correction and vertical FOV scaling on top of the
/// camera's orthonormal basis. `width`/`height` are clamped to at least 1
/// pixel to avoid division by zero.
pub fn primary_ray(
    camera: &Camera,
    pixel_x: usize,
    pixel_y: usize,
    width: usize,
    height: usize,
) -> Ray {
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;

    let ndc_x = (2.0 * (pixel_x as f32 + 0.5) / width) - 1.0;
    let ndc_y = 1.0 - (2.0 * (pixel_y as f32 + 0.5) / height);

    let half_fov = camera.fov.to_radians() * 0.5;
    let tan_half_fov = half_fov.tan();

    let screen_x = ndc_x * camera.aspect_ratio * tan_half_fov;
    let screen_y = ndc_y * tan_half_fov;

    let basis = camera.basis();
    let direction = basis.forward + basis.right * screen_x + basis.up * screen_y;

    Ray::new(camera.position, direction)
}

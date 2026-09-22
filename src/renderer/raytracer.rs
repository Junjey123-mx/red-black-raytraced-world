// Foundation-stage primitive: lands ahead of the full framebuffer render
// loop that will call `cast_ray`/`cast_ray_lit` once per pixel.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::cube::Cube;
use crate::core::material::Material;
use crate::core::math::Vec3;
use crate::core::ray::Ray;
use crate::renderer::shading;
use crate::renderer::shadows;
use crate::scene::light::Light;

/// Resolves a primary ray against a single diagnostic cube. This is
/// deliberately unlit: a hit produces a diagnostic color derived purely
/// from the surface normal, and a miss produces `background`. There is no
/// ambient/diffuse/specular shading, material, or light source involved.
pub fn cast_ray(cube: &Cube, ray: &Ray, background: Color) -> Color {
    match cube.intersect(ray, 0.0, f32::INFINITY) {
        Some(hit) => normal_to_diagnostic_color(hit.normal),
        None => background,
    }
}

/// Remaps each unit-normal component from `[-1, 1]` to `[0, 1]` so the six
/// cube faces render as six visually distinct, deterministic colors.
fn normal_to_diagnostic_color(normal: Vec3) -> Color {
    Color::new(
        (normal.x + 1.0) * 0.5,
        (normal.y + 1.0) * 0.5,
        (normal.z + 1.0) * 0.5,
        1.0,
    )
}

/// Resolves a primary ray against the nearest of a small explicit
/// collection of diagnostic `(Cube, Material)` objects, then evaluates full
/// local lighting: ambient always applies; each light's diffuse and
/// specular contribution is added only when an epsilon-offset shadow ray
/// toward that light finds no blocking geometry among `objects`. A miss
/// returns `background`. This is the minimal nearest-hit search needed for
/// this Gate's diagnostic scene — not a VoxelWorld, DDA, or scene graph.
pub fn cast_ray_lit(
    objects: &[(Cube, Material)],
    ray: &Ray,
    camera_position: Vec3,
    lights: &[Light],
    ambient_factor: f32,
    background: Color,
) -> Color {
    let mut nearest: Option<(f32, Vec3, Vec3, &Material)> = None;

    for (cube, material) in objects {
        if let Some(hit) = cube.intersect(ray, 0.0, f32::INFINITY) {
            let is_closer = nearest.as_ref().is_none_or(|(t, ..)| hit.distance < *t);
            if is_closer {
                nearest = Some((hit.distance, hit.point, hit.normal, material));
            }
        }
    }

    let Some((_, point, normal, material)) = nearest else {
        return background;
    };

    let view_direction = (camera_position - point).normalize();
    let cubes: Vec<&Cube> = objects.iter().map(|(cube, _)| cube).collect();

    let mut color = shading::ambient(material, ambient_factor);

    for light in lights {
        match light {
            Light::Directional(directional) => {
                let shadow_ray =
                    shadows::shadow_ray_to_directional_light(point, normal, directional);
                let blocked = shadows::is_occluded(
                    cubes.iter().copied(),
                    &shadow_ray,
                    shadows::DIRECTIONAL_SHADOW_RANGE,
                );
                if !blocked {
                    color = color + shading::diffuse_directional(material, normal, directional);
                    color = color
                        + shading::specular_directional(
                            material,
                            normal,
                            view_direction,
                            directional,
                        );
                }
            }
            Light::Point(point_light) => {
                let (shadow_ray, distance) =
                    shadows::shadow_ray_to_point_light(point, normal, point_light);
                let blocked = shadows::is_occluded(cubes.iter().copied(), &shadow_ray, distance);
                if !blocked {
                    color = color + shading::diffuse_point(material, normal, point, point_light);
                    color = color
                        + shading::specular_point(
                            material,
                            normal,
                            point,
                            view_direction,
                            point_light,
                        );
                }
            }
        }
    }

    color.clamp()
}

#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/aabb.rs"]
    pub mod aabb;
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/cube.rs"]
    pub mod cube;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
}

use core::color::Color;
use core::cube::Cube;
use core::hit::Face;
use core::math::{Vec2, Vec3};
use core::ray::Ray;
use core::texture::CpuTexture;
use renderer::texture_sampling::sample_nearest;

fn c(index: u32) -> Color {
    Color::new(index as f32, 0.0, 0.0, 1.0)
}

/// A 4x4 texture where every one of the 16 texels holds a distinct color
/// (`texel(x, y) == c(y * 4 + x)`). Deliberately not a symmetric
/// checkerboard: any horizontal mirror, vertical mirror, rotation, or wrong
/// face selection in the Ray -> Cube -> UV -> sample_nearest chain produces
/// a different (and therefore wrong) index than the one asserted here.
fn asymmetric_4x4() -> CpuTexture {
    let pixels: Vec<Color> = (0..16).map(c).collect();
    CpuTexture::new(4, 4, pixels).unwrap()
}

/// Resolves a real ray against a real cube, then feeds the real resulting
/// UV through the real nearest-neighbor sampler. Returns `(face, color)`.
fn trace(cube: &Cube, ray: &Ray, texture: &CpuTexture) -> (Face, Color) {
    let hit = cube.intersect(ray, 0.0, f32::INFINITY).unwrap();
    let color = sample_nearest(texture, hit.uv);
    (hit.face, color)
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

/// Expected texel index for a 4x4 texture given a normalized UV, using the
/// exact same floor/clamp policy as `sample_nearest`.
fn expected_index(u: f32, v: f32) -> u32 {
    let x = ((u * 4.0).floor() as i32).clamp(0, 3) as u32;
    let y = ((v * 4.0).floor() as i32).clamp(0, 3) as u32;
    y * 4 + x
}

// --- Six faces through the real Ray -> Cube -> UV -> sample chain ---

#[test]
fn positive_z_center_samples_expected_texel() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let (face, color) = trace(&cube, &ray, &texture);

    assert_eq!(face, Face::PositiveZ);
    assert_eq!(color, c(expected_index(0.5, 0.5)));
}

#[test]
fn positive_z_upper_left_quadrant_samples_expected_texel() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();
    let ray = Ray::new(Vec3::new(-0.5, 0.5, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let (face, color) = trace(&cube, &ray, &texture);

    assert_eq!(face, Face::PositiveZ);
    assert_eq!(color, c(expected_index(0.25, 0.25)));
}

#[test]
fn positive_z_lower_right_quadrant_samples_expected_texel() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();
    let ray = Ray::new(Vec3::new(0.5, -0.5, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let (face, color) = trace(&cube, &ray, &texture);

    assert_eq!(face, Face::PositiveZ);
    assert_eq!(color, c(expected_index(0.75, 0.75)));
}

#[test]
fn negative_z_center_and_orientation_sample_expected_texels() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();

    let center_ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let (center_face, center_color) = trace(&cube, &center_ray, &texture);
    assert_eq!(center_face, Face::NegativeZ);
    assert_eq!(center_color, c(expected_index(0.5, 0.5)));

    // Same (x, y) as the +Z upper-left quadrant case, opposite face: this
    // catches a horizontal-mirror bug between +Z and -Z.
    let mirrored_ray = Ray::new(Vec3::new(-0.5, 0.5, -5.0), Vec3::new(0.0, 0.0, 1.0));
    let (mirrored_face, mirrored_color) = trace(&cube, &mirrored_ray, &texture);
    assert_eq!(mirrored_face, Face::NegativeZ);
    assert_eq!(mirrored_color, c(expected_index(0.75, 0.25)));
}

#[test]
fn positive_x_center_and_orientation_sample_expected_texels() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();

    let center_ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let (center_face, center_color) = trace(&cube, &center_ray, &texture);
    assert_eq!(center_face, Face::PositiveX);
    assert_eq!(center_color, c(expected_index(0.5, 0.5)));

    let quadrant_ray = Ray::new(Vec3::new(5.0, 0.5, -0.5), Vec3::new(-1.0, 0.0, 0.0));
    let (quadrant_face, quadrant_color) = trace(&cube, &quadrant_ray, &texture);
    assert_eq!(quadrant_face, Face::PositiveX);
    assert_eq!(quadrant_color, c(expected_index(0.75, 0.25)));
}

#[test]
fn negative_x_center_and_orientation_sample_expected_texels() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();

    let center_ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
    let (center_face, center_color) = trace(&cube, &center_ray, &texture);
    assert_eq!(center_face, Face::NegativeX);
    assert_eq!(center_color, c(expected_index(0.5, 0.5)));

    // Same (y, z) as the +X quadrant case: catches a mirror bug between +X
    // and -X.
    let quadrant_ray = Ray::new(Vec3::new(-5.0, 0.5, -0.5), Vec3::new(1.0, 0.0, 0.0));
    let (quadrant_face, quadrant_color) = trace(&cube, &quadrant_ray, &texture);
    assert_eq!(quadrant_face, Face::NegativeX);
    assert_eq!(quadrant_color, c(expected_index(0.25, 0.25)));
}

#[test]
fn positive_y_center_and_orientation_sample_expected_texels() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();

    let center_ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
    let (center_face, center_color) = trace(&cube, &center_ray, &texture);
    assert_eq!(center_face, Face::PositiveY);
    assert_eq!(center_color, c(expected_index(0.5, 0.5)));

    let quadrant_ray = Ray::new(Vec3::new(-0.5, 5.0, 0.5), Vec3::new(0.0, -1.0, 0.0));
    let (quadrant_face, quadrant_color) = trace(&cube, &quadrant_ray, &texture);
    assert_eq!(quadrant_face, Face::PositiveY);
    assert_eq!(quadrant_color, c(expected_index(0.25, 0.75)));
}

#[test]
fn negative_y_center_and_orientation_sample_expected_texels() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();

    let center_ray = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
    let (center_face, center_color) = trace(&cube, &center_ray, &texture);
    assert_eq!(center_face, Face::NegativeY);
    assert_eq!(center_color, c(expected_index(0.5, 0.5)));

    // Same (x, z) as the +Y quadrant case: catches a vertical-inversion bug
    // between +Y (top) and -Y (bottom).
    let quadrant_ray = Ray::new(Vec3::new(-0.5, -5.0, 0.5), Vec3::new(0.0, 1.0, 0.0));
    let (quadrant_face, quadrant_color) = trace(&cube, &quadrant_ray, &texture);
    assert_eq!(quadrant_face, Face::NegativeY);
    assert_eq!(quadrant_color, c(expected_index(0.25, 0.25)));
}

// --- Cross-cutting integration contracts ---

#[test]
fn hit_from_inside_cube_samples_expected_texel() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();
    let ray = Ray::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0));

    let (face, color) = trace(&cube, &ray, &texture);

    assert_eq!(face, Face::PositiveZ);
    assert_eq!(color, c(expected_index(0.5, 0.5)));
}

#[test]
fn translated_cube_samples_same_texel_as_origin_cube() {
    let origin_cube = unit_cube();
    let translated_cube = Cube::new(Vec3::new(9.0, 19.0, -31.0), Vec3::new(11.0, 21.0, -29.0));
    let texture = asymmetric_4x4();

    let ray_origin = Ray::new(Vec3::new(0.3, 5.0, 0.2), Vec3::new(0.0, -1.0, 0.0));
    let ray_translated = Ray::new(Vec3::new(10.3, 25.0, -29.8), Vec3::new(0.0, -1.0, 0.0));

    let (face_origin, color_origin) = trace(&origin_cube, &ray_origin, &texture);
    let (face_translated, color_translated) = trace(&translated_cube, &ray_translated, &texture);

    assert_eq!(face_origin, face_translated);
    assert_eq!(color_origin, color_translated);
}

#[test]
fn uv_near_a_texel_border_still_resolves_in_bounds() {
    let cube = unit_cube();
    let texture = asymmetric_4x4();
    // x = 0.99 sits just inside the +Z face's right edge (not exactly on
    // the cube boundary, so the face selection stays unambiguous).
    let ray = Ray::new(Vec3::new(0.99, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let hit = cube.intersect(&ray, 0.0, f32::INFINITY).unwrap();
    assert_eq!(hit.face, Face::PositiveZ);
    assert!((0.0..=1.0).contains(&hit.uv.x));
    assert!((0.0..=1.0).contains(&hit.uv.y));

    let color = sample_nearest(&texture, hit.uv);
    assert_eq!(color, c(expected_index(hit.uv.x, hit.uv.y)));
}

#[test]
fn sampling_at_u_equals_one_selects_last_column() {
    let texture = asymmetric_4x4();
    let color = sample_nearest(&texture, Vec2::new(1.0, 0.0));
    assert_eq!(color, c(expected_index(1.0, 0.0)));
    assert_eq!(color, c(3));
}

#[test]
fn sampling_at_v_equals_one_selects_last_row() {
    let texture = asymmetric_4x4();
    let color = sample_nearest(&texture, Vec2::new(0.0, 1.0));
    assert_eq!(color, c(expected_index(0.0, 1.0)));
    assert_eq!(color, c(12));
}

#[test]
fn one_by_one_texture_always_resolves_through_the_sampler() {
    let texture = CpuTexture::new(1, 1, vec![c(42)]).unwrap();
    assert_eq!(sample_nearest(&texture, Vec2::new(0.0, 0.0)), c(42));
    assert_eq!(sample_nearest(&texture, Vec2::new(1.0, 1.0)), c(42));
    assert_eq!(sample_nearest(&texture, Vec2::new(0.5, 0.5)), c(42));
}

#[test]
fn rectangular_texture_resolves_through_the_sampler() {
    // 4 wide, 2 tall.
    let pixels: Vec<Color> = (0..8).map(c).collect();
    let texture = CpuTexture::new(4, 2, pixels).unwrap();

    assert_eq!(sample_nearest(&texture, Vec2::new(0.0, 0.0)), c(0));
    assert_eq!(sample_nearest(&texture, Vec2::new(1.0, 0.0)), c(3));
    assert_eq!(sample_nearest(&texture, Vec2::new(0.0, 1.0)), c(4));
    assert_eq!(sample_nearest(&texture, Vec2::new(1.0, 1.0)), c(7));
}

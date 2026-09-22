#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
        pub use vec2::Vec2;
        pub use vec3::Vec3;
    }

    #[path = "../src/core/color.rs"]
    pub mod color;

    #[path = "../src/core/ray.rs"]
    pub mod ray;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
}

use core::color::Color;
use core::math::{IVec3, Vec2, Vec3};
use core::ray::Ray;
use renderer::framebuffer::Framebuffer;
use std::collections::HashMap;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

/// Vec3 feeds a Ray, whose evaluated point stays finite and matches P(t).
#[test]
fn ray_built_from_vec3_evaluates_correctly() {
    let origin = Vec3::new(0.0, 1.0, 0.0);
    let direction = Vec3::new(0.0, 0.0, 2.0);
    let ray = Ray::new(origin, direction);

    assert!(approx_eq(ray.direction.length(), 1.0));

    let point = ray.at(3.0);
    assert!(point.x.is_finite() && point.y.is_finite() && point.z.is_finite());
    assert!(approx_eq(point.x, 0.0));
    assert!(approx_eq(point.y, 1.0));
    assert!(approx_eq(point.z, 3.0));
}

/// Vec2 drives a UV-style lookup that selects a Color, which is then written
/// into the Framebuffer and read back unchanged.
#[test]
fn vec2_uv_selects_color_written_to_framebuffer() {
    let uv = Vec2::new(0.75, 0.25).normalize();
    assert!(approx_eq(uv.length(), 1.0));

    let selected = if uv.x >= uv.y {
        Color::white()
    } else {
        Color::black()
    };

    let mut framebuffer = Framebuffer::new(2, 2);
    framebuffer.clear(Color::black());
    framebuffer.set_pixel(0, 0, selected);

    assert_eq!(framebuffer.get_pixel(0, 0), Some(selected));
    assert_eq!(framebuffer.get_pixel(1, 1), Some(Color::black()));
}

/// IVec3 keys a voxel-cell-like map to a Color, and that Color round-trips
/// through clamping and 8-bit conversion without producing NaN or overflow.
#[test]
fn ivec3_keyed_colors_convert_to_finite_rgba8() {
    let mut palette: HashMap<IVec3, Color> = HashMap::new();
    palette.insert(IVec3::new(-1, 0, 1), Color::new(1.5, -0.5, 0.5, 1.0));
    palette.insert(IVec3::zero(), Color::black());

    let cell = IVec3::new(-1, 0, 1);
    let color = *palette.get(&cell).expect("cell must be present");
    let rgba = color.to_rgba8();

    for channel in rgba {
        assert!((0..=255).contains(&(channel as i32)));
    }
    assert_eq!(rgba[0], 255);
    assert_eq!(rgba[1], 0);
}

/// The full chain composes: a deterministic pattern driven by Vec2 UVs and
/// IVec3 cell coordinates fills a Framebuffer whose pixel count matches its
/// declared dimensions and whose contents stay finite.
#[test]
fn full_foundation_chain_fills_framebuffer_deterministically() {
    let width = 4;
    let height = 4;
    let mut framebuffer = Framebuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let uv = Vec2::new(x as f32 / width as f32, y as f32 / height as f32);
            let cell = IVec3::new(x as i32, 0, y as i32);
            let is_bright = (cell.x + cell.z) % 2 == 0;
            let color = if is_bright {
                Color::new(uv.x, uv.y, 0.5, 1.0)
            } else {
                Color::black()
            };
            framebuffer.set_pixel(x, y, color);
        }
    }

    assert_eq!(framebuffer.pixels().len(), width * height);
    for pixel in framebuffer.pixels() {
        assert!(pixel.r.is_finite() && pixel.g.is_finite() && pixel.b.is_finite());
    }
}

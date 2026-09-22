#[path = "../src/core/color.rs"]
mod color;

use color::Color;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn black_and_white_constants() {
    let black = Color::black();
    assert!(approx_eq(black.r, 0.0));
    assert!(approx_eq(black.g, 0.0));
    assert!(approx_eq(black.b, 0.0));
    assert!(approx_eq(black.a, 1.0));

    let white = Color::white();
    assert!(approx_eq(white.r, 1.0));
    assert!(approx_eq(white.g, 1.0));
    assert!(approx_eq(white.b, 1.0));
    assert!(approx_eq(white.a, 1.0));
}

#[test]
fn zero_and_one_map_to_byte_bounds() {
    let black = Color::new(0.0, 0.0, 0.0, 0.0).to_rgba8();
    assert_eq!(black, [0, 0, 0, 0]);

    let white = Color::new(1.0, 1.0, 1.0, 1.0).to_rgba8();
    assert_eq!(white, [255, 255, 255, 255]);
}

#[test]
fn clamp_bounds_below_zero_and_above_one() {
    let out_of_range = Color::new(-0.5, 1.5, 0.5, -1.0).clamp();
    assert!(approx_eq(out_of_range.r, 0.0));
    assert!(approx_eq(out_of_range.g, 1.0));
    assert!(approx_eq(out_of_range.b, 0.5));
    assert!(approx_eq(out_of_range.a, 0.0));
}

#[test]
fn component_wise_multiplication() {
    let a = Color::new(1.0, 0.5, 0.25, 1.0);
    let b = Color::new(0.5, 0.5, 0.5, 1.0);
    let result = a * b;

    assert!(approx_eq(result.r, 0.5));
    assert!(approx_eq(result.g, 0.25));
    assert!(approx_eq(result.b, 0.125));
    assert!(approx_eq(result.a, 1.0));
}

#[test]
fn addition_and_scalar_multiplication() {
    let a = Color::new(0.1, 0.2, 0.3, 1.0);
    let b = Color::new(0.1, 0.1, 0.1, 0.0);
    let sum = a + b;

    assert!(approx_eq(sum.r, 0.2));
    assert!(approx_eq(sum.g, 0.3));
    assert!(approx_eq(sum.b, 0.4));

    let scaled = a * 2.0;
    assert!(approx_eq(scaled.r, 0.2));
    assert!(approx_eq(scaled.g, 0.4));
    assert!(approx_eq(scaled.b, 0.6));
}

#[test]
fn out_of_range_conversion_stays_finite_and_in_byte_bounds() {
    let out_of_range = Color::new(-2.0, 3.0, 0.5, 2.0).to_rgba8();
    for channel in out_of_range {
        assert!((0..=255).contains(&(channel as i32)));
    }
}

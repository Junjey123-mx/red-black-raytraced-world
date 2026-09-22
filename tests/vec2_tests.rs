#[path = "../src/core/math/vec2.rs"]
mod vec2;

use vec2::Vec2;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn add_and_sub() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, -1.0);

    let sum = a + b;
    assert!(approx_eq(sum.x, 4.0));
    assert!(approx_eq(sum.y, 1.0));

    let diff = a - b;
    assert!(approx_eq(diff.x, -2.0));
    assert!(approx_eq(diff.y, 3.0));
}

#[test]
fn scalar_multiply_and_divide() {
    let a = Vec2::new(2.0, 4.0);

    let scaled = a * 2.0;
    assert!(approx_eq(scaled.x, 4.0));
    assert!(approx_eq(scaled.y, 8.0));

    let divided = a / 2.0;
    assert!(approx_eq(divided.x, 1.0));
    assert!(approx_eq(divided.y, 2.0));
}

#[test]
fn dot_product() {
    let a = Vec2::new(1.0, 0.0);
    let b = Vec2::new(0.0, 1.0);
    assert!(approx_eq(a.dot(b), 0.0));

    let c = Vec2::new(2.0, 3.0);
    let d = Vec2::new(4.0, 5.0);
    assert!(approx_eq(c.dot(d), 23.0));
}

#[test]
fn length_3_4_5() {
    let v = Vec2::new(3.0, 4.0);
    assert!(approx_eq(v.length(), 5.0));
}

#[test]
fn normalize_produces_unit_length() {
    let v = Vec2::new(3.0, 4.0).normalize();
    assert!(approx_eq(v.length(), 1.0));
}

#[test]
fn normalize_zero_vector_is_finite() {
    let v = Vec2::zero().normalize();
    assert!(v.x.is_finite());
    assert!(v.y.is_finite());
    assert!(approx_eq(v.x, 0.0));
    assert!(approx_eq(v.y, 0.0));
}

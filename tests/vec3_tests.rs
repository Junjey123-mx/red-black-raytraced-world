#[path = "../src/core/math/vec3.rs"]
mod vec3;

use vec3::Vec3;

const EPS: f32 = 1e-5;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= EPS
}

#[test]
fn add_sub_and_negate() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, -1.0, 2.0);

    let sum = a + b;
    assert!(approx_eq(sum.x, 5.0));
    assert!(approx_eq(sum.y, 1.0));
    assert!(approx_eq(sum.z, 5.0));

    let diff = a - b;
    assert!(approx_eq(diff.x, -3.0));
    assert!(approx_eq(diff.y, 3.0));
    assert!(approx_eq(diff.z, 1.0));

    let negated = -a;
    assert!(approx_eq(negated.x, -1.0));
    assert!(approx_eq(negated.y, -2.0));
    assert!(approx_eq(negated.z, -3.0));
}

#[test]
fn scalar_multiply_and_divide() {
    let a = Vec3::new(2.0, 4.0, 6.0);

    let scaled = a * 2.0;
    assert!(approx_eq(scaled.x, 4.0));
    assert!(approx_eq(scaled.y, 8.0));
    assert!(approx_eq(scaled.z, 12.0));

    let divided = a / 2.0;
    assert!(approx_eq(divided.x, 1.0));
    assert!(approx_eq(divided.y, 2.0));
    assert!(approx_eq(divided.z, 3.0));
}

#[test]
fn dot_product_of_orthogonal_vectors_is_zero() {
    let x_axis = Vec3::new(1.0, 0.0, 0.0);
    let y_axis = Vec3::new(0.0, 1.0, 0.0);
    assert!(approx_eq(x_axis.dot(y_axis), 0.0));
}

#[test]
fn cross_product_is_perpendicular_and_correct() {
    let x_axis = Vec3::new(1.0, 0.0, 0.0);
    let y_axis = Vec3::new(0.0, 1.0, 0.0);
    let z_axis = x_axis.cross(y_axis);

    assert!(approx_eq(z_axis.x, 0.0));
    assert!(approx_eq(z_axis.y, 0.0));
    assert!(approx_eq(z_axis.z, 1.0));

    assert!(approx_eq(z_axis.dot(x_axis), 0.0));
    assert!(approx_eq(z_axis.dot(y_axis), 0.0));
}

#[test]
fn length_and_length_squared() {
    let v = Vec3::new(2.0, 3.0, 6.0);
    assert!(approx_eq(v.length_squared(), 49.0));
    assert!(approx_eq(v.length(), 7.0));
}

#[test]
fn normalize_produces_unit_length() {
    let v = Vec3::new(2.0, 3.0, 6.0).normalize();
    assert!(approx_eq(v.length(), 1.0));
}

#[test]
fn normalize_zero_vector_is_finite() {
    let v = Vec3::zero().normalize();
    assert!(v.x.is_finite());
    assert!(v.y.is_finite());
    assert!(v.z.is_finite());
    assert!(approx_eq(v.x, 0.0));
    assert!(approx_eq(v.y, 0.0));
    assert!(approx_eq(v.z, 0.0));
}

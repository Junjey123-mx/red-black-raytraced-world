#[path = "../src/core/math/ivec3.rs"]
mod ivec3;

use ivec3::IVec3;
use std::collections::HashMap;

#[test]
fn equality() {
    let a = IVec3::new(1, -2, 3);
    let b = IVec3::new(1, -2, 3);
    let c = IVec3::new(1, -2, 4);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn negative_coordinates_are_valid() {
    let v = IVec3::new(-5, -10, -15);
    assert_eq!(v.x, -5);
    assert_eq!(v.y, -10);
    assert_eq!(v.z, -15);
}

#[test]
fn add_and_sub() {
    let a = IVec3::new(1, 2, 3);
    let b = IVec3::new(-4, 5, -6);

    let sum = a + b;
    assert_eq!(sum, IVec3::new(-3, 7, -3));

    let diff = a - b;
    assert_eq!(diff, IVec3::new(5, -3, 9));
}

#[test]
fn usable_as_hashmap_key() {
    let mut map: HashMap<IVec3, &str> = HashMap::new();
    map.insert(IVec3::new(0, 0, 0), "origin");
    map.insert(IVec3::new(-1, 2, -3), "negative cell");

    assert_eq!(map.get(&IVec3::new(0, 0, 0)), Some(&"origin"));
    assert_eq!(map.get(&IVec3::new(-1, 2, -3)), Some(&"negative cell"));
    assert_eq!(map.get(&IVec3::new(9, 9, 9)), None);
}

#[test]
fn default_is_zero() {
    assert_eq!(IVec3::default(), IVec3::zero());
    assert_eq!(IVec3::zero(), IVec3::new(0, 0, 0));
}

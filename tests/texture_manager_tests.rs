#[path = "."]
mod core {
    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
}

use core::color::Color;
use core::texture::TextureId;
use scene::texture_manager::{TextureLoadError, TextureManager};

const EPS: f32 = 1.0 / 255.0 + 1e-6;

fn approx_color(a: Color, b: Color) -> bool {
    (a.r - b.r).abs() <= EPS
        && (a.g - b.g).abs() <= EPS
        && (a.b - b.b).abs() <= EPS
        && (a.a - b.a).abs() <= EPS
}

fn diagnostic_png_path() -> String {
    format!(
        "{}/assets/textures/diagnostic/uv_reference.png",
        env!("CARGO_MANIFEST_DIR")
    )
}

const RED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
const GREEN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
const BLUE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
const YELLOW: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
const MAGENTA: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

#[test]
fn diagnostic_png_exists_and_can_be_loaded() {
    let path = diagnostic_png_path();
    assert!(std::path::Path::new(&path).exists());

    let mut manager = TextureManager::new();
    assert!(manager.load(&path).is_ok());
}

#[test]
fn loading_returns_a_valid_texture_id_that_resolves() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();

    assert!(manager.get(id).is_some());
}

#[test]
fn loaded_texture_has_expected_width() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    assert_eq!(texture.width(), 16);
}

#[test]
fn loaded_texture_has_expected_height() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    assert_eq!(texture.height(), 16);
}

#[test]
fn top_left_texel_is_red() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    let texel = texture.texel(0, 0).unwrap();
    assert!(approx_color(texel, RED), "got {texel:?}");
}

#[test]
fn top_right_texel_is_green() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    let texel = texture.texel(15, 0).unwrap();
    assert!(approx_color(texel, GREEN), "got {texel:?}");
}

#[test]
fn bottom_left_texel_is_blue() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    let texel = texture.texel(0, 15).unwrap();
    assert!(approx_color(texel, BLUE), "got {texel:?}");
}

#[test]
fn bottom_right_texel_is_yellow() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    let texel = texture.texel(15, 15).unwrap();
    assert!(approx_color(texel, YELLOW), "got {texel:?}");
}

#[test]
fn center_region_is_magenta() {
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    let center_a = texture.texel(7, 7).unwrap();
    let center_b = texture.texel(8, 8).unwrap();
    assert!(approx_color(center_a, MAGENTA), "got {center_a:?}");
    assert!(approx_color(center_b, MAGENTA), "got {center_b:?}");
}

#[test]
fn orientation_is_not_flipped_during_raylib_to_cpu_conversion() {
    // If rows were flipped during conversion, the top row (y=0) would show
    // the source image's bottom colors (blue/yellow) instead of its top
    // colors (red/green), and vice versa. Checking both rows against their
    // distinct expected colors catches a vertical flip; checking both
    // columns catches a horizontal flip.
    let mut manager = TextureManager::new();
    let id = manager.load(diagnostic_png_path()).unwrap();
    let texture = manager.get(id).unwrap();

    assert!(approx_color(texture.texel(0, 0).unwrap(), RED));
    assert!(approx_color(texture.texel(0, 15).unwrap(), BLUE));
    assert_ne!(
        texture.texel(0, 0).unwrap(),
        texture.texel(0, 15).unwrap(),
        "top-left and bottom-left must differ; a vertical flip would collapse orientation"
    );
    assert!(approx_color(texture.texel(0, 0).unwrap(), RED));
    assert!(approx_color(texture.texel(15, 0).unwrap(), GREEN));
    assert_ne!(
        texture.texel(0, 0).unwrap(),
        texture.texel(15, 0).unwrap(),
        "top-left and top-right must differ; a horizontal flip would collapse orientation"
    );
}

#[test]
fn loading_the_same_path_twice_returns_the_same_id() {
    let mut manager = TextureManager::new();
    let id_a = manager.load(diagnostic_png_path()).unwrap();
    let id_b = manager.load(diagnostic_png_path()).unwrap();

    assert_eq!(id_a, id_b);
}

#[test]
fn second_load_of_the_same_path_does_not_grow_storage() {
    let mut manager = TextureManager::new();
    manager.load(diagnostic_png_path()).unwrap();
    assert_eq!(manager.len(), 1);

    manager.load(diagnostic_png_path()).unwrap();
    assert_eq!(manager.len(), 1);
}

#[test]
fn two_distinct_paths_yield_distinct_ids() {
    // Duplicate the diagnostic asset under a different filename in a
    // temporary directory at test time, so identity is tested without
    // committing a second PNG to the repository.
    let original = diagnostic_png_path();
    let mut duplicate = std::env::temp_dir();
    duplicate.push("texture_manager_tests_duplicate_uv_reference.png");
    std::fs::copy(&original, &duplicate).unwrap();

    let mut manager = TextureManager::new();
    let id_original = manager.load(&original).unwrap();
    let id_duplicate = manager.load(&duplicate).unwrap();

    assert_ne!(id_original, id_duplicate);
    assert_eq!(manager.len(), 2);

    let _ = std::fs::remove_file(&duplicate);
}

#[test]
fn nonexistent_path_returns_decode_failed_error() {
    let mut manager = TextureManager::new();
    let result = manager.load("this/path/does/not/exist/anywhere.png");

    assert!(matches!(result, Err(TextureLoadError::DecodeFailed)));
}

#[test]
fn texture_id_outside_the_manager_resolves_to_none() {
    let mut manager = TextureManager::new();
    manager.load(diagnostic_png_path()).unwrap();

    let bogus_id = TextureId::new(9_999);
    assert!(manager.get(bogus_id).is_none());
}

#[test]
fn empty_manager_reports_zero_length() {
    let manager = TextureManager::new();
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

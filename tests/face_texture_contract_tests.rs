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
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
}

use core::color::Color;
use core::cube::Cube;
use core::face_textures::FaceTextures;
use core::hit::Face;
use core::math::Vec3;
use core::ray::Ray;
use core::texture::TextureId;
use renderer::texture_sampling::sample_nearest;
use scene::texture_manager::TextureManager;

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
const CYAN: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

const EPS: f32 = 1.0 / 255.0 + 1e-6;

fn approx_color(a: Color, b: Color) -> bool {
    (a.r - b.r).abs() <= EPS
        && (a.g - b.g).abs() <= EPS
        && (a.b - b.b).abs() <= EPS
        && (a.a - b.a).abs() <= EPS
}

fn face_png_path(name: &str) -> String {
    format!(
        "{}/assets/textures/diagnostic/faces/{}.png",
        env!("CARGO_MANIFEST_DIR"),
        name
    )
}

/// Loads the six real diagnostic PNGs into a fresh `TextureManager` and
/// returns the manager plus the `TextureId` for each face, in
/// (+X, -X, +Y, -Y, +Z, -Z) order.
fn load_all_face_textures() -> (TextureManager, [TextureId; 6]) {
    let mut manager = TextureManager::new();
    let positive_x = manager.load(face_png_path("positive_x")).unwrap();
    let negative_x = manager.load(face_png_path("negative_x")).unwrap();
    let positive_y = manager.load(face_png_path("positive_y")).unwrap();
    let negative_y = manager.load(face_png_path("negative_y")).unwrap();
    let positive_z = manager.load(face_png_path("positive_z")).unwrap();
    let negative_z = manager.load(face_png_path("negative_z")).unwrap();

    (
        manager,
        [
            positive_x, negative_x, positive_y, negative_y, positive_z, negative_z,
        ],
    )
}

fn unit_cube() -> Cube {
    Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0))
}

// --- PNG diagnostic loading tests (TextureManager only, no FaceTextures) ---

#[test]
fn all_six_face_pngs_load_successfully() {
    let (_manager, ids) = load_all_face_textures();
    // `load` returning Ok for all six is implicit in `load_all_face_textures`
    // (it unwraps each call); this test exists to name that contract.
    assert_eq!(ids.len(), 6);
}

#[test]
fn all_six_face_pngs_have_expected_dimensions() {
    let (manager, ids) = load_all_face_textures();
    for id in ids {
        let texture = manager.get(id).unwrap();
        assert_eq!(texture.width(), 16);
        assert_eq!(texture.height(), 16);
    }
}

#[test]
fn all_six_face_pngs_produce_distinct_texture_ids() {
    let (_manager, ids) = load_all_face_textures();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "ids at {i} and {j} collided");
        }
    }
}

#[test]
fn manager_holds_exactly_six_textures_after_loading_all_faces() {
    let (manager, _ids) = load_all_face_textures();
    assert_eq!(manager.len(), 6);
}

#[test]
fn each_face_png_resolves_to_its_own_dominant_color_at_center() {
    let (manager, ids) = load_all_face_textures();
    let expected = [RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN];

    for (id, expected_color) in ids.iter().zip(expected.iter()) {
        let texture = manager.get(*id).unwrap();
        let center = texture.texel(7, 7).unwrap();
        assert!(
            approx_color(center, *expected_color),
            "expected {expected_color:?}, got {center:?}"
        );
    }
}

#[test]
fn each_face_png_has_its_asymmetric_marker_at_the_expected_position() {
    let (manager, ids) = load_all_face_textures();

    // (+X, -X, +Y, -Y, +Z, -Z): one representative marker pixel per file,
    // matching the generator's placement.
    let marker_checks: [(usize, usize, Color); 6] = [
        (1, 1, WHITE),   // positive_x: white 2x2 top-left
        (13, 1, BLACK),  // negative_x: black 2x2 top-right
        (1, 13, WHITE),  // positive_y: white 2x2 bottom-left
        (13, 13, BLACK), // negative_y: black 2x2 bottom-right
        (2, 7, WHITE),   // positive_z: white horizontal line, row 7
        (14, 5, BLACK),  // negative_z: black vertical line, col 14
    ];

    for (id, (x, y, expected_color)) in ids.iter().zip(marker_checks.iter()) {
        let texture = manager.get(*id).unwrap();
        let marker = texture.texel(*x, *y).unwrap();
        assert!(
            approx_color(marker, *expected_color),
            "marker at ({x},{y}) expected {expected_color:?}, got {marker:?}"
        );
    }
}

// --- Full integration: Ray -> Cube -> HitRecord -> Face -> FaceTextures ->
// TextureId -> TextureManager -> CpuTexture -> sample_nearest -> Color ---

fn trace_face_color(
    cube: &Cube,
    ray: &Ray,
    face_textures: &FaceTextures,
    manager: &TextureManager,
) -> (Face, TextureId, Color) {
    let hit = cube.intersect(ray, 0.0, f32::INFINITY).unwrap();
    let texture_id = face_textures.texture_for_face(hit.face);
    let texture = manager.get(texture_id).unwrap();
    let color = sample_nearest(texture, hit.uv);
    (hit.face, texture_id, color)
}

#[test]
fn positive_x_face_resolves_through_the_full_chain() {
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));

    let (face, texture_id, color) = trace_face_color(&cube, &ray, &face_textures, &manager);

    assert_eq!(face, Face::PositiveX);
    assert_eq!(texture_id, ids[0]);
    assert!(approx_color(color, RED), "got {color:?}");
}

#[test]
fn negative_x_face_resolves_through_the_full_chain() {
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

    let (face, texture_id, color) = trace_face_color(&cube, &ray, &face_textures, &manager);

    assert_eq!(face, Face::NegativeX);
    assert_eq!(texture_id, ids[1]);
    assert!(approx_color(color, GREEN), "got {color:?}");
}

#[test]
fn positive_y_face_resolves_through_the_full_chain() {
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));

    let (face, texture_id, color) = trace_face_color(&cube, &ray, &face_textures, &manager);

    assert_eq!(face, Face::PositiveY);
    assert_eq!(texture_id, ids[2]);
    assert!(approx_color(color, BLUE), "got {color:?}");
}

#[test]
fn negative_y_face_resolves_through_the_full_chain() {
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

    let (face, texture_id, color) = trace_face_color(&cube, &ray, &face_textures, &manager);

    assert_eq!(face, Face::NegativeY);
    assert_eq!(texture_id, ids[3]);
    assert!(approx_color(color, YELLOW), "got {color:?}");
}

#[test]
fn positive_z_face_resolves_through_the_full_chain() {
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

    let (face, texture_id, color) = trace_face_color(&cube, &ray, &face_textures, &manager);

    assert_eq!(face, Face::PositiveZ);
    assert_eq!(texture_id, ids[4]);
    assert!(approx_color(color, MAGENTA), "got {color:?}");
}

#[test]
fn negative_z_face_resolves_through_the_full_chain() {
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();
    let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0));

    let (face, texture_id, color) = trace_face_color(&cube, &ray, &face_textures, &manager);

    assert_eq!(face, Face::NegativeZ);
    assert_eq!(texture_id, ids[5]);
    assert!(approx_color(color, CYAN), "got {color:?}");
}

#[test]
fn all_six_faces_produce_pairwise_distinct_colors_in_one_pass() {
    // A single combined pass over all six real hits: if any pair of
    // opposite faces (+X/-X, +Y/-Y, +Z/-Z) were swapped in FaceTextures or
    // misassigned by Cube's face selection, two of these would collapse to
    // the same color instead of staying pairwise distinct.
    let (manager, ids) = load_all_face_textures();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let cube = unit_cube();

    let rays = [
        Ray::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
        Ray::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)),
        Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::new(0.0, 0.0, 1.0)),
    ];

    let colors: Vec<Color> = rays
        .iter()
        .map(|ray| trace_face_color(&cube, ray, &face_textures, &manager).2)
        .collect();

    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            assert!(
                !approx_color(colors[i], colors[j]),
                "faces at index {i} and {j} produced the same color {:?}",
                colors[i]
            );
        }
    }
}

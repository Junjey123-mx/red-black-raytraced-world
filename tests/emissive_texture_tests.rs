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

    #[path = "../src/core/color.rs"]
    pub mod color;
    #[path = "../src/core/face_textures.rs"]
    pub mod face_textures;
    #[path = "../src/core/hit.rs"]
    pub mod hit;
    #[path = "../src/core/material.rs"]
    pub mod material;
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
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
}

use core::color::Color;
use core::material::Material;
use core::math::Vec2;
use core::texture::TextureId;
use renderer::emission::sample_emissive;
use renderer::texture_sampling::sample_nearest;
use scene::texture_manager::TextureManager;

const EPS: f32 = 1e-5;
const N: usize = 16;
const ALBEDO: &str = "assets/textures/overworld/redstone_lamp/albedo.png";
const EMISSIVE: &str = "assets/textures/overworld/redstone_lamp/emissive.png";

struct Lamp {
    manager: TextureManager,
    albedo: TextureId,
    emissive: TextureId,
}

fn lamp() -> Lamp {
    let mut manager = TextureManager::new();
    let albedo = manager.load(ALBEDO).expect("lamp albedo");
    let emissive = manager.load(EMISSIVE).expect("lamp emissive mask");
    Lamp {
        manager,
        albedo,
        emissive,
    }
}

fn lamp_material(lamp: &Lamp, strength: f32) -> Material {
    Material::matte(Color::new(0.4, 0.3, 0.2, 1.0))
        .with_emissive_texture(lamp.emissive)
        .with_emission_strength(strength)
}

/// Center of texel `(i, j)` in UV space (`v = 0` is the top row).
fn uv(i: usize, j: usize) -> Vec2 {
    Vec2::new((i as f32 + 0.5) / N as f32, (j as f32 + 0.5) / N as f32)
}

fn is_black(c: Color) -> bool {
    c.r.abs() < EPS && c.g.abs() < EPS && c.b.abs() < EPS
}

/// Texels of the mask that emit, as `(i, j)`.
fn lit_texels(lamp: &Lamp) -> Vec<(usize, usize)> {
    let texture = lamp.manager.get(lamp.emissive).unwrap();
    let mut lit = Vec::new();
    for j in 0..N {
        for i in 0..N {
            let t = texture.texel(i, j).unwrap();
            if t.r > 0.0 || t.g > 0.0 || t.b > 0.0 {
                lit.push((i, j));
            }
        }
    }
    lit
}

#[test]
fn a_luminous_texel_emits_its_mask_color() {
    let lamp = lamp();
    let material = lamp_material(&lamp, 1.0);
    let (i, j) = *lit_texels(&lamp).first().expect("the mask has lit texels");

    let emitted = sample_emissive(&material, uv(i, j), &lamp.manager);
    let texel = lamp
        .manager
        .get(lamp.emissive)
        .unwrap()
        .texel(i, j)
        .unwrap();

    assert!(!is_black(emitted));
    assert!((emitted.r - texel.r).abs() < EPS && (emitted.g - texel.g).abs() < EPS);
    assert!((emitted.b - texel.b).abs() < EPS);
}

#[test]
fn a_non_luminous_texel_emits_nothing() {
    let lamp = lamp();
    let material = lamp_material(&lamp, 3.0);

    // The outer frame ring is never a panel.
    for k in 0..N {
        for (i, j) in [(k, 0), (k, N - 1), (0, k), (N - 1, k)] {
            let emitted = sample_emissive(&material, uv(i, j), &lamp.manager);
            assert!(
                is_black(emitted),
                "frame texel ({i},{j}) emitted {emitted:?}"
            );
        }
    }
}

#[test]
fn emission_strength_scales_the_output_linearly() {
    let lamp = lamp();
    let (i, j) = lit_texels(&lamp)[0];

    let base = sample_emissive(&lamp_material(&lamp, 1.0), uv(i, j), &lamp.manager);
    let doubled = sample_emissive(&lamp_material(&lamp, 2.0), uv(i, j), &lamp.manager);
    let off = sample_emissive(&lamp_material(&lamp, 0.0), uv(i, j), &lamp.manager);

    assert!((doubled.r - 2.0 * base.r).abs() < EPS);
    assert!((doubled.g - 2.0 * base.g).abs() < EPS);
    assert!((doubled.b - 2.0 * base.b).abs() < EPS);
    assert!(is_black(off), "strength 0 must switch emission off");
}

#[test]
fn a_material_without_an_emissive_texture_emits_nothing() {
    let lamp = lamp();
    let material = Material::matte(Color::new(0.4, 0.3, 0.2, 1.0)).with_emission_strength(5.0);
    let (i, j) = lit_texels(&lamp)[0];

    assert!(is_black(sample_emissive(
        &material,
        uv(i, j),
        &lamp.manager
    )));
}

#[test]
fn an_unloaded_emissive_texture_id_is_safely_black() {
    let lamp = lamp();
    let material = Material::matte(Color::black())
        .with_emissive_texture(TextureId::new(9999))
        .with_emission_strength(2.0);

    assert!(is_black(sample_emissive(
        &material,
        uv(8, 8),
        &lamp.manager
    )));
}

#[test]
fn albedo_and_emissive_share_the_same_uv() {
    let lamp = lamp();
    let material = lamp_material(&lamp, 1.0);
    let albedo = lamp.manager.get(lamp.albedo).unwrap();

    for j in 0..N {
        for i in 0..N {
            let coordinate = uv(i, j);
            let emitted = sample_emissive(&material, coordinate, &lamp.manager);
            let base = sample_nearest(albedo, coordinate);

            if is_black(emitted) {
                // Frame texel: dark brown, clearly darker than any panel.
                let lum = 0.2126 * base.r + 0.7152 * base.g + 0.0722 * base.b;
                assert!(lum < 0.3, "texel ({i},{j}) frame albedo {base:?}");
            } else {
                // Panel texel: amber/bright albedo, and the emitted color
                // is the same hue as the albedo texel at that very UV.
                let lum = 0.2126 * base.r + 0.7152 * base.g + 0.0722 * base.b;
                assert!(lum > 0.3, "texel ({i},{j}) panel albedo {base:?}");
                assert!(emitted.r <= base.r + EPS && emitted.b <= base.b + EPS);
            }
        }
    }
}

#[test]
fn the_redstone_mask_does_not_emit_over_the_whole_block() {
    let lamp = lamp();
    let lit = lit_texels(&lamp).len();

    // Five diamond panels (one central, four in the corners): a substantial
    // part of the face, far from all of it.
    assert!(lit >= N * N / 4, "too few emitting texels: {lit}");
    assert!(lit <= N * N * 2 / 3, "too many emitting texels: {lit}");
    // The outer frame ring never emits.
    let material = lamp_material(&lamp, 1.0);
    for k in 0..N {
        for (x, y) in [(k, 0), (k, N - 1), (0, k), (N - 1, k)] {
            assert!(is_black(sample_emissive(
                &material,
                uv(x, y),
                &lamp.manager
            )));
        }
    }
}

#[test]
fn sampling_is_nearest_neighbor_without_interpolation() {
    let lamp = lamp();
    let material = lamp_material(&lamp, 1.0);
    let (i, j) = lit_texels(&lamp)[0];

    let a = sample_emissive(
        &material,
        Vec2::new((i as f32 + 0.05) / 16.0, uv(i, j).y),
        &lamp.manager,
    );
    let b = sample_emissive(
        &material,
        Vec2::new((i as f32 + 0.95) / 16.0, uv(i, j).y),
        &lamp.manager,
    );

    assert_eq!(a, b, "the whole texel must return one constant color");
}

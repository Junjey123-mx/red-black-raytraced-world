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
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

use core::face_textures::FaceTextures;
use core::hit::Face;
use core::texture::TextureId;

/// Six distinct, hand-constructed ids. `TextureId::new` is `pub(crate)`,
/// visible here because this test file is recompiled as part of the same
/// crate tree as `core::texture` (via the `#[path]` inclusion above), which
/// is the same mechanism `TextureManager` itself uses to mint real ids.
fn six_distinct_ids() -> [TextureId; 6] {
    [
        TextureId::new(10),
        TextureId::new(11),
        TextureId::new(12),
        TextureId::new(13),
        TextureId::new(14),
        TextureId::new(15),
    ]
}

fn sample_face_textures() -> (FaceTextures, [TextureId; 6]) {
    let ids = six_distinct_ids();
    let face_textures = FaceTextures::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    (face_textures, ids)
}

#[test]
fn positive_x_returns_its_own_texture_id() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::PositiveX), ids[0]);
}

#[test]
fn negative_x_returns_its_own_texture_id() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::NegativeX), ids[1]);
}

#[test]
fn positive_y_returns_its_own_texture_id() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::PositiveY), ids[2]);
}

#[test]
fn negative_y_returns_its_own_texture_id() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::NegativeY), ids[3]);
}

#[test]
fn positive_z_returns_its_own_texture_id() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::PositiveZ), ids[4]);
}

#[test]
fn negative_z_returns_its_own_texture_id() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::NegativeZ), ids[5]);
}

#[test]
fn no_two_faces_resolve_to_the_same_texture_id() {
    let (face_textures, _ids) = sample_face_textures();
    let resolved = [
        face_textures.texture_for_face(Face::PositiveX),
        face_textures.texture_for_face(Face::NegativeX),
        face_textures.texture_for_face(Face::PositiveY),
        face_textures.texture_for_face(Face::NegativeY),
        face_textures.texture_for_face(Face::PositiveZ),
        face_textures.texture_for_face(Face::NegativeZ),
    ];

    for i in 0..resolved.len() {
        for j in (i + 1)..resolved.len() {
            assert_ne!(
                resolved[i], resolved[j],
                "faces at index {i} and {j} resolved to the same TextureId"
            );
        }
    }
}

#[test]
fn swapping_positive_and_negative_x_would_be_detected() {
    let (face_textures, ids) = sample_face_textures();
    // This is the failure mode the suite must catch: if +X and -X were
    // accidentally swapped in the implementation, this assertion would
    // fail because texture_for_face(PositiveX) would equal ids[1] instead.
    assert_ne!(face_textures.texture_for_face(Face::PositiveX), ids[1]);
    assert_ne!(face_textures.texture_for_face(Face::NegativeX), ids[0]);
}

#[test]
fn repeated_calls_are_deterministic() {
    let (face_textures, ids) = sample_face_textures();
    assert_eq!(face_textures.texture_for_face(Face::PositiveZ), ids[4]);
    assert_eq!(face_textures.texture_for_face(Face::PositiveZ), ids[4]);
}

#[test]
fn uniform_assigns_the_same_id_to_all_six_faces() {
    let id = TextureId::new(7);
    let face_textures = FaceTextures::uniform(id);

    assert_eq!(face_textures.texture_for_face(Face::PositiveX), id);
    assert_eq!(face_textures.texture_for_face(Face::NegativeX), id);
    assert_eq!(face_textures.texture_for_face(Face::PositiveY), id);
    assert_eq!(face_textures.texture_for_face(Face::NegativeY), id);
    assert_eq!(face_textures.texture_for_face(Face::PositiveZ), id);
    assert_eq!(face_textures.texture_for_face(Face::NegativeZ), id);
}

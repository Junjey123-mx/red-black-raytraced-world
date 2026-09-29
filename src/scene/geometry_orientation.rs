// Scene-stage transform: lands ahead of the block shape factory that will
// define canonical shapes and orient them per placed block.
#![allow(dead_code)]

use crate::core::math::Vec3;
use crate::core::prism::Prism;
use crate::scene::block_geometry::BlockGeometry;
use crate::scene::orientation::Orientation;

/// Orients a canonical local shape inside its unit cell `[0, 1]^3`.
///
/// Convention (same axes as `Orientation`): a canonical shape is authored
/// *facing South* (+Z) and *upright* (+Y).
///
/// ```text
/// South, Up -> identity (canonical)
/// East      -> rotated about the cell's vertical axis so +Z maps to +X
/// North     -> rotated 180 degrees so +Z maps to -Z
/// West      -> rotated about the vertical axis so +Z maps to -X
/// Down      -> mirrored top-to-bottom (y -> 1 - y), facing unchanged
/// ```
///
/// Cardinal orientations only turn the shape around the vertical axis and
/// never move it off the ground plane; `Down` flips it upside down, which is
/// what the inverted Red-Black half needs. Everything is exact remapping of
/// coordinates inside the unit cube (no matrices), so bounds stay in
/// `[0, 1]`.
fn map_point(point: Vec3, orientation: Orientation) -> Vec3 {
    let (x, y, z) = (point.x, point.y, point.z);
    match orientation {
        Orientation::Up | Orientation::South => Vec3::new(x, y, z),
        Orientation::Down => Vec3::new(x, 1.0 - y, z),
        Orientation::North => Vec3::new(1.0 - x, y, 1.0 - z),
        Orientation::East => Vec3::new(z, y, 1.0 - x),
        Orientation::West => Vec3::new(1.0 - z, y, x),
    }
}

/// Orients a single prism (see `map_point` for the convention).
pub fn orient_prism(prism: &Prism, orientation: Orientation) -> Prism {
    Prism::new(
        map_point(prism.min(), orientation),
        map_point(prism.max(), orientation),
    )
}

/// Orients a whole geometry. A `FullCube` is symmetric and is returned
/// unchanged; prisms and composites keep their part count and order.
pub fn orient_geometry(geometry: &BlockGeometry, orientation: Orientation) -> BlockGeometry {
    match geometry {
        BlockGeometry::FullCube => BlockGeometry::FullCube,
        BlockGeometry::Prism(prism) => BlockGeometry::Prism(orient_prism(prism, orientation)),
        BlockGeometry::Composite(parts) => BlockGeometry::Composite(
            parts
                .iter()
                .map(|part| orient_prism(part, orientation))
                .collect(),
        ),
    }
}

/// Maps a shape authored *growing Up* from the cell's bottom face onto the
/// face selected by `orientation`: the shape's base sits on the opposite
/// face and its tips point along the orientation axis.
///
/// ```text
/// Up    -> identity (base on -Y, tips toward +Y)
/// Down  -> base on +Y, tips toward -Y
/// North -> base on +Z, tips toward -Z      East -> base on -X, tips toward +X
/// South -> base on -Z, tips toward +Z      West -> base on +X, tips toward -X
/// ```
///
/// Each mapping is an exact proper rotation of the unit cube, so bounds stay
/// in `[0, 1]`.
fn map_growth_point(point: Vec3, orientation: Orientation) -> Vec3 {
    let (x, y, z) = (point.x, point.y, point.z);
    match orientation {
        Orientation::Up => Vec3::new(x, y, z),
        Orientation::Down => Vec3::new(x, 1.0 - y, 1.0 - z),
        Orientation::North => Vec3::new(x, z, 1.0 - y),
        Orientation::South => Vec3::new(x, 1.0 - z, y),
        Orientation::East => Vec3::new(y, 1.0 - x, z),
        Orientation::West => Vec3::new(1.0 - y, x, z),
    }
}

/// Orients a shape that grows out of a face (see `map_growth_point`).
pub fn orient_growth(geometry: &BlockGeometry, orientation: Orientation) -> BlockGeometry {
    let orient = |prism: &Prism| {
        Prism::new(
            map_growth_point(prism.min(), orientation),
            map_growth_point(prism.max(), orientation),
        )
    };

    match geometry {
        BlockGeometry::FullCube => BlockGeometry::FullCube,
        BlockGeometry::Prism(prism) => BlockGeometry::Prism(orient(prism)),
        BlockGeometry::Composite(parts) => {
            BlockGeometry::Composite(parts.iter().map(orient).collect())
        }
    }
}

// ---------------------------------------------------------------------
// Functional faces: which texture a geometric face shows
// ---------------------------------------------------------------------

use crate::core::hit::Face;
use crate::core::math::Vec2;

/// The block-local ("functional") face that a world-space geometric `face`
/// of a block placed with `orientation` corresponds to, i.e. the inverse of
/// `map_point` applied to face normals. Face textures are authored for the
/// canonical pose (South, Up); a hit on a geometric face samples the texture
/// of its functional face:
///
/// ```text
/// Up / South -> identity
/// Down       -> +Y <-> -Y (the logical top now faces down)
/// North      -> +X <-> -X, +Z <-> -Z
/// East       -> +X -> +Z, -X -> -Z, +Z -> -X, -Z -> +X
/// West       -> +X -> -Z, -X -> +Z, +Z -> +X, -Z -> -X
/// ```
pub fn functional_face(face: Face, orientation: Orientation) -> Face {
    use Face::*;
    match orientation {
        Orientation::Up | Orientation::South => face,
        Orientation::Down => match face {
            PositiveY => NegativeY,
            NegativeY => PositiveY,
            other => other,
        },
        Orientation::North => match face {
            PositiveX => NegativeX,
            NegativeX => PositiveX,
            PositiveZ => NegativeZ,
            NegativeZ => PositiveZ,
            other => other,
        },
        Orientation::East => match face {
            PositiveX => PositiveZ,
            NegativeX => NegativeZ,
            PositiveZ => NegativeX,
            NegativeZ => PositiveX,
            other => other,
        },
        Orientation::West => match face {
            PositiveX => NegativeZ,
            NegativeX => PositiveZ,
            PositiveZ => PositiveX,
            NegativeZ => NegativeX,
            other => other,
        },
    }
}

/// The UV, in the functional face's own convention, of a hit at `uv` on
/// the geometric `face` of a block placed with `orientation` (see
/// `functional_face`). Lateral faces keep their UV under the cardinal
/// rotations (the per-face UV conventions rotate with them); `Down` flips
/// `v` on every face (the block is mirrored top-to-bottom) and the rotated
/// `+/-Y` faces turn their UV with the block.
pub fn functional_uv(face: Face, uv: Vec2, orientation: Orientation) -> Vec2 {
    let vertical = matches!(face, Face::PositiveY | Face::NegativeY);
    match orientation {
        Orientation::Up | Orientation::South => uv,
        Orientation::Down => Vec2::new(uv.x, 1.0 - uv.y),
        Orientation::North => {
            if vertical {
                Vec2::new(1.0 - uv.x, 1.0 - uv.y)
            } else {
                uv
            }
        }
        Orientation::East => match face {
            Face::PositiveY => Vec2::new(1.0 - uv.y, uv.x),
            Face::NegativeY => Vec2::new(uv.y, 1.0 - uv.x),
            _ => uv,
        },
        Orientation::West => match face {
            Face::PositiveY => Vec2::new(uv.y, 1.0 - uv.x),
            Face::NegativeY => Vec2::new(1.0 - uv.y, uv.x),
            _ => uv,
        },
    }
}

/// `functional_face` and `functional_uv` together.
pub fn functional_face_uv(face: Face, uv: Vec2, orientation: Orientation) -> (Face, Vec2) {
    (
        functional_face(face, orientation),
        functional_uv(face, uv, orientation),
    )
}

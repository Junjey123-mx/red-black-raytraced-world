// IVec3 and Vec2 are re-exported ahead of the voxel/UV code that will use
// them, so the bin target does not consume these names yet.
#![allow(unused_imports)]

mod ivec3;
mod vec2;
mod vec3;

pub use ivec3::IVec3;
pub use vec2::Vec2;
pub use vec3::Vec3;

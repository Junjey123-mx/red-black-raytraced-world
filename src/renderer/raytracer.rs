// Foundation-stage primitive: lands ahead of the full framebuffer render
// loop that will call `cast_ray`/`cast_ray_lit` once per pixel.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::cube::Cube;
use crate::core::hit::{Face, HitRecord};
use crate::core::material::Material;
use crate::core::math::{IVec3, Vec2, Vec3};
use crate::core::ray::Ray;
use crate::renderer::shading;
use crate::renderer::shadows;
use crate::renderer::texture_sampling::sample_nearest;
use crate::renderer::voxel_traversal::DdaState;
use crate::scene::block::BlockInstance;
use crate::scene::block_shape_factory::block_geometry;
use crate::scene::light::Light;
use crate::scene::material_library::MaterialLibrary;
use crate::scene::texture_manager::TextureManager;
use crate::scene::voxel_world::VoxelWorld;

/// Defensive cap on DDA steps for a single voxel raycast. `max_distance`
/// is the primary termination contract; this only guarantees the loop
/// ends even when a caller passes an unbounded distance.
pub const MAX_VOXEL_STEPS: usize = 4096;

/// The first real voxel hit of a ray: the occupied cell, its block record
/// (type, material reference, orientation), and the geometric
/// `HitRecord` (distance, point, normal, face, uv) produced by the local
/// geometry intersection (the `Cube::intersect` contract underneath).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelHit {
    pub cell: IVec3,
    pub block: BlockInstance,
    pub hit: HitRecord,
}

/// The full unit cube of a voxel cell: exactly `[x, x+1) x [y, y+1) x
/// [z, z+1)`, with no gaps between neighboring cells.
pub fn cell_cube(cell: IVec3) -> Cube {
    let min = Vec3::new(cell.x as f32, cell.y as f32, cell.z as f32);
    Cube::new(min, min + Vec3::new(1.0, 1.0, 1.0))
}

/// World-space origin (minimum corner) of a voxel cell.
fn cell_origin(cell: IVec3) -> Vec3 {
    Vec3::new(cell.x as f32, cell.y as f32, cell.z as f32)
}

/// Finds the nearest real voxel hit along `ray` within `max_distance`.
///
/// The 3D DDA only nominates candidate cells, in increasing entry
/// distance. An occupied cell is *not* automatically a hit: the local
/// geometry of its block (resolved from `BlockType` + `Orientation`, and
/// translated to the cell) must actually be struck by the ray, otherwise
/// traversal continues, so a ray can pass through the empty part of a stair,
/// fence, door, portal, or crystal cluster and hit whatever lies behind.
/// Every shape lives inside its own cell, and cells are visited in
/// increasing entry distance, so the first confirmed hit is the nearest one.
/// Returns `None` for an empty world, a miss, a non-positive/NaN
/// `max_distance`, or when `MAX_VOXEL_STEPS` is reached.
pub fn nearest_voxel_hit(world: &VoxelWorld, ray: &Ray, max_distance: f32) -> Option<VoxelHit> {
    if world.is_empty() || max_distance.is_nan() || max_distance <= 0.0 {
        return None;
    }

    let mut state = DdaState::from_ray(ray);

    for _ in 0..MAX_VOXEL_STEPS {
        if let Some(block) = world.get(state.cell)
            && let Some(hit) = block_geometry(block.block_type(), block.orientation())
                .translated(cell_origin(state.cell))
                .intersect_local(ray, 0.0, max_distance)
        {
            return Some(VoxelHit {
                cell: state.cell,
                block: *block,
                hit,
            });
        }

        let step = state.advance()?;
        if step.t_enter > max_distance {
            return None;
        }
    }

    None
}

/// Resolves a primary ray against a single diagnostic cube. This is
/// deliberately unlit: a hit produces a diagnostic color derived purely
/// from the surface normal, and a miss produces `background`. There is no
/// ambient/diffuse/specular shading, material, or light source involved.
pub fn cast_ray(cube: &Cube, ray: &Ray, background: Color) -> Color {
    match cube.intersect(ray, 0.0, f32::INFINITY) {
        Some(hit) => normal_to_diagnostic_color(hit.normal),
        None => background,
    }
}

/// Remaps each unit-normal component from `[-1, 1]` to `[0, 1]` so the six
/// cube faces render as six visually distinct, deterministic colors.
fn normal_to_diagnostic_color(normal: Vec3) -> Color {
    Color::new(
        (normal.x + 1.0) * 0.5,
        (normal.y + 1.0) * 0.5,
        (normal.z + 1.0) * 0.5,
        1.0,
    )
}

/// Resolves the ambient/diffuse albedo for a hit: a material without
/// `face_textures` keeps using its uniform `albedo` unchanged; a material
/// with `face_textures` looks up the `TextureId` for `face`, resolves it
/// through `texture_manager` (already-loaded, no disk access here), and
/// samples it at `uv` with the project's nearest-neighbor sampler.
/// `texture_manager` is a shared reference, so this can never load a file:
/// `TextureManager::load` requires `&mut self`.
fn resolve_albedo(
    material: &Material,
    face: Face,
    uv: Vec2,
    texture_manager: &TextureManager,
) -> Color {
    match &material.face_textures {
        Some(face_textures) => {
            let texture_id = face_textures.texture_for_face(face);
            let texture = texture_manager
                .get(texture_id)
                .expect("FaceTextures must only reference ids already loaded in TextureManager");
            sample_nearest(texture, uv)
        }
        None => material.albedo,
    }
}

/// The surface data lighting needs, independent of how the hit was found
/// (explicit cube list or voxel DDA).
struct SurfaceHit<'a> {
    point: Vec3,
    normal: Vec3,
    face: Face,
    uv: Vec2,
    material: &'a Material,
}

/// Shared local lighting: ambient always applies; each light adds its
/// diffuse and specular contribution only when `occluded` reports that the
/// epsilon-offset shadow ray toward it is unobstructed. `occluded(ray,
/// max_distance)` is injected so both the legacy cube list and the voxel
/// world answer occlusion through the same shading code.
/// `directional_range` bounds a directional light's shadow ray.
fn shade_surface(
    surface: &SurfaceHit,
    camera_position: Vec3,
    lights: &[Light],
    ambient_factor: f32,
    texture_manager: &TextureManager,
    directional_range: f32,
    occluded: impl Fn(&Ray, f32) -> bool,
) -> Color {
    let SurfaceHit {
        point,
        normal,
        material,
        ..
    } = *surface;

    let albedo = resolve_albedo(material, surface.face, surface.uv, texture_manager);
    // Ambient/diffuse read albedo through this per-hit override; specular
    // never depends on albedo, so it is unaffected by texturing either way.
    let shading_material = Material::new(albedo, material.specular, material.shininess);

    let view_direction = (camera_position - point).normalize();
    let mut color = shading::ambient(&shading_material, ambient_factor);

    for light in lights {
        match light {
            Light::Directional(directional) => {
                let shadow_ray =
                    shadows::shadow_ray_to_directional_light(point, normal, directional);
                let blocked = occluded(&shadow_ray, directional_range);
                if !blocked {
                    color = color
                        + shading::diffuse_directional(&shading_material, normal, directional);
                    color = color
                        + shading::specular_directional(
                            &shading_material,
                            normal,
                            view_direction,
                            directional,
                        );
                }
            }
            Light::Point(point_light) => {
                let (shadow_ray, distance) =
                    shadows::shadow_ray_to_point_light(point, normal, point_light);
                let blocked = occluded(&shadow_ray, distance);
                if !blocked {
                    color = color
                        + shading::diffuse_point(&shading_material, normal, point, point_light);
                    color = color
                        + shading::specular_point(
                            &shading_material,
                            normal,
                            point,
                            view_direction,
                            point_light,
                        );
                }
            }
        }
    }

    color.clamp()
}

/// Resolves a primary ray against the nearest of a small explicit
/// collection of diagnostic `(Cube, Material)` objects, then evaluates full
/// local lighting: ambient always applies; each light's diffuse and
/// specular contribution is added only when an epsilon-offset shadow ray
/// toward that light finds no blocking geometry among `objects`. A miss
/// returns `background`. This is the minimal nearest-hit search needed for
/// this Gate's diagnostic scene — not a VoxelWorld, DDA, or scene graph.
///
/// `texture_manager` supplies already-loaded `CpuTexture`s for any object
/// whose material carries `face_textures`; shadow occlusion stays purely
/// geometric and never consults it.
pub fn cast_ray_lit(
    objects: &[(Cube, Material)],
    ray: &Ray,
    camera_position: Vec3,
    lights: &[Light],
    ambient_factor: f32,
    background: Color,
    texture_manager: &TextureManager,
) -> Color {
    let mut nearest: Option<(f32, Vec3, Vec3, &Material, Face, Vec2)> = None;

    for (cube, material) in objects {
        if let Some(hit) = cube.intersect(ray, 0.0, f32::INFINITY) {
            let is_closer = nearest.as_ref().is_none_or(|(t, ..)| hit.distance < *t);
            if is_closer {
                nearest = Some((
                    hit.distance,
                    hit.point,
                    hit.normal,
                    material,
                    hit.face,
                    hit.uv,
                ));
            }
        }
    }

    let Some((_, point, normal, material, face, uv)) = nearest else {
        return background;
    };

    let cubes: Vec<&Cube> = objects.iter().map(|(cube, _)| cube).collect();
    let surface = SurfaceHit {
        point,
        normal,
        face,
        uv,
        material,
    };

    shade_surface(
        &surface,
        camera_position,
        lights,
        ambient_factor,
        texture_manager,
        shadows::DIRECTIONAL_SHADOW_RANGE,
        |shadow_ray, max_distance| {
            shadows::is_occluded(cubes.iter().copied(), shadow_ray, max_distance)
        },
    )
}

/// Voxel occlusion query for shadow rays: `true` when any real voxel hit
/// lies along `ray` within `max_distance`. It reuses `nearest_voxel_hit`, so
/// shadows go through exactly the same DDA + local-geometry path as primary
/// rays (no second traversal algorithm), including partial shapes. Blocks
/// are opaque.
pub fn is_voxel_occluded(world: &VoxelWorld, ray: &Ray, max_distance: f32) -> bool {
    nearest_voxel_hit(world, ray, max_distance).is_some()
}

/// Loud, deterministic color for a voxel whose `MaterialId` has no entry in
/// the supplied material map (a scene-construction bug, never silent).
pub const MISSING_MATERIAL_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

/// Full voxel render path for one primary ray: `VoxelWorld` + 3D DDA finds
/// the first real voxel hit, its `MaterialId` is resolved through the
/// `MaterialLibrary`, the albedo comes from the same `FaceTextures` /
/// `sample_nearest(hit.uv)` route as before, and ambient + diffuse +
/// specular are evaluated by the shared `shade_surface`. Shadow rays query
/// the *same* `world` through `is_voxel_occluded`.
///
/// `max_distance` bounds both the primary traversal and directional shadow
/// rays (it is the scene range, not a per-object value); a miss returns
/// `background`.
#[allow(clippy::too_many_arguments)]
pub fn cast_ray_voxel_lit(
    world: &VoxelWorld,
    materials: &MaterialLibrary,
    ray: &Ray,
    camera_position: Vec3,
    lights: &[Light],
    ambient_factor: f32,
    background: Color,
    texture_manager: &TextureManager,
    max_distance: f32,
) -> Color {
    let Some(voxel) = nearest_voxel_hit(world, ray, max_distance) else {
        return background;
    };

    let Some(material) = materials.get(voxel.block.material_id()) else {
        return MISSING_MATERIAL_COLOR;
    };

    let surface = SurfaceHit {
        point: voxel.hit.point,
        normal: voxel.hit.normal,
        face: voxel.hit.face,
        uv: voxel.hit.uv,
        material,
    };

    shade_surface(
        &surface,
        camera_position,
        lights,
        ambient_factor,
        texture_manager,
        shadows::DIRECTIONAL_SHADOW_RANGE.min(max_distance),
        |shadow_ray, shadow_distance| is_voxel_occluded(world, shadow_ray, shadow_distance),
    )
}

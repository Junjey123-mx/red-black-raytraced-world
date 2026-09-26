// Foundation-stage primitive: lands ahead of the full framebuffer render
// loop that will call `cast_ray`/`cast_ray_lit` once per pixel.
#![allow(dead_code)]

use crate::core::color::Color;
use crate::core::cube::Cube;
use crate::core::hit::{Face, HitRecord};
use crate::core::material::{AlphaMode, Material};
use crate::core::math::{IVec3, Vec2, Vec3};
use crate::core::ray::Ray;
use crate::core::reflection::reflect;
use crate::core::refraction::refract;
use crate::renderer::emission::sample_emissive;
use crate::renderer::normal_mapping::sample_shading_normal;
use crate::renderer::shading;
use crate::renderer::shadows;
use crate::renderer::texture_sampling::sample_nearest;
use crate::renderer::voxel_traversal::DdaState;
use crate::scene::block::BlockInstance;
use crate::scene::block_geometry::BlockGeometry;
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

/// UV of a point on a face measured against the *whole cell* (`local` in
/// `[0, 1]^3`), with exactly the per-face mapping `Cube` uses. Partial shapes
/// (stairs, fences, doors) use it instead of their own prism's extents, so a
/// half-height step shows the matching *part* of the texture — the pattern
/// continues across tread and riser instead of being squashed into each box.
/// For a prism that spans the full cell on the two axes of a face (a door's or
/// the portal membrane's broad faces) the result equals the prism-local UV.
fn cell_local_uv(local: Vec3, face: Face) -> Vec2 {
    let (lx, ly, lz) = (
        local.x.clamp(0.0, 1.0),
        local.y.clamp(0.0, 1.0),
        local.z.clamp(0.0, 1.0),
    );
    match face {
        Face::PositiveZ => Vec2::new(lx, 1.0 - ly),
        Face::NegativeZ => Vec2::new(1.0 - lx, 1.0 - ly),
        Face::PositiveX => Vec2::new(1.0 - lz, 1.0 - ly),
        Face::NegativeX => Vec2::new(lz, 1.0 - ly),
        Face::PositiveY => Vec2::new(lx, lz),
        Face::NegativeY => Vec2::new(lx, 1.0 - lz),
    }
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
    nearest_voxel_hit_where(world, ray, max_distance, |_| true)
}

/// Upper bound on hits discarded inside a single cell (a cut-out texel can
/// only ever reject a handful of faces of one small shape).
const MAX_REJECTED_HITS_PER_CELL: usize = 8;

/// Distance a ray is advanced past a discarded hit before the cell's
/// geometry is tested again, so the rejected surface is not hit twice.
const REJECTED_HIT_ADVANCE: f32 = shadows::SHADOW_EPSILON;

/// `nearest_voxel_hit` with an acceptance test: every geometric hit is shown
/// to `accept`, and a rejected hit (a cut-out, empty texel) is discarded so
/// the ray carries on — first through the rest of the same cell's geometry,
/// then through the DDA into the following cells — exactly as if that
/// surface point were air. `accept` always sees the full `VoxelHit` (cell,
/// block, geometric `HitRecord` with uv). Distances stay measured from the
/// original ray origin.
pub fn nearest_voxel_hit_where(
    world: &VoxelWorld,
    ray: &Ray,
    max_distance: f32,
    accept: impl Fn(&VoxelHit) -> bool,
) -> Option<VoxelHit> {
    if world.is_empty() || max_distance.is_nan() || max_distance <= 0.0 {
        return None;
    }

    let mut state = DdaState::from_ray(ray);

    for _ in 0..MAX_VOXEL_STEPS {
        if let Some(block) = world.get(state.cell) {
            let base_geometry = block_geometry(block.block_type(), block.orientation());
            let partial = !base_geometry.is_full_cube();
            let origin = cell_origin(state.cell);
            let geometry = base_geometry.translated(origin);
            let mut current = *ray;
            let mut offset = 0.0;

            for _ in 0..MAX_REJECTED_HITS_PER_CELL {
                let Some(mut hit) = geometry.intersect_local(&current, 0.0, max_distance - offset)
                else {
                    break;
                };
                hit.distance += offset;
                if partial {
                    hit.uv = cell_local_uv(hit.point - origin, hit.face);
                }

                let candidate = VoxelHit {
                    cell: state.cell,
                    block: *block,
                    hit,
                };
                if accept(&candidate) {
                    return Some(candidate);
                }

                offset = hit.distance + REJECTED_HIT_ADVANCE;
                current = Ray::new(ray.at(offset), ray.direction);
            }
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

/// Samples the material's albedo source at a hit: a material without
/// `face_textures` yields its uniform `albedo`; a material with
/// `face_textures` looks up the `TextureId` for `face`, resolves it through
/// `texture_manager` (already-loaded, no disk access here), and samples it at
/// `uv` with the project's nearest-neighbor sampler. `texture_manager` is a
/// shared reference, so this can never load a file: `TextureManager::load`
/// requires `&mut self`.
fn sample_albedo(
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

/// Resolves the ambient/diffuse albedo for a hit, returned with its alpha
/// forced to `1` (so shading can never leak a non-opaque framebuffer pixel)
/// together with the sampled texel alpha, which only `AlphaMode::Blend`
/// (per-texel opacity) and `AlphaMode::Cutout` (empty texels) interpret.
fn resolve_albedo(
    material: &Material,
    face: Face,
    uv: Vec2,
    texture_manager: &TextureManager,
) -> (Color, f32) {
    let sampled = sample_albedo(material, face, uv, texture_manager);

    (Color::new(sampled.r, sampled.g, sampled.b, 1.0), sampled.a)
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

/// The local-lighting result split into the pieces transparent materials
/// need: `full` is the classic ambient + diffuse + specular sum (what an
/// opaque surface shows, plus emission); `ambient`, `body` (ambient +
/// diffuse, no highlight, no emission) and `specular` let a transparent
/// surface keep its highlights and self-emission at full strength while its
/// body color is blended with what lies behind. `emissive` is the
/// UV-masked self-emitted radiance: it is independent of every light and
/// shadow, so an emissive texel stays visible in complete darkness.
/// `texel_alpha` is the albedo texel's alpha (see `AlphaMode`).
struct LocalShading {
    full: Color,
    ambient: Color,
    body: Color,
    specular: Color,
    emissive: Color,
    texel_alpha: f32,
}

/// Shared local lighting: ambient always applies; each light adds its
/// diffuse and specular contribution scaled by `visibility(ray,
/// max_distance)`, the fraction of that light reaching the surface along the
/// epsilon-offset shadow ray (`1` unobstructed, `0` fully blocked, in
/// between through transparent occluders). `visibility` is injected so both
/// the legacy cube list and the voxel world answer occlusion through the
/// same shading code.
/// `directional_range` bounds a directional light's shadow ray. `view_origin`
/// is where the viewing ray came from (the camera for primary rays, the
/// previous bounce point for secondary rays), used for the specular term.
fn shade_surface(
    surface: &SurfaceHit,
    view_origin: Vec3,
    lights: &[Light],
    ambient_factor: f32,
    texture_manager: &TextureManager,
    directional_range: f32,
    visibility: impl Fn(&Ray, f32) -> f32,
) -> LocalShading {
    let SurfaceHit {
        point,
        normal,
        material,
        ..
    } = *surface;

    let (albedo, texel_alpha) = resolve_albedo(material, surface.face, surface.uv, texture_manager);
    // Ambient/diffuse read albedo through this per-hit override; specular
    // never depends on albedo, so it is unaffected by texturing either way.
    let shading_material = Material::new(albedo, material.specular, material.shininess);

    // The normal-map-perturbed normal drives diffuse and specular; the
    // geometric `normal` keeps driving shadow-ray offsets and the "is this
    // light on the visible side" test below, so relief can never leak light
    // through the back of a surface. Without a normal texture the two are
    // identical.
    let shading_normal =
        sample_shading_normal(material, surface.face, surface.uv, normal, texture_manager);

    let view_direction = (view_origin - point).normalize();
    let ambient = shading::ambient(&shading_material, ambient_factor);
    // `full` keeps the original accumulation order; `body`/`specular` track
    // the same terms separately.
    let mut full = ambient;
    let mut body = ambient;
    let mut specular = Color::black();

    for light in lights {
        let (visible, diffuse_term, specular_term) = match light {
            Light::Directional(directional) => {
                if normal.dot(directional.direction) <= 0.0 {
                    continue;
                }
                let shadow_ray =
                    shadows::shadow_ray_to_directional_light(point, normal, directional);
                (
                    visibility(&shadow_ray, directional_range),
                    shading::diffuse_directional(&shading_material, shading_normal, directional),
                    shading::specular_directional(
                        &shading_material,
                        shading_normal,
                        view_direction,
                        directional,
                    ),
                )
            }
            Light::Point(point_light) => {
                if normal.dot(point_light.position - point) <= 0.0 {
                    continue;
                }
                let (shadow_ray, distance) =
                    shadows::shadow_ray_to_point_light(point, normal, point_light);
                (
                    visibility(&shadow_ray, distance),
                    shading::diffuse_point(&shading_material, shading_normal, point, point_light),
                    shading::specular_point(
                        &shading_material,
                        shading_normal,
                        point,
                        view_direction,
                        point_light,
                    ),
                )
            }
        };

        if visible > 0.0 {
            let diffuse_term = diffuse_term * visible;
            let specular_term = specular_term * visible;
            full = full + diffuse_term;
            full = full + specular_term;
            body = body + diffuse_term;
            specular = specular + specular_term;
        }
    }

    // Emission is added after (and regardless of) the lights and shadows.
    let emissive = sample_emissive(material, surface.uv, texture_manager);
    full = full + emissive;

    LocalShading {
        full: full.clamp(),
        ambient,
        body: body.clamp(),
        specular: specular.clamp(),
        emissive,
        texel_alpha,
    }
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
            if shadows::is_occluded(cubes.iter().copied(), shadow_ray, max_distance) {
                0.0
            } else {
                1.0
            }
        },
    )
    .full
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

/// Maximum number of secondary-ray bounces. The primary ray is depth `0`;
/// a ray traced at `depth >= MAX_RAY_DEPTH` contributes only local shading,
/// so recursion always terminates (primary -> secondary -> tertiary -> stop).
pub const MAX_RAY_DEPTH: u32 = 3;

/// Refractive index of the surrounding medium (air) that transparent
/// materials transition to and from.
pub const AIR_REFRACTIVE_INDEX: f32 = 1.0;

/// Offset applied along the surface normal to a secondary ray's origin so it
/// cannot immediately re-intersect the surface it leaves. It is the same
/// centralized constant the shadow rays use.
pub const SECONDARY_RAY_EPSILON: f32 = shadows::SHADOW_EPSILON;

/// Everything a voxel ray needs to be traced, bundled so recursion stays a
/// two-argument affair (`trace_ray(scene, ray, depth)`).
pub struct VoxelScene<'a> {
    pub world: &'a VoxelWorld,
    pub materials: &'a MaterialLibrary,
    pub camera_position: Vec3,
    pub lights: &'a [Light],
    pub ambient_factor: f32,
    pub background: Color,
    pub texture_manager: &'a TextureManager,
    pub max_distance: f32,
}

/// The geometric normal flipped, if needed, to face the side the ray comes
/// from (rays that start inside a shape report the exit face's outward
/// normal, which points along the ray).
fn facing_normal(ray: &Ray, normal: Vec3) -> Vec3 {
    if ray.direction.dot(normal) > 0.0 {
        -normal
    } else {
        normal
    }
}

/// Linear blend `a * (1 - t) + b * t`, clamped to the color range.
fn mix(a: Color, b: Color, t: f32) -> Color {
    (a * (1.0 - t) + b * t).clamp()
}

/// The integer step across a face of a full cube, to the neighboring cell.
fn face_step(face: Face) -> IVec3 {
    match face {
        Face::PositiveX => IVec3::new(1, 0, 0),
        Face::NegativeX => IVec3::new(-1, 0, 0),
        Face::PositiveY => IVec3::new(0, 1, 0),
        Face::NegativeY => IVec3::new(0, -1, 0),
        Face::PositiveZ => IVec3::new(0, 0, 1),
        Face::NegativeZ => IVec3::new(0, 0, -1),
    }
}

/// `true` when `material` is a continuous transmissive medium (water, glass):
/// transparent and not a cut-out.
fn is_transmissive_medium(material: &Material) -> bool {
    material.transparency > 0.0 && material.alpha_mode != AlphaMode::Cutout
}

/// Whether the hit is on a face shared by two contiguous cells of the *same*
/// transmissive medium (two `Water` voxels side by side). Such a boundary is
/// not an interface at all — light stays inside one body of water — so it must
/// neither refract, reflect, tint, nor darken a shadow. Only full cubes of the
/// same material qualify; a different neighbor, air, or a partial shape leaves
/// the face a real surface.
fn is_internal_medium_face(scene: &VoxelScene, voxel: &VoxelHit, material: &Material) -> bool {
    if !is_transmissive_medium(material) {
        return false;
    }

    let neighbor_cell = voxel.cell + face_step(voxel.hit.face);
    let Some(neighbor) = scene.world.get(neighbor_cell) else {
        return false;
    };

    neighbor.material_id() == voxel.block.material_id()
        && matches!(
            block_geometry(voxel.block.block_type(), voxel.block.orientation()),
            BlockGeometry::FullCube
        )
        && matches!(
            block_geometry(neighbor.block_type(), neighbor.orientation()),
            BlockGeometry::FullCube
        )
}

/// Whether a geometric voxel hit is a real surface point: `false` for a hit on
/// an `AlphaMode::Cutout` material at an empty texel (air), and for a face
/// shared by two contiguous voxels of the same transmissive medium (see
/// `is_internal_medium_face`). Unknown materials are accepted so the loud
/// missing-material path still triggers.
fn is_solid_hit(scene: &VoxelScene, voxel: &VoxelHit) -> bool {
    let Some(material) = scene.materials.get(voxel.block.material_id()) else {
        return true;
    };
    if is_internal_medium_face(scene, voxel, material) {
        return false;
    }
    if material.alpha_mode != AlphaMode::Cutout {
        return true;
    }

    let texel = sample_albedo(
        material,
        voxel.hit.face,
        voxel.hit.uv,
        scene.texture_manager,
    );
    !material.is_cut_out(texel.a)
}

/// The nearest *visible* voxel hit: `nearest_voxel_hit` where hits on empty
/// texels of cut-out materials (leaf holes) are ignored, so the ray carries
/// on behind them. Primary, secondary and shadow rays all go through here.
pub fn nearest_visible_hit(scene: &VoxelScene, ray: &Ray, max_distance: f32) -> Option<VoxelHit> {
    nearest_voxel_hit_where(scene.world, ray, max_distance, |voxel| {
        is_solid_hit(scene, voxel)
    })
}

/// Upper bound on surfaces a single shadow ray is followed through.
const MAX_SHADOW_HITS: usize = 8;

/// Fraction of a light that reaches the start of `ray` (a shadow ray) within
/// `max_distance`: `1.0` with nothing in the way, `0.0` when an opaque
/// surface blocks it. Empty cut-out texels are transparent to the light
/// (dappled foliage shadows), and a transparent occluder (glass, water)
/// dims the light by its effective transparency at that texel instead of
/// casting a hard black shadow. An unknown material blocks fully.
pub fn light_visibility(scene: &VoxelScene, ray: &Ray, max_distance: f32) -> f32 {
    let mut visibility = 1.0;
    let mut current = *ray;
    let mut remaining = max_distance;

    for _ in 0..MAX_SHADOW_HITS {
        let Some(voxel) = nearest_visible_hit(scene, &current, remaining) else {
            break;
        };

        let transparency = scene
            .materials
            .get(voxel.block.material_id())
            .map_or(0.0, |material| {
                let texel = sample_albedo(
                    material,
                    voxel.hit.face,
                    voxel.hit.uv,
                    scene.texture_manager,
                );
                material.effective_transparency(texel.a).clamp(0.0, 1.0)
            });

        visibility *= transparency;
        if visibility <= 0.0 {
            return 0.0;
        }

        let advance = voxel.hit.distance + REJECTED_HIT_ADVANCE;
        remaining -= advance;
        if remaining <= 0.0 {
            break;
        }
        current = Ray::new(current.at(advance), current.direction);
    }

    visibility
}

/// Traces `ray` through the scene at recursion `depth`.
///
/// The nearest real voxel hit is shaded locally (ambient + diffuse + specular
/// with hard shadows). When the hit material has `reflectivity > 0` and
/// `depth < MAX_RAY_DEPTH`, a mirror ray is spawned from the hit point offset
/// by `SECONDARY_RAY_EPSILON` along the surface normal, traced recursively
/// through the same DDA path, and blended:
///
/// `final = local * (1 - reflectivity) + reflected * reflectivity`
///
/// A material with `reflectivity == 0` (or a ray at the depth limit) returns
/// its local color untouched. A miss returns `background`; a voxel whose
/// `MaterialId` is unknown returns `MISSING_MATERIAL_COLOR`.
pub fn trace_ray(scene: &VoxelScene, ray: &Ray, depth: u32) -> Color {
    let Some(voxel) = nearest_visible_hit(scene, ray, scene.max_distance) else {
        return scene.background;
    };

    let Some(material) = scene.materials.get(voxel.block.material_id()) else {
        return MISSING_MATERIAL_COLOR;
    };

    // A cut-out surface can be reached from the back, through a hole in the
    // near face: light it from the side the ray actually sees.
    let shading_normal = if material.alpha_mode == AlphaMode::Cutout {
        facing_normal(ray, voxel.hit.normal)
    } else {
        voxel.hit.normal
    };

    let surface = SurfaceHit {
        point: voxel.hit.point,
        normal: shading_normal,
        face: voxel.hit.face,
        uv: voxel.hit.uv,
        material,
    };

    // Primary rays keep the camera-based view vector; secondary rays view
    // the surface from where they were spawned.
    let view_origin = if depth == 0 {
        scene.camera_position
    } else {
        ray.origin
    };

    let local = shade_surface(
        &surface,
        view_origin,
        scene.lights,
        scene.ambient_factor,
        scene.texture_manager,
        shadows::DIRECTIONAL_SHADOW_RANGE.min(scene.max_distance),
        |shadow_ray, shadow_distance| light_visibility(scene, shadow_ray, shadow_distance),
    );

    let transparency = material
        .effective_transparency(local.texel_alpha)
        .clamp(0.0, 1.0);
    let reflectivity = material.reflectivity;

    if depth >= MAX_RAY_DEPTH || (reflectivity <= 0.0 && transparency <= 0.0) {
        return local.full;
    }

    let point = voxel.hit.point;
    let normal = facing_normal(ray, voxel.hit.normal);
    let reflected_ray = Ray::new(
        point + normal * SECONDARY_RAY_EPSILON,
        reflect(ray.direction, normal),
    );
    let mut reflected: Option<Color> = None;
    let mut surface_color = local.full;

    if transparency > 0.0 {
        // Leaving the medium when the ray travels along the outward normal
        // (it started inside the shape); otherwise entering it.
        let exiting = ray.direction.dot(voxel.hit.normal) > 0.0;
        let (eta_i, eta_t) = if exiting {
            (material.refractive_index, AIR_REFRACTIVE_INDEX)
        } else {
            (AIR_REFRACTIVE_INDEX, material.refractive_index)
        };

        let transmitted = match refract(ray.direction, normal, eta_i, eta_t) {
            Some(direction) => {
                let transmitted_ray = Ray::new(point - normal * SECONDARY_RAY_EPSILON, direction);
                trace_ray(scene, &transmitted_ray, depth + 1)
            }
            // Total internal reflection: the energy stays in the medium.
            None => *reflected.insert(trace_ray(scene, &reflected_ray, depth + 1)),
        };

        // The back face of a medium is only seen through it: tint with the
        // ambient body color and skip lit body, highlights and emission there
        // (a self-luminous membrane such as the portal core is counted once,
        // on the face the viewer meets first).
        let (body, specular, emissive) = if exiting {
            (local.ambient, Color::black(), Color::black())
        } else {
            (local.body, local.specular, local.emissive)
        };
        surface_color =
            (body * (1.0 - transparency) + transmitted * transparency + specular + emissive)
                .clamp();
    }

    if reflectivity > 0.0 {
        let reflected = match reflected {
            Some(color) => color,
            None => trace_ray(scene, &reflected_ray, depth + 1),
        };
        surface_color = mix(surface_color, reflected, reflectivity);
    }

    Color::new(surface_color.r, surface_color.g, surface_color.b, 1.0)
}

/// Full voxel render path for one primary ray: `VoxelWorld` + 3D DDA finds
/// the first real voxel hit, its `MaterialId` is resolved through the
/// `MaterialLibrary`, the albedo comes from the same `FaceTextures` /
/// `sample_nearest(hit.uv)` route as before, and ambient + diffuse +
/// specular are evaluated by the shared `shade_surface`. Shadow rays query
/// the *same* `world` through `is_voxel_occluded`. Reflective materials
/// additionally bounce through `trace_ray` (see there).
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
    let scene = VoxelScene {
        world,
        materials,
        camera_position,
        lights,
        ambient_factor,
        background,
        texture_manager,
        max_distance,
    };

    trace_ray(&scene, ray, 0)
}

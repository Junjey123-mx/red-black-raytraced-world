#[path = "."]
mod core {
    #[path = "."]
    pub mod math {
        #[path = "../src/core/math/ivec3.rs"]
        pub mod ivec3;
        #[path = "../src/core/math/vec2.rs"]
        pub mod vec2;
        #[path = "../src/core/math/vec3.rs"]
        pub mod vec3;

        pub use ivec3::IVec3;
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
    #[path = "../src/core/material.rs"]
    pub mod material;
    #[path = "../src/core/prism.rs"]
    pub mod prism;
    #[path = "../src/core/ray.rs"]
    pub mod ray;
    #[path = "../src/core/reflection.rs"]
    pub mod reflection;
    #[path = "../src/core/refraction.rs"]
    pub mod refraction;
    #[path = "../src/core/texture.rs"]
    pub mod texture;
}

#[path = "."]
mod camera {
    #[path = "../src/camera/camera.rs"]
    pub mod camera;
    #[path = "../src/camera/diagnostic.rs"]
    pub mod diagnostic;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
}

#[path = "."]
mod scene {
    #[path = "../src/scene/block.rs"]
    pub mod block;
    #[path = "../src/scene/block_geometry.rs"]
    pub mod block_geometry;
    #[path = "../src/scene/block_shape_factory.rs"]
    pub mod block_shape_factory;
    #[path = "../src/scene/block_type.rs"]
    pub mod block_type;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/geometry_orientation.rs"]
    pub mod geometry_orientation;
    #[path = "../src/scene/light.rs"]
    pub mod light;
    #[path = "../src/scene/material_gallery.rs"]
    pub mod material_gallery;
    #[path = "../src/scene/material_library.rs"]
    pub mod material_library;
    #[path = "../src/scene/orientation.rs"]
    pub mod orientation;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use camera::diagnostic::DiagnosticCameraState;
use camera::projection::primary_ray;
use core::color::Color;
use core::hit::Face;
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::framebuffer::Framebuffer;
use renderer::raytracer::{
    MISSING_MATERIAL_COLOR, VoxelScene, cast_ray_voxel_lit, nearest_visible_hit, nearest_voxel_hit,
};
use scene::block::BlockInstance;
use scene::block_geometry::BlockGeometry;
use scene::block_shape_factory::block_geometry;
use scene::block_type::BlockType;
use scene::catalog::*;
use scene::material_gallery::{
    BACK_Z, GALLERY_MAX_DISTANCE, GALLERY_WIDTH, GALLERY_Z_MIN, LEAVES_X, gallery_background,
    gallery_camera, gallery_lights, glass_material_id, leaves_material_id, portal_core_material_id,
    redstone_lamp_material_id,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 60.0;
/// Number of catalog samples (updated as definitive blocks join the catalog).
const ENTRY_COUNT: usize = 22;

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}

struct Env {
    manager: TextureManager,
    library: MaterialLibrary,
}

fn env() -> Env {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let library = catalog_materials(&textures);
    Env { manager, library }
}

fn entry(name: &str) -> CatalogEntry {
    catalog_entries()
        .into_iter()
        .find(|e| e.display_name == name)
        .unwrap_or_else(|| panic!("no entry {name}"))
}

/// A private world holding only one entry (and its extra cells), so a ray can
/// be aimed at it without the rest of the catalog in the way.
fn world_with(entry: &CatalogEntry) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(entry.position, entry.block());
    for (c, b) in &entry.extra_blocks {
        world.insert(*c, *b);
    }
    world
}

// ---- catalog contents -------------------------------------------------

#[test]
fn the_catalog_is_not_empty_and_has_every_current_sample() {
    let scene = CatalogScene::new();
    assert!(!scene.is_empty());
    assert_eq!(scene.len(), ENTRY_COUNT);
    assert_eq!(scene.entries().len(), ENTRY_COUNT);
}

#[test]
fn positions_are_deterministic_unique_and_inside_the_floor() {
    let a = catalog_entries();
    let b = catalog_entries();
    assert_eq!(a, b);

    let mut seen = std::collections::HashSet::new();
    for e in &a {
        assert!(
            seen.insert((e.position.x, e.position.y, e.position.z)),
            "{}",
            e.display_name
        );
        assert!((0..GALLERY_WIDTH).contains(&e.position.x));
        assert!((GALLERY_Z_MIN - 6..9).contains(&e.position.z));
        assert_eq!(e.position.y, 1, "samples stand on the floor");
    }
    let s1 = CatalogScene::new();
    let s2 = CatalogScene::new();
    assert_eq!(s1.world().len(), s2.world().len());
}

#[test]
fn entries_have_names_and_finite_focus_points_near_their_cells() {
    for e in catalog_entries() {
        assert!(!e.display_name.is_empty());
        assert!(finite(e.focus_point), "{}", e.display_name);
        let c = Vec3::new(
            e.position.x as f32,
            e.position.y as f32,
            e.position.z as f32,
        );
        assert!((e.focus_point - c).length() < 2.0, "{}", e.display_name);
    }
    let mut names: Vec<_> = catalog_entries().iter().map(|e| e.display_name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), ENTRY_COUNT, "names are unique");
}

#[test]
fn every_partial_geometry_sample_is_present() {
    let kinds: Vec<_> = catalog_entries()
        .iter()
        .filter(|e| e.kind == SampleKind::PartialGeometry)
        .map(|e| e.block_type)
        .collect();
    for t in [
        BlockType::WoodStairs,
        BlockType::Fence,
        BlockType::WoodDoor,
        BlockType::PortalCoreDarkCrimson,
        BlockType::AmethystCluster,
    ] {
        assert!(kinds.contains(&t), "{t:?}");
    }
}

#[test]
fn stairs_fence_door_portal_and_amethyst_resolve_partial_geometry_not_full_cubes() {
    for (name, composite) in [
        ("Wood stairs", true),
        ("Fence", true),
        ("WoodDoor", false),
        ("Portal core", false),
        ("Amethyst cluster", true),
    ] {
        let e = entry(name);
        let geometry = block_geometry(e.block_type, e.orientation);
        assert!(
            !matches!(geometry, BlockGeometry::FullCube),
            "{name} is a full cube"
        );
        if composite {
            assert!(matches!(geometry, BlockGeometry::Composite(_)), "{name}");
        } else {
            assert!(matches!(geometry, BlockGeometry::Prism(_)), "{name}");
        }
    }
}

#[test]
fn the_required_optical_samples_are_present() {
    let entries = catalog_entries();
    let has = |k: SampleKind| entries.iter().any(|e| e.kind == k);
    for k in [
        SampleKind::Control,
        SampleKind::Reflective,
        SampleKind::Transmissive,
        SampleKind::Cutout,
        SampleKind::Emissive,
        SampleKind::NormalMapped,
    ] {
        assert!(has(k), "{k:?}");
    }
    let types: Vec<_> = entries.iter().map(|e| e.block_type).collect();
    for t in [
        BlockType::Glass,
        BlockType::Water,
        BlockType::Leaves,
        BlockType::RedstoneLampLit,
    ] {
        assert!(types.contains(&t), "{t:?}");
    }
}

// ---- selection and focus ---------------------------------------------

#[test]
fn selection_moves_to_the_next_sample() {
    let mut s = CatalogScene::new();
    assert_eq!(s.selected_index(), 0);
    s.select_next();
    assert_eq!(s.selected_index(), 1);
    assert_eq!(s.selected().display_name, s.entries()[1].display_name);
}

#[test]
fn selection_moves_to_the_previous_sample() {
    let mut s = CatalogScene::new();
    s.select_next();
    s.select_next();
    s.select_previous();
    assert_eq!(s.selected_index(), 1);
}

#[test]
fn selection_wraps_in_both_directions() {
    let mut s = CatalogScene::new();
    s.select_previous();
    assert_eq!(
        s.selected_index(),
        s.len() - 1,
        "previous from first -> last"
    );
    s.select_next();
    assert_eq!(s.selected_index(), 0, "next from last -> first");

    for _ in 0..s.len() {
        s.select_next();
    }
    assert_eq!(s.selected_index(), 0, "a full cycle returns to the start");
}

#[test]
fn the_label_reports_the_selection() {
    let mut s = CatalogScene::new();
    s.select_next();
    let label = s.label();
    assert!(label.contains(s.selected().display_name));
    assert!(label.contains(&format!("2 / {ENTRY_COUNT}")), "{label}");
}

#[test]
fn focus_returns_a_finite_target_for_every_sample() {
    let mut s = CatalogScene::new();
    for _ in 0..s.len() {
        assert!(finite(s.focus_target()));
        s.select_next();
    }
}

#[test]
fn focus_moves_the_camera_to_the_selected_sample_only() {
    let overview = gallery_camera(4.0 / 3.0);
    let mut view = DiagnosticCameraState::from_pose(overview.position, overview.target);
    view.set_angles(0.8, 0.5);
    let (yaw, pitch) = (view.yaw, view.pitch);

    let mut s = CatalogScene::new();
    for i in 0..s.len() {
        let before = s.world().len();
        s.focus_camera(&mut view);

        assert_eq!(view.target, s.selected().focus_point, "entry {i}");
        assert_eq!(view.target, s.focus_target());
        assert!((view.distance - FOCUS_DISTANCE).abs() < EPS);
        assert_eq!(
            (view.yaw, view.pitch),
            (yaw, pitch),
            "orbit angles are kept"
        );
        assert_eq!(
            s.world().len(),
            before,
            "focusing never moves or adds blocks"
        );
        s.select_next();
    }
}

#[test]
fn reset_returns_to_the_overview_not_to_the_selected_sample() {
    let overview = gallery_camera(4.0 / 3.0);
    let mut view = DiagnosticCameraState::from_pose(overview.position, overview.target);
    let initial = view;

    let mut s = CatalogScene::new();
    s.select_next();
    s.focus_camera(&mut view);
    assert_ne!(view.target, initial.target);

    view.reset();
    assert_eq!(view, initial);
    assert_eq!(s.selected_index(), 1, "reset does not touch the selection");
}

// ---- the catalog is the real world, materials and textures -------------

#[test]
fn the_catalog_builds_a_real_voxel_world_containing_every_entry() {
    let s = CatalogScene::new();
    for e in s.entries() {
        assert_eq!(
            s.world().get(e.position),
            Some(&e.block()),
            "{}",
            e.display_name
        );
        for (c, b) in &e.extra_blocks {
            assert_eq!(s.world().get(*c), Some(b), "{} extra", e.display_name);
        }
    }
    assert!(
        s.world().len() > s.entries().len(),
        "floor and backdrops are kept"
    );
}

#[test]
fn the_catalog_has_no_geometry_list_parallel_to_the_world() {
    let source = std::fs::read_to_string(format!(
        "{}/src/scene/catalog.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let code: String = source
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        );

    // Placement goes through BlockInstance/VoxelWorld only: no shapes, no
    // Materials or block types of its own.
    for forbidden in [
        "Prism::new",
        "Aabb",
        "Cube::new",
        "CatalogBlockType",
        "CatalogMaterial",
        "CatalogRenderer",
        "Material::new",
    ] {
        assert!(
            !code.contains(forbidden),
            "catalog.rs must not contain {forbidden}"
        );
    }
    assert!(code.contains("advanced_materials_world()"));
    assert!(code.contains("world.insert(entry.position, entry.block())"));
}

#[test]
fn every_material_id_used_by_the_world_resolves() {
    let e = env();
    let s = CatalogScene::new();
    let mut checked = 0;
    for x in -1..GALLERY_WIDTH + 1 {
        for y in -1..4 {
            for z in GALLERY_Z_MIN - 8..10 {
                if let Some(b) = s.world().get(cell(x, y, z)) {
                    assert!(e.library.contains(b.material_id()), "({x},{y},{z})");
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, s.world().len());
}

#[test]
fn every_texture_referenced_by_the_samples_is_loaded() {
    let e = env();
    let faces = [
        Face::PositiveX,
        Face::NegativeX,
        Face::PositiveY,
        Face::NegativeY,
        Face::PositiveZ,
        Face::NegativeZ,
    ];
    let s = CatalogScene::new();
    let mut ids: Vec<MaterialId> = s.entries().iter().map(|x| x.material_id).collect();
    for x in s.entries() {
        ids.extend(x.extra_blocks.iter().map(|(_, b)| b.material_id()));
    }

    for id in ids {
        let m = e.library.get(id).expect("material resolves");
        if let Some(ft) = m.face_textures {
            for f in faces {
                assert!(e.manager.get(ft.texture_for_face(f)).is_some());
            }
        }
        for t in [m.emissive_texture, m.normal_texture].into_iter().flatten() {
            assert!(e.manager.get(t).is_some());
        }
    }
}

#[test]
fn the_samples_keep_their_gate_06_and_gate_07_materials() {
    let e = env();
    let get = |name: &str| e.library.get(entry(name).material_id).unwrap().clone();

    let glass = get("Glass");
    assert!(glass.transparency > 0.5 && (glass.refractive_index - 1.5).abs() < EPS);
    let water = get("Water");
    assert!(water.transparency > 0.5 && (water.refractive_index - 1.33).abs() < EPS);
    assert!(water.reflectivity > 0.0);
    assert_eq!(get("Leaves").alpha_mode, AlphaMode::Cutout);
    let lamp = get("Redstone Lamp (lit)");
    assert!(lamp.emissive_texture.is_some() && lamp.emission_strength >= 1.8);
    let portal = get("Portal core");
    assert!(portal.emissive_texture.is_some() && portal.transparency > 0.0);
    assert!(get("Reflective cube").reflectivity > 0.3);
    assert!(get("Deepslate Bricks").normal_texture.is_some());
    // Gate 06 shape materials keep their textures.
    assert!(get("Wood stairs").face_textures.is_some());
    assert!(get("WoodDoor").face_textures.is_some());
    assert_eq!(e.library.get(portal_core_material_id()).unwrap(), &portal);
    assert_eq!(e.library.get(glass_material_id()).unwrap(), &glass);
    assert_eq!(e.library.get(redstone_lamp_material_id()).unwrap(), &lamp);
}

// ---- integration: catalog -> VoxelWorld -> DDA -> geometry -> materials ----

#[test]
fn a_fence_post_is_hit_and_a_ray_through_a_gap_continues() {
    let fence = entry("Fence");
    let mut world = world_with(&fence);
    let behind = cell(fence.position.x, 1, fence.position.z - 3);
    world.insert(
        behind,
        BlockInstance::new(BlockType::Stone, MaterialId::new(2), Orientation::Up),
    );
    let x = fence.position.x as f32;
    let z0 = fence.position.z as f32 + 6.0;

    // Straight at the post (below the rails).
    let post = ray((x + 0.5, 1.2, z0), (0.0, 0.0, -1.0));
    let hit = nearest_voxel_hit(&world, &post, RANGE).expect("post must be hit");
    assert_eq!(hit.cell, fence.position);
    assert!(hit.hit.distance > 5.0 && hit.hit.distance < 5.5);

    // Beside the post: the cell is occupied but the ray finds only air, so
    // traversal continues and reaches the block behind.
    let gap = ray((x + 0.15, 1.2, z0), (0.0, 0.0, -1.0));
    let hit = nearest_voxel_hit(&world, &gap, RANGE).expect("gap ray reaches the block behind");
    assert_eq!(hit.cell, behind);
}

#[test]
fn a_door_leaf_is_hit_but_the_rest_of_its_cell_is_air() {
    let door = entry("WoodDoor");
    let world = world_with(&door);
    let x = door.position.x as f32;
    let z = door.position.z as f32;

    let leaf = ray((x + 0.5, 1.5, z + 6.0), (0.0, 0.0, -1.0));
    let hit = nearest_voxel_hit(&world, &leaf, RANGE).expect("the leaf must be hit");
    assert_eq!(hit.cell, door.position);
    assert_eq!(hit.hit.face, Face::PositiveZ);
    assert!(
        (hit.hit.point.z - (z + 1.0)).abs() < EPS,
        "leaf sits on the cell's +Z edge"
    );

    // Sideways through the empty part of the cell: no false full-cube hit.
    let across = ray((x - 3.0, 1.5, z + 0.4), (1.0, 0.0, 0.0));
    assert!(nearest_voxel_hit(&world, &across, RANGE).is_none());
    // The upper half of the door is the second cell.
    assert!(
        world
            .get(cell(door.position.x, 2, door.position.z))
            .is_some()
    );
}

#[test]
fn a_leaves_cutout_texel_lets_the_ray_continue() {
    let e = env();
    let scene = CatalogScene::new();
    let leaves = entry("Leaves");
    let texture_id = e
        .library
        .get(leaves_material_id())
        .unwrap()
        .face_textures
        .unwrap()
        .texture_for_face(Face::PositiveZ);
    let texture = e.manager.get(texture_id).unwrap();
    let solid = |i: usize, j: usize| texture.texel(i, j).unwrap().a >= 0.5;
    let (i, j) = (0..256)
        .map(|n| (n % 16, n / 16))
        .find(|&(i, j)| !solid(i, j) && !solid(15 - i, j))
        .expect("a see-through column exists");

    let lx = LEAVES_X as f32 + (i as f32 + 0.5) / 16.0;
    let ly = 1.0 + 1.0 - (j as f32 + 0.5) / 16.0;
    let r = ray((lx, ly, 4.0), (0.0, 0.0, -1.0));
    let vscene = VoxelScene {
        world: scene.world(),
        materials: &e.library,
        camera_position: r.origin,
        lights: &[],
        ambient_factor: 1.0,
        background: Color::black(),
        texture_manager: &e.manager,
        max_distance: RANGE,
    };

    // Geometry alone stops at the leaves; the visible-hit query does not.
    assert_eq!(
        nearest_voxel_hit(scene.world(), &r, RANGE).unwrap().cell,
        leaves.position
    );
    let hit = nearest_visible_hit(&vscene, &r, RANGE).expect("the block behind the hole");
    assert_eq!(hit.cell, cell(LEAVES_X, 1, BACK_Z - 1));
}

#[test]
fn glass_resolves_a_transparent_refractive_material_through_the_dda() {
    let e = env();
    let scene = CatalogScene::new();
    let glass = entry("Glass");
    let r = ray(
        (
            glass.position.x as f32 + 0.5,
            1.5,
            glass.position.z as f32 + 6.0,
        ),
        (0.0, 0.0, -1.0),
    );

    let hit = nearest_voxel_hit(scene.world(), &r, RANGE).unwrap();
    assert_eq!(hit.cell, glass.position);
    let m: &Material = e.library.get(hit.block.material_id()).unwrap();
    assert!(m.transparency > 0.0 && m.refractive_index > 1.0);
}

#[test]
fn the_portal_core_is_a_thin_emissive_membrane() {
    let e = env();
    let portal = entry("Portal core");
    let world = world_with(&portal);
    let x = portal.position.x as f32;
    let z = portal.position.z as f32;

    // Head-on through the middle of the cell: hits the membrane.
    let front = ray((x + 0.5, 1.5, z + 6.0), (0.0, 0.0, -1.0));
    let hit = nearest_voxel_hit(&world, &front, RANGE).expect("membrane hit");
    assert!(
        (hit.hit.point.z - (z + 0.5625)).abs() < EPS,
        "thin slab in the middle of the cell"
    );
    // Beside it, inside the cell but off the slab: air.
    let miss = ray((x - 2.0, 1.5, z + 0.1), (1.0, 0.0, 0.0));
    assert!(nearest_voxel_hit(&world, &miss, RANGE).is_none());

    let m = e.library.get(portal.material_id).unwrap();
    assert!(m.emissive_texture.is_some() && m.emission_strength > 2.0 && m.transparency > 0.0);
}

#[test]
fn the_whole_catalog_renders_a_finite_frame_from_every_focus() {
    let e = env();
    let scene = CatalogScene::new();
    let lights = gallery_lights();
    let overview = gallery_camera(4.0 / 3.0);
    let mut view = DiagnosticCameraState::from_pose(overview.position, overview.target);

    let mut s = CatalogScene::new();
    for i in 0..=s.len() {
        if i > 0 {
            s.select_next();
            s.focus_camera(&mut view);
        }
        let camera = view.build_camera(4.0 / 3.0);
        let (w, h) = (48, 36);
        let mut fb = Framebuffer::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let r = primary_ray(&camera, x, y, w, h);
                let c = cast_ray_voxel_lit(
                    scene.world(),
                    &e.library,
                    &r,
                    camera.position,
                    &lights,
                    0.1,
                    gallery_background(),
                    &e.manager,
                    GALLERY_MAX_DISTANCE,
                );
                assert!(c.r.is_finite() && c.g.is_finite() && c.b.is_finite());
                assert!((0.0..=1.0).contains(&c.r) && (0.0..=1.0).contains(&c.g));
                assert_ne!(c, MISSING_MATERIAL_COLOR);
                fb.set_pixel(x, y, c);
            }
        }
        assert_eq!(fb.pixels().len(), w * h);
    }
}

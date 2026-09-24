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

use camera::projection::primary_ray;
use core::color::Color;
use core::hit::Face;
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::framebuffer::Framebuffer;
use renderer::normal_mapping::sample_shading_normal;
use renderer::raytracer::{
    MAX_RAY_DEPTH, MISSING_MATERIAL_COLOR, VoxelScene, cast_ray_voxel_lit, light_visibility,
    nearest_visible_hit, nearest_voxel_hit, trace_ray,
};
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::*;
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;

const EPS: f32 = 1e-4;
const RANGE: f32 = 60.0;
const N: usize = 16;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

fn approx_color(a: Color, b: Color) -> bool {
    approx(a.r, b.r) && approx(a.g, b.g) && approx(a.b, b.b)
}

fn cell(x: i32, y: i32, z: i32) -> IVec3 {
    IVec3::new(x, y, z)
}

fn ray(origin: (f32, f32, f32), direction: (f32, f32, f32)) -> Ray {
    Ray::new(
        Vec3::new(origin.0, origin.1, origin.2),
        Vec3::new(direction.0, direction.1, direction.2),
    )
}

fn black() -> Color {
    Color::new(0.0, 0.0, 0.0, 1.0)
}

fn finite(c: Color) -> bool {
    c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && (0.0..=1.0).contains(&c.r)
}

struct Gallery {
    manager: TextureManager,
    library: MaterialLibrary,
}

fn gallery() -> Gallery {
    let mut manager = TextureManager::new();
    let textures = GalleryTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
    )
    .unwrap();
    let library = advanced_materials_library(&textures);
    Gallery { manager, library }
}

impl Gallery {
    fn with_extra(mut self, id: u32, material: Material) -> Self {
        self.library.insert(MaterialId::new(id), material);
        self
    }

    fn trace(
        &self,
        world: &VoxelWorld,
        lights: &[Light],
        ambient: f32,
        background: Color,
        r: &Ray,
        depth: u32,
    ) -> Color {
        let scene = VoxelScene {
            world,
            materials: &self.library,
            camera_position: r.origin,
            lights,
            ambient_factor: ambient,
            background,
            texture_manager: &self.manager,
            max_distance: RANGE,
        };
        trace_ray(&scene, r, depth)
    }
}

fn block(block_type: BlockType, id: MaterialId) -> BlockInstance {
    BlockInstance::new(block_type, id, Orientation::Up)
}

fn lone(block_type: BlockType, id: MaterialId) -> VoxelWorld {
    let mut world = VoxelWorld::new();
    world.insert(cell(0, 0, 0), block(block_type, id));
    world
}

// ---------------------------------------------------------------------
// 1. Matte control keeps the classic ambient + diffuse (+ specular) result.
// ---------------------------------------------------------------------

#[test]
fn a_matte_control_keeps_its_legacy_shading() {
    let g = gallery();
    let matte = g.library.get(matte_material_id()).unwrap();
    assert_eq!(matte.reflectivity, 0.0);
    assert_eq!(matte.transparency, 0.0);
    assert_eq!(matte.emission_strength, 0.0);
    assert!(matte.emissive_texture.is_none() && matte.normal_texture.is_none());
    assert_eq!(matte.alpha_mode, AlphaMode::Ignore);

    // Top face lit head-on: albedo * (ambient + intensity * N.L), no specular.
    let world = lone(BlockType::Stone, matte_material_id());
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Color::white(),
        1.0,
    ))];
    let color = g.trace(
        &world,
        &lights,
        0.1,
        black(),
        &ray((0.5, 5.0, 0.5), (0.0, -1.0, 0.0)),
        0,
    );
    let albedo = matte.albedo;
    assert!(approx_color(
        color,
        Color::new(albedo.r * 1.1, albedo.g * 1.1, albedo.b * 1.1, 1.0)
    ));
}

// ---------------------------------------------------------------------
// 2. Reflection launches secondary rays.
// ---------------------------------------------------------------------

#[test]
fn a_reflective_specimen_shows_the_scene_in_front_of_it_and_a_matte_one_does_not() {
    let g = gallery();
    let world = advanced_materials_world();
    let lights: [Light; 0] = [];
    // Eye ray aimed at the mirror's +Z face; its bounce lands on the yellow
    // patch, which is therefore visible in the mirror.
    let eye = Vec3::new(6.0, 5.6, 14.0);
    let target = Vec3::new(MIRROR_X as f32 + 0.5, 1.5, FRONT_Z as f32 + 1.0);
    let mirror_ray = Ray::new(eye, target - eye);
    let matte_target = Vec3::new(MATTE_X as f32 + 0.5, 1.5, FRONT_Z as f32 + 1.0);
    let matte_ray = Ray::new(eye, matte_target - eye);

    let mirror = g.trace(&world, &lights, 0.5, black(), &mirror_ray, 0);
    let matte = g.trace(&world, &lights, 0.5, black(), &matte_ray, 0);
    let patch = g
        .library
        .get(reflection_target_material_id())
        .unwrap()
        .albedo;

    // The mirror picks up the yellow patch's hue; the matte cube does not.
    assert!(
        mirror.r > mirror.b + 0.15 && mirror.g > mirror.b + 0.1,
        "mirror = {mirror:?}"
    );
    assert!(patch.r > patch.b);
    assert!((matte.r - matte.b).abs() < 0.05, "matte = {matte:?}");
}

// ---------------------------------------------------------------------
// 3 / 4. Transparency and refraction, glass vs water.
// ---------------------------------------------------------------------

#[test]
fn glass_and_water_show_the_backdrop_behind_them_and_use_different_indices() {
    let g = gallery();
    let glass = g.library.get(glass_material_id()).unwrap();
    let water = g.library.get(water_material_id()).unwrap();

    assert!((glass.refractive_index - 1.5).abs() < EPS);
    assert!((water.refractive_index - 1.33).abs() < EPS);
    assert_ne!(glass.refractive_index, water.refractive_index);
    assert!(glass.transparency > 0.0 && water.transparency > 0.0);

    let world = advanced_materials_world();
    // Straight through the centers of the glass and water cubes toward the
    // red and blue backdrop cubes.
    let through = |x: i32| {
        g.trace(
            &world,
            &[],
            1.0,
            black(),
            &ray((x as f32 + 0.5, 1.5, 12.0), (0.0, 0.0, -1.0)),
            0,
        )
    };
    let through_glass = through(GLASS_X);
    let through_water = through(WATER_X);

    assert!(
        through_glass.r > 2.0 * through_glass.b,
        "glass shows red: {through_glass:?}"
    );
    assert!(
        through_water.b > 2.0 * through_water.r,
        "water shows blue: {through_water:?}"
    );
}

// ---------------------------------------------------------------------
// 5 / 6. Alpha cutout: primary hits and shadow rays.
// ---------------------------------------------------------------------

fn leaves_texture_solid(g: &Gallery) -> Vec<Vec<bool>> {
    let id = g
        .library
        .get(leaves_material_id())
        .unwrap()
        .face_textures
        .unwrap()
        .texture_for_face(Face::PositiveZ);
    let texture = g.manager.get(id).unwrap();
    (0..N)
        .map(|y| {
            (0..N)
                .map(|x| texture.texel(x, y).unwrap().a >= 0.5)
                .collect()
        })
        .collect()
}

#[test]
fn alpha_cutout_lets_a_ray_reach_the_block_behind_the_leaves() {
    let g = gallery();
    let world = advanced_materials_world();
    let solid = leaves_texture_solid(&g);
    let (i, j) = (0..N * N)
        .map(|n| (n % N, n / N))
        .find(|&(i, j)| !solid[j][i] && !solid[j][N - 1 - i])
        .expect("a see-through column exists");
    let lx = LEAVES_X as f32 + (i as f32 + 0.5) / N as f32;
    let ly = 1.0 + 1.0 - (j as f32 + 0.5) / N as f32;
    // Start between the front row and the leaves so no other specimen is in the way.
    let r = ray((lx, ly, 4.0), (0.0, 0.0, -1.0));

    let hit = nearest_visible_hit(
        &VoxelScene {
            world: &world,
            materials: &g.library,
            camera_position: r.origin,
            lights: &[],
            ambient_factor: 1.0,
            background: black(),
            texture_manager: &g.manager,
            max_distance: RANGE,
        },
        &r,
        RANGE,
    )
    .expect("something lies behind the hole");

    assert_eq!(
        hit.cell,
        cell(LEAVES_X, 1, BACK_Z - 1),
        "the red cube behind the leaves"
    );
    assert_eq!(
        g.trace(&world, &[], 1.0, black(), &r, 0),
        g.library.get(backdrop_red_material_id()).unwrap().albedo
    );
}

#[test]
fn alpha_cutout_also_governs_shadow_rays() {
    let g = gallery();
    let solid = leaves_texture_solid(&g);
    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        block(BlockType::Stone, backdrop_red_material_id()),
    );
    world.insert(
        cell(0, 2, 0),
        block(BlockType::Leaves, leaves_material_id()),
    );
    let scene = VoxelScene {
        world: &world,
        materials: &g.library,
        camera_position: Vec3::zero(),
        lights: &[],
        ambient_factor: 1.0,
        background: black(),
        texture_manager: &g.manager,
        max_distance: RANGE,
    };
    let shadow_ray = |i: usize, j_top: usize| {
        ray(
            ((i as f32 + 0.5) / 16.0, 1.0001, (j_top as f32 + 0.5) / 16.0),
            (0.0, 1.0, 0.0),
        )
    };

    let mut through = 0;
    let mut blocked = 0;
    for j in 0..N {
        for i in 0..N {
            // Top face texel (i, j), bottom face texel (i, N-1-j).
            let v = light_visibility(&scene, &shadow_ray(i, j), 10.0);
            if !solid[j][i] && !solid[N - 1 - j][i] {
                assert!(approx(v, 1.0));
                through += 1;
            } else if solid[N - 1 - j][i] {
                assert!(approx(v, 0.0));
                blocked += 1;
            }
        }
    }
    assert!(through > 0 && blocked > 0);
}

// ---------------------------------------------------------------------
// 7. Emission persists in shadow.
// ---------------------------------------------------------------------

#[test]
fn emission_persists_when_the_light_is_blocked() {
    let g = gallery();
    let id = redstone_lamp_material_id();
    let mut world = lone(BlockType::RedstoneLampLit, id);
    world.insert(cell(0, 0, 3), block(BlockType::Stone, matte_material_id()));
    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.0, 0.0, 1.0),
        Color::white(),
        1.0,
    ))];
    let lamp = g.library.get(id).unwrap();
    let texture = g.manager.get(lamp.emissive_texture.unwrap()).unwrap();
    let (i, j) = (0..N * N)
        .map(|n| (n % N, n / N))
        .find(|&(i, j)| {
            let t = texture.texel(i, j).unwrap();
            t.r + t.g + t.b > 0.0
        })
        .expect("the lamp mask has lit texels");
    let lx = (i as f32 + 0.5) / 16.0;
    let ly = 1.0 - (j as f32 + 0.5) / 16.0;

    let in_shadow = g.trace(
        &world,
        &lights,
        0.0,
        black(),
        &ray((lx, ly, 2.0), (0.0, 0.0, -1.0)),
        0,
    );

    assert!(
        in_shadow.r > 0.1,
        "a shadowed lamp panel still glows: {in_shadow:?}"
    );
}

// ---------------------------------------------------------------------
// 8. Normal mapping changes shading, not the geometric hit.
// ---------------------------------------------------------------------

#[test]
fn normal_mapping_changes_shading_without_changing_the_hit() {
    let g = gallery();
    let bricks = deepslate_bricks_material_id();
    let mapped = lone(BlockType::DeepslateBricks, bricks);

    let mut plain_material = g.library.get(bricks).unwrap().clone();
    plain_material.normal_texture = None;
    let g = g.with_extra(200, plain_material);
    let plain = lone(BlockType::DeepslateBricks, MaterialId::new(200));

    let lights = [Light::Directional(DirectionalLight::new(
        Vec3::new(0.7, 0.5, 0.5),
        Color::white(),
        1.0,
    ))];
    let mut changed = 0;
    for j in 0..N {
        for i in 0..N {
            let r = ray(
                ((i as f32 + 0.5) / 16.0, 1.0 - (j as f32 + 0.5) / 16.0, 6.0),
                (0.0, 0.0, -1.0),
            );
            assert_eq!(
                nearest_voxel_hit(&mapped, &r, RANGE),
                nearest_voxel_hit(&plain, &r, RANGE).map(|mut h| {
                    h.block = *mapped.get(cell(0, 0, 0)).unwrap();
                    h
                })
            );
            let a = g.trace(&mapped, &lights, 0.1, black(), &r, 0);
            let b = g.trace(&plain, &lights, 0.1, black(), &r, 0);
            if (a.r - b.r).abs() > 0.01 {
                changed += 1;
            }
        }
    }
    assert!(changed > 30, "only {changed} texels changed shading");

    let material = g.library.get(bricks).unwrap();
    let normal = sample_shading_normal(
        material,
        Face::PositiveZ,
        core::math::Vec2::new(0.1, 0.1),
        Face::PositiveZ.normal(),
        &g.manager,
    );
    assert!((normal.length() - 1.0).abs() < 1e-4);
}

// ---------------------------------------------------------------------
// 9. Recursion depth bounds reflection and refraction.
// ---------------------------------------------------------------------

#[test]
fn recursion_depth_limits_reflection_and_refraction() {
    let g = gallery();
    // Two facing mirrors and a stack of glass: unbounded without a cap.
    let mut world = VoxelWorld::new();
    for x in 0..30 {
        world.insert(cell(x, 0, 0), block(BlockType::Stone, mirror_material_id()));
        world.insert(cell(x, 4, 0), block(BlockType::Stone, mirror_material_id()));
    }
    for y in 1..4 {
        world.insert(cell(40, y, 0), block(BlockType::Glass, glass_material_id()));
    }
    for r in [
        ray((0.5, 2.5, 0.5), (1.0, -0.4, 0.0)),
        ray((35.0, 2.5, 0.5), (1.0, -0.2, 0.0)),
    ] {
        assert!(finite(g.trace(&world, &[], 0.5, black(), &r, 0)));
    }

    // At the depth limit only local shading remains: a mirror equals its
    // own ambient color instead of the reflected scene.
    let mirror = g.library.get(mirror_material_id()).unwrap();
    let one = lone(BlockType::Stone, mirror_material_id());
    let r = ray((0.5, 0.5, 5.0), (0.0, 0.0, -1.0));
    let at_limit = g.trace(
        &one,
        &[],
        1.0,
        Color::new(0.0, 0.0, 1.0, 1.0),
        &r,
        MAX_RAY_DEPTH,
    );
    assert!(approx_color(at_limit, mirror.albedo));
    let below = g.trace(&one, &[], 1.0, Color::new(0.0, 0.0, 1.0, 1.0), &r, 0);
    assert!(
        below.b > at_limit.b,
        "one bounce reaches the blue background"
    );
}

// ---------------------------------------------------------------------
// 10. DDA remains the traversal; 11. partial geometry keeps working.
// ---------------------------------------------------------------------

#[test]
fn the_dda_is_still_the_primary_traversal_of_every_ray_kind() {
    let source = std::fs::read_to_string(format!(
        "{}/src/renderer/raytracer.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();

    assert!(source.contains("DdaState::from_ray"));
    assert!(source.contains("pub fn nearest_voxel_hit_where"));
    // Primary, secondary and shadow rays all resolve through the same DDA
    // query, never through a scan of every cell or a legacy cube list.
    assert!(source.contains("nearest_visible_hit(scene, ray, scene.max_distance)"));
    assert!(source.contains("nearest_voxel_hit_where(scene.world"));
    let app =
        std::fs::read_to_string(format!("{}/src/app.rs", env!("CARGO_MANIFEST_DIR"))).unwrap();
    assert!(app.contains("cast_ray_voxel_lit"));
    assert!(app.contains("advanced_materials_world"));
}

#[test]
fn partial_geometry_works_with_advanced_materials() {
    let g = gallery();
    // A mirror-finish stair in front of a red block: the bounce off the
    // stair's step reaches the red block behind the eye.
    let mut world = VoxelWorld::new();
    world.insert(
        cell(0, 0, 0),
        BlockInstance::new(
            BlockType::WoodStairs,
            mirror_material_id(),
            Orientation::South,
        ),
    );
    world.insert(
        cell(0, 0, 8),
        block(BlockType::Stone, backdrop_red_material_id()),
    );
    let r = ray((0.5, 0.75, 6.0), (0.0, 0.0, -1.0));
    let hit = nearest_voxel_hit(&world, &r, RANGE).expect("stair must be hit");
    assert_eq!(hit.block.block_type(), BlockType::WoodStairs);

    let color = g.trace(&world, &[], 0.5, black(), &r, 0);
    assert!(
        color.r > color.g + 0.05,
        "stair reflects the red block: {color:?}"
    );

    // A glass door leaf (thin prism) is see-through partial geometry.
    let mut door = VoxelWorld::new();
    door.insert(
        cell(0, 0, 0),
        BlockInstance::new(BlockType::WoodDoor, glass_material_id(), Orientation::South),
    );
    door.insert(
        cell(0, 0, -3),
        block(BlockType::Stone, backdrop_red_material_id()),
    );
    let seen = g.trace(
        &door,
        &[],
        1.0,
        black(),
        &ray((0.5, 0.5, 6.0), (0.0, 0.0, -1.0)),
        0,
    );
    assert!(
        seen.r > 2.0 * seen.b,
        "red seen through the thin glass leaf: {seen:?}"
    );
}

// ---------------------------------------------------------------------
// 12. The gallery is built on the real MaterialLibrary.
// ---------------------------------------------------------------------

#[test]
fn every_gallery_block_resolves_through_the_material_library() {
    let g = gallery();
    let world = advanced_materials_world();
    let mut count = 0;
    let mut types = std::collections::HashSet::new();

    for x in -1..GALLERY_WIDTH + 1 {
        for y in -1..4 {
            for z in GALLERY_Z_MIN - 1..GALLERY_DEPTH + 1 {
                if let Some(b) = world.get(cell(x, y, z)) {
                    count += 1;
                    assert!(g.library.contains(b.material_id()), "cell ({x},{y},{z})");
                    types.insert(format!("{:?}", b.block_type()));
                }
            }
        }
    }
    assert_eq!(count, world.len());

    // One specimen per required optical behavior is present.
    for t in [
        "Stone",
        "Glass",
        "Water",
        "Leaves",
        "RedstoneLampLit",
        "PortalCoreDarkCrimson",
        "DeepslateBricks",
    ] {
        assert!(types.contains(t), "missing specimen {t}");
    }
    for (x, z, id) in [
        (MATTE_X, FRONT_Z, matte_material_id()),
        (MIRROR_X, FRONT_Z, mirror_material_id()),
        (GLASS_X, FRONT_Z, glass_material_id()),
        (WATER_X, FRONT_Z, water_material_id()),
        (LEAVES_X, BACK_Z, leaves_material_id()),
        (LAMP_X, BACK_Z, redstone_lamp_material_id()),
        (PORTAL_X, BACK_Z, portal_core_material_id()),
        (BRICKS_X, BACK_Z, deepslate_bricks_material_id()),
    ] {
        assert_eq!(world.get(cell(x, 1, z)).unwrap().material_id(), id);
    }
}

// ---------------------------------------------------------------------
// Whole-frame stability of the final gallery render.
// ---------------------------------------------------------------------

#[test]
fn the_gallery_frame_is_finite_stable_and_free_of_missing_materials() {
    let g = gallery();
    let world = advanced_materials_world();
    let lights = gallery_lights();
    let camera = gallery_camera(4.0 / 3.0);
    let (w, h) = (96, 72);
    let mut framebuffer = Framebuffer::new(w, h);

    let (mut background_pixels, mut lit_pixels) = (0, 0);
    for y in 0..h {
        for x in 0..w {
            let r = primary_ray(&camera, x, y, w, h);
            let c = cast_ray_voxel_lit(
                &world,
                &g.library,
                &r,
                camera.position,
                &lights,
                0.1,
                gallery_background(),
                &g.manager,
                GALLERY_MAX_DISTANCE,
            );
            assert!(
                finite(c) && c.g.is_finite() && c.b.is_finite(),
                "pixel ({x},{y}) = {c:?}"
            );
            assert!((0.0..=1.0).contains(&c.g) && (0.0..=1.0).contains(&c.b));
            assert_eq!(c.a, 1.0, "opaque framebuffer pixel ({x},{y})");
            assert_ne!(c, MISSING_MATERIAL_COLOR);
            if c == gallery_background() {
                background_pixels += 1;
            } else {
                lit_pixels += 1;
            }
            framebuffer.set_pixel(x, y, c);
        }
    }

    assert_eq!(framebuffer.pixels().len(), w * h);
    assert!(background_pixels > 100, "background = {background_pixels}");
    assert!(lit_pixels > w * h / 3, "lit = {lit_pixels}");
}

#[test]
fn the_specimens_are_not_hidden_from_the_gallery_camera() {
    let g = gallery();
    let world = advanced_materials_world();
    let camera = gallery_camera(4.0 / 3.0);

    for (x, z) in [
        (MATTE_X, FRONT_Z),
        (MIRROR_X, FRONT_Z),
        (GLASS_X, FRONT_Z),
        (WATER_X, FRONT_Z),
        (LEAVES_X, BACK_Z),
        (LAMP_X, BACK_Z),
        (PORTAL_X, BACK_Z),
        (BRICKS_X, BACK_Z),
    ] {
        // The top-center of the specimen is visible: the first visible hit
        // along the sight line is the specimen itself.
        let target = Vec3::new(x as f32 + 0.5, 1.95, z as f32 + 0.5);
        let r = Ray::new(camera.position, target - camera.position);
        let scene = VoxelScene {
            world: &world,
            materials: &g.library,
            camera_position: camera.position,
            lights: &[],
            ambient_factor: 1.0,
            background: black(),
            texture_manager: &g.manager,
            max_distance: RANGE,
        };
        let hit = nearest_visible_hit(&scene, &r, RANGE).expect("sight line hits something");
        assert!(
            hit.cell.x == x && hit.cell.z == z && hit.cell.y == 1,
            "specimen ({x},{z}) is hidden by cell {:?}",
            hit.cell
        );
    }
}

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
    #[path = "../src/camera/portal_crossing.rs"]
    pub mod portal_crossing;
    #[path = "../src/camera/projection.rs"]
    pub mod projection;
    #[path = "../src/camera/world_free_fly.rs"]
    pub mod world_free_fly;
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
    #[path = "../src/scene/castle.rs"]
    pub mod castle;
    #[path = "../src/scene/catalog.rs"]
    pub mod catalog;
    #[path = "../src/scene/cutaway.rs"]
    pub mod cutaway;
    #[path = "../src/scene/descent.rs"]
    pub mod descent;
    #[path = "../src/scene/environment.rs"]
    pub mod environment;
    #[path = "../src/scene/expansion.rs"]
    pub mod expansion;
    #[path = "../src/scene/fortress.rs"]
    pub mod fortress;
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
    #[path = "../src/scene/overworld.rs"]
    pub mod overworld;
    #[path = "../src/scene/overworld_blocks.rs"]
    pub mod overworld_blocks;
    #[path = "../src/scene/portal.rs"]
    pub mod portal;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/rhombus.rs"]
    pub mod rhombus;
    #[path = "../src/scene/scene.rs"]
    pub mod scene;
    #[path = "../src/scene/terrain/mod.rs"]
    pub mod terrain;
    #[path = "../src/scene/texture_manager.rs"]
    pub mod texture_manager;
    #[path = "../src/scene/voxel_world.rs"]
    pub mod voxel_world;
    #[path = "../src/scene/world.rs"]
    pub mod world;
}

#[path = "."]
mod renderer {
    #[path = "../src/renderer/emission.rs"]
    pub mod emission;
    #[path = "../src/renderer/framebuffer.rs"]
    pub mod framebuffer;
    #[path = "../src/renderer/normal_mapping.rs"]
    pub mod normal_mapping;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/shading.rs"]
    pub mod shading;
    #[path = "../src/renderer/shadows.rs"]
    pub mod shadows;
    #[path = "../src/renderer/skybox.rs"]
    pub mod skybox;
    #[path = "../src/renderer/texture_sampling.rs"]
    pub mod texture_sampling;
    #[path = "../src/renderer/voxel_traversal.rs"]
    pub mod voxel_traversal;
}

use camera::projection::primary_ray;
use camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm};
use core::color::Color;
use core::material::{AlphaMode, Material, MaterialId};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use renderer::raytracer::{
    INTERACTIVE_REFLECTION_THRESHOLD, RenderQuality, VoxelScene, cast_ray_voxel, cast_ray_voxel_at,
    nearest_visible_hit, trace_ray, trace_ray_at,
};
use renderer::skybox::Background;
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::catalog::{CatalogScene, CatalogTextures, catalog_materials};
use scene::light::{DirectionalLight, Light};
use scene::material_gallery::{
    GALLERY_MAX_DISTANCE, gallery_background, gallery_camera, gallery_lights, glass_material_id,
    portal_core_material_id, water_material_id,
};
use scene::material_library::MaterialLibrary;
use scene::orientation::Orientation;
use scene::overworld_blocks::amethyst_cluster_material_id;
use scene::texture_manager::TextureManager;
use scene::voxel_world::VoxelWorld;
use scene::world::{
    WORLD_AMBIENT_FACTOR, WORLD_MAX_DISTANCE, WorldScene, WorldTextures, world_free_fly_camera,
    world_lights, world_materials,
};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

struct Rig {
    manager: TextureManager,
    scene: WorldScene,
    materials: MaterialLibrary,
    lights: Vec<Light>,
}

impl Rig {
    fn new() -> Self {
        let mut manager = TextureManager::new();
        let textures = WorldTextures::load(
            &mut manager,
            "assets/textures/overworld",
            "assets/textures/overworld/grass",
            "assets/textures/portal",
        )
        .unwrap();
        Self {
            manager,
            scene: WorldScene::new(),
            materials: world_materials(&textures),
            lights: world_lights(),
        }
    }

    fn scene(&self, state: &WorldFreeFlyCameraState) -> VoxelScene<'_> {
        VoxelScene {
            world: self.scene.world(),
            materials: &self.materials,
            camera_position: state.position,
            lights: &self.lights,
            ambient_factor: WORLD_AMBIENT_FACTOR * state.environment().ambient_scale,
            background: state.background(),
            texture_manager: &self.manager,
            max_distance: WORLD_MAX_DISTANCE,
        }
    }
}

fn render(
    scene: &VoxelScene,
    camera: &camera::camera::Camera,
    w: usize,
    h: usize,
    q: RenderQuality,
) -> Vec<Color> {
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            out.push(cast_ray_voxel_at(
                scene,
                &primary_ray(camera, x, y, w, h),
                q,
            ));
        }
    }
    out
}

/// A two-cell probe: the `probe` material in front of a plain grey wall
/// that the mirror ray would see. Lit by one sun from behind the camera.
struct Probe {
    world: VoxelWorld,
    materials: MaterialLibrary,
    lights: Vec<Light>,
    manager: TextureManager,
}

impl Probe {
    fn new(probe: Material) -> Self {
        let mut materials = MaterialLibrary::new();
        materials.insert(MaterialId::new(1), probe);
        materials.insert(
            MaterialId::new(2),
            Material::new(Color::new(0.9, 0.2, 0.2, 1.0), 0.0, 1.0),
        );
        let mut world = VoxelWorld::new();
        world.insert(
            IVec3::new(0, 0, 0),
            BlockInstance::new(BlockType::Stone, MaterialId::new(1), Orientation::Up),
        );
        // A red wall the reflection off the probe's +Z face looks at.
        world.insert(
            IVec3::new(0, 3, 0),
            BlockInstance::new(BlockType::Stone, MaterialId::new(2), Orientation::Up),
        );
        Self {
            world,
            materials,
            lights: vec![Light::Directional(DirectionalLight::new(
                v(0.2, 0.3, 1.0),
                Color::new(1.0, 1.0, 1.0, 1.0),
                1.0,
            ))],
            manager: TextureManager::new(),
        }
    }

    fn color(&self, q: RenderQuality) -> Color {
        let scene = VoxelScene {
            world: &self.world,
            materials: &self.materials,
            camera_position: v(0.5, -2.0, 3.0),
            lights: &self.lights,
            ambient_factor: 0.3,
            background: Background::Solid(Color::new(0.1, 0.1, 0.1, 1.0)),
            texture_manager: &self.manager,
            max_distance: 96.0,
        };
        // Hits the +Z face at 45 degrees, so the mirror ray goes up to the wall.
        let ray = Ray::new(v(0.5, -2.0, 3.0), v(0.0, 2.5, -2.5));
        assert!(nearest_visible_hit(&scene, &ray, 96.0).is_some());
        cast_ray_voxel_at(&scene, &ray, q)
    }
}

/// The same optics without the face textures a probe cannot sample.
fn untextured(material: &Material) -> Material {
    Material {
        face_textures: None,
        normal_texture: None,
        emissive_texture: None,
        ..material.clone()
    }
}

fn opaque(reflectivity: f32) -> Material {
    Material::new(Color::new(0.5, 0.5, 0.5, 1.0), 0.1, 10.0).with_reflectivity(reflectivity)
}

#[test]
fn full_quality_reproduces_the_certified_renderer() {
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    for y in 0..30 {
        for x in 0..40 {
            let ray = primary_ray(&camera, x, y, 40, 30);
            let reference = trace_ray(&scene, &ray, 0);
            assert_eq!(
                trace_ray_at(&scene, &ray, 0, RenderQuality::Full),
                reference
            );
            assert_eq!(
                cast_ray_voxel_at(&scene, &ray, RenderQuality::Full),
                reference
            );
            assert_eq!(cast_ray_voxel(&scene, &ray), reference);
        }
    }
    assert_eq!(RenderQuality::default(), RenderQuality::Full);
}

#[test]
fn interactive_skips_reflections_below_the_threshold() {
    assert_eq!(INTERACTIVE_REFLECTION_THRESHOLD, 0.05);
    for weak in [0.01, 0.02, 0.03, 0.049] {
        let probe = Probe::new(opaque(weak));
        let full = probe.color(RenderQuality::Full);
        let interactive = probe.color(RenderQuality::Interactive);
        let none = Probe::new(opaque(0.0)).color(RenderQuality::Full);
        assert_ne!(full, none, "reflectivity {weak} must reflect at Full");
        assert_eq!(interactive, none, "reflectivity {weak} must be skipped");
        assert!(!RenderQuality::Interactive.traces_reflection(&opaque(weak)));
        assert!(RenderQuality::Full.traces_reflection(&opaque(weak)));
    }
    assert!(!RenderQuality::Full.traces_reflection(&opaque(0.0)));
}

#[test]
fn reflectivity_at_the_threshold_is_preserved() {
    for strong in [0.05, 0.051, 0.10, 0.65] {
        let probe = Probe::new(opaque(strong));
        assert_eq!(
            probe.color(RenderQuality::Interactive),
            probe.color(RenderQuality::Full),
            "reflectivity {strong}"
        );
        assert!(RenderQuality::Interactive.traces_reflection(&opaque(strong)));
    }
}

#[test]
fn water_keeps_its_reflection_in_interactive() {
    let rig = Rig::new();
    let water = rig.materials.get(water_material_id()).unwrap();
    assert!(water.reflectivity >= INTERACTIVE_REFLECTION_THRESHOLD);
    assert!(RenderQuality::Interactive.traces_reflection(water));
    let probe = Probe::new(untextured(water));
    assert_eq!(
        probe.color(RenderQuality::Interactive),
        probe.color(RenderQuality::Full)
    );
}

#[test]
fn glass_keeps_reflection_and_refraction_in_interactive() {
    let rig = Rig::new();
    let glass = rig.materials.get(glass_material_id()).unwrap();
    assert!(glass.transparency > 0.0 && glass.reflectivity > 0.0);
    assert!(RenderQuality::Interactive.traces_reflection(glass));
    let probe = Probe::new(untextured(glass));
    assert_eq!(
        probe.color(RenderQuality::Interactive),
        probe.color(RenderQuality::Full)
    );
    // The cluster (0.20, transparent) as well.
    let amethyst = rig.materials.get(amethyst_cluster_material_id()).unwrap();
    assert!(RenderQuality::Interactive.traces_reflection(amethyst));
}

#[test]
fn the_portal_core_keeps_its_full_behavior_in_interactive() {
    let rig = Rig::new();
    let core = rig.materials.get(portal_core_material_id()).unwrap();
    // Weakly reflective but a transmissive medium: its reflection is part
    // of its optics, so the material property (not a block type) keeps it.
    assert!(core.reflectivity < INTERACTIVE_REFLECTION_THRESHOLD);
    assert!(core.transparency > 0.0 && core.alpha_mode != AlphaMode::Cutout);
    assert!(RenderQuality::Interactive.traces_reflection(core));
    // In front of a non-reflective wall the membrane renders identically
    // (in the real portal only the weakly reflective rock *behind* it
    // differs, as for any other view through a transparent medium).
    let probe = Probe::new(untextured(core));
    assert_eq!(
        probe.color(RenderQuality::Interactive),
        probe.color(RenderQuality::Full)
    );
    let state = WorldFreeFlyCameraState::looking_at(v(16.5, -7.0, 17.5), v(16.5, -7.0, 14.5));
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    let ray = primary_ray(&camera, 20, 15, 40, 30);
    let hit = nearest_visible_hit(&scene, &ray, WORLD_MAX_DISTANCE).unwrap();
    assert_eq!(hit.block.material_id(), portal_core_material_id());
}

#[test]
fn both_qualities_share_primary_hits_and_shadows() {
    // A non-reflective probe shades identically: same hit, same shadow.
    let probe = Probe::new(opaque(0.0));
    assert_eq!(
        probe.color(RenderQuality::Interactive),
        probe.color(RenderQuality::Full)
    );
    let rig = Rig::new();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    // Sky pixels and pixels on non-reflective blocks are identical too.
    let mut same = 0;
    for y in 0..30 {
        for x in 0..40 {
            let ray = primary_ray(&camera, x, y, 40, 30);
            let reflective = nearest_visible_hit(&scene, &ray, WORLD_MAX_DISTANCE)
                .and_then(|h| rig.materials.get(h.block.material_id()))
                .is_some_and(|m| m.reflectivity > 0.0);
            if !reflective {
                assert_eq!(
                    cast_ray_voxel_at(&scene, &ray, RenderQuality::Interactive),
                    cast_ray_voxel_at(&scene, &ray, RenderQuality::Full)
                );
                same += 1;
            }
        }
    }
    assert!(same > 100);
}

#[test]
fn the_catalog_at_full_quality_is_unchanged() {
    let mut manager = TextureManager::new();
    let textures = CatalogTextures::load(
        &mut manager,
        "assets/textures/overworld",
        "assets/textures/portal",
        "assets/textures/overworld/grass",
        "assets/textures/diagnostic/partial",
    )
    .unwrap();
    let catalog = CatalogScene::new();
    let materials = catalog_materials(&textures);
    let lights = gallery_lights();
    let camera = gallery_camera(4.0 / 3.0);
    let scene = VoxelScene {
        world: catalog.world(),
        materials: &materials,
        camera_position: camera.position,
        lights: &lights,
        ambient_factor: renderer::shading::DEFAULT_AMBIENT_FACTOR,
        background: Background::Solid(gallery_background()),
        texture_manager: &manager,
        max_distance: GALLERY_MAX_DISTANCE,
    };
    let mut reference = Vec::new();
    for y in 0..48 {
        for x in 0..64 {
            reference.push(trace_ray(&scene, &primary_ray(&camera, x, y, 64, 48), 0));
        }
    }
    assert_eq!(
        render(&scene, &camera, 64, 48, RenderQuality::Full),
        reference
    );
}

#[test]
fn rendering_at_either_quality_mutates_no_material() {
    let rig = Rig::new();
    let before: Vec<(MaterialId, Material)> = [
        water_material_id(),
        glass_material_id(),
        portal_core_material_id(),
        amethyst_cluster_material_id(),
        scene::scene::grass_material_id(),
    ]
    .into_iter()
    .map(|id| (id, rig.materials.get(id).unwrap().clone()))
    .collect();
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    render(&scene, &camera, 40, 30, RenderQuality::Interactive);
    render(&scene, &camera, 40, 30, RenderQuality::Full);
    for (id, material) in &before {
        assert_eq!(rig.materials.get(*id).unwrap(), material);
    }
    assert_eq!(rig.materials.len(), before.len().max(rig.materials.len()));
}

#[test]
fn the_quality_enum_touches_no_scene_state() {
    let rig = Rig::new();
    let (len, bounds) = (rig.scene.world().len(), rig.scene.world().bounds());
    let state = world_free_fly_camera();
    let camera = state.camera(4.0 / 3.0);
    let scene = rig.scene(&state);
    render(&scene, &camera, 20, 15, RenderQuality::Interactive);
    assert_eq!(
        (rig.scene.world().len(), rig.scene.world().bounds()),
        (len, bounds)
    );
    assert_eq!(len, WorldScene::new().world().len()); // deterministic (Gate 14 changed the total)
    assert!(len > 6000);
    assert_eq!(rig.lights.len(), 5);
    assert_eq!(RenderQuality::Interactive.label(), "Interactive");
    assert_eq!(RenderQuality::Full.label(), "Full");
    // `traces_reflection` is a pure function of the material.
    let m = opaque(0.02);
    assert_eq!(
        RenderQuality::Interactive.traces_reflection(&m),
        RenderQuality::Interactive.traces_reflection(&m.clone())
    );
}

/// Quantifies the Interactive approximation at reduced resolution for the
/// benchmark poses (printed with `--nocapture`, bounded by the assertion).
#[test]
fn interactive_image_difference_is_small_and_quantified() {
    let rig = Rig::new();
    let mut red_black = |p: Vec3, t: Vec3| {
        let mut s = WorldFreeFlyCameraState::looking_at(p, t);
        s.realm = WorldRealm::RedBlack;
        s.local_up = WorldRealm::RedBlack.up();
        s.pitch = -s.pitch;
        s
    };
    let poses = [
        ("A", world_free_fly_camera()),
        (
            "B",
            WorldFreeFlyCameraState::looking_at(v(15.0, 11.0, 21.0), v(8.0, 6.0, 10.0)),
        ),
        (
            "C",
            WorldFreeFlyCameraState::looking_at(v(16.5, -6.0, 20.5), v(16.5, -7.0, 14.5)),
        ),
        (
            "E",
            WorldFreeFlyCameraState::looking_at(v(7.5, 10.0, 25.0), v(7.5, 5.0, 18.0)),
        ),
        ("F", red_black(v(16.5, -7.0, 13.8), v(16.5, -7.0, 10.0))),
    ];
    for (name, state) in poses {
        let camera = state.camera(4.0 / 3.0);
        let scene = rig.scene(&state);
        let full = render(&scene, &camera, 80, 60, RenderQuality::Full);
        let interactive = render(&scene, &camera, 80, 60, RenderQuality::Interactive);
        let (mut max, mut sum, mut changed) = (0i32, 0i64, 0usize);
        for (a, b) in full.iter().zip(&interactive) {
            let (a, b) = (a.to_rgba8(), b.to_rgba8());
            let d = (0..3)
                .map(|k| (a[k] as i32 - b[k] as i32).abs())
                .max()
                .unwrap();
            max = max.max(d);
            sum += d as i64;
            changed += (d > 0) as usize;
        }
        println!(
            "INTERACTIVE_DIFF {name} 80x60 max_channel_diff_8bit={max} mean_diff={:.4} pixels_changed={:.2}%",
            sum as f64 / full.len() as f64,
            100.0 * changed as f64 / full.len() as f64
        );
        assert!(max <= 12, "{name}: max diff {max}/255");
    }
}

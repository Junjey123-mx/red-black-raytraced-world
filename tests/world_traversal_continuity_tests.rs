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
    #[path = "../src/camera/world_collision.rs"]
    pub mod world_collision;
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
    #[path = "../src/renderer/parallel.rs"]
    pub mod parallel;
    #[path = "../src/renderer/perf.rs"]
    pub mod perf;
    #[path = "../src/renderer/preview.rs"]
    pub mod preview;
    #[path = "../src/renderer/raytracer.rs"]
    pub mod raytracer;
    #[path = "../src/renderer/refinement.rs"]
    pub mod refinement;
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

use camera::world_collision::{
    CameraCollisionConfig, CollisionState, is_camera_passable, is_camera_solid, is_position_clear,
    resolve_camera_motion,
};
use core::math::{IVec3, Vec3};
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::world::WorldScene;

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(
        scene.world(),
        p,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(
        scene.world(),
        from,
        delta,
        &CameraCollisionConfig::default(),
        &is_camera_solid,
    )
}

/// Eye of the inverted viewer under a `Down` tread cell.
fn eye_under(t: IVec3) -> Vec3 {
    v(t.x as f32 + 0.5, t.y as f32 - 1.5, t.z as f32 + 0.5)
}

/// Eye of the Overworld viewer standing on a tread cell (on its low half).
fn eye_over(t: IVec3) -> Vec3 {
    v(t.x as f32 + 0.5, t.y as f32 + 2.0, t.z as f32 + 0.5)
}

/// The canonical route from the Overworld path end to the Red-Black
/// landing, as a list of eye positions the camera must reach in order,
/// with collision on and no noclip.
fn canonical_route(scene: &WorldScene) -> Vec<Vec3> {
    let mut points = Vec::new();
    // Overworld: end of the cobblestone path, then every upper tread.
    let end = *scene.path().cells.last().unwrap();
    points.push(v(
        end.x as f32 + 0.5,
        end.y as f32 + 2.0,
        end.z as f32 + 0.5,
    ));
    for (tread, _) in &scene.upper_descent().treads {
        points.push(eye_over(*tread));
    }
    // The brick approach in front of the membrane, the membrane, the exit.
    points.push(v(16.5, -7.5, 16.5));
    points.push(v(16.5, -7.5, 14.5));
    let layout = scene.inverted_route().layout;
    let exit = layout.portal_exit;
    points.push(v(
        exit.x as f32 + 0.5,
        exit.y as f32 + 0.5,
        exit.z as f32 + 0.5,
    ));
    // The inverted flight, the rim step, the landing, the open underside.
    for (tread, _) in &scene.inverted_route().treads {
        points.push(eye_under(*tread));
    }
    let p = IVec3::new(layout.x, layout.platform_y, layout.platform_z_max);
    points.push(eye_under(p));
    let p = IVec3::new(layout.x, layout.platform_y, layout.platform_z_min);
    points.push(eye_under(p));
    points.push(eye_under(IVec3::new(
        layout.x,
        layout.platform_y,
        layout.platform_z_min - 2,
    )));
    points
}

/// Moves from `from` toward `target` with collision on: straight, or, when
/// the straight line cuts a corner, the way a walker turns it (horizontal
/// then vertical, or vertical then horizontal). Returns where it ended.
fn step(scene: &WorldScene, from: Vec3, target: Vec3) -> Vec3 {
    let direct = resolve(scene, from, target - from);
    if (direct - target).length() < 1e-3 {
        return direct;
    }
    let corner_a = v(target.x, from.y, target.z);
    let corner_b = v(from.x, target.y, from.z);
    for corner in [corner_a, corner_b] {
        let mid = resolve(scene, from, corner - from);
        if (mid - corner).length() < 1e-3 {
            let end = resolve(scene, mid, target - mid);
            if (end - target).length() < 1e-3 {
                return end;
            }
        }
    }
    direct
}

fn walk(scene: &WorldScene, points: &[Vec3]) {
    let mut position = points[0];
    assert!(clear(scene, position), "start {position:?} is not clear");
    for (i, target) in points.iter().enumerate().skip(1) {
        let reached = step(scene, position, *target);
        assert!(
            (reached - *target).length() < 1e-3,
            "segment {i}: stuck at {reached:?} short of {target:?}"
        );
        position = reached;
    }
}

#[test]
fn the_portal_to_surface_route_is_contiguous() {
    let scene = WorldScene::new();
    let route = scene.inverted_route();
    let layout = route.layout;
    // Treads: in-mass flight then the rim step, each one north and one
    // lower than the previous.
    let treads: Vec<IVec3> = route.treads.iter().map(|(c, _)| *c).collect();
    assert_eq!(treads.len(), layout.step_count as usize + 1);
    for pair in treads.windows(2) {
        assert_eq!(
            (pair[1].x, pair[1].y, pair[1].z),
            (pair[0].x, pair[0].y - 1, pair[0].z - 1)
        );
    }
    assert_eq!(*treads.last().unwrap(), layout.rim_step);
    // The rim step hangs from its support; the platform's face is one
    // cell below the rim step (one more step), two cells further north.
    assert_eq!(
        layout.rim_support,
        IVec3::new(layout.rim_step.x, layout.rim_step.y + 1, layout.rim_step.z)
    );
    assert!(scene.world().contains(layout.rim_support));
    assert_eq!(layout.platform_y, layout.rim_step.y - 1);
    assert_eq!(layout.platform_z_max, layout.rim_step.z - 1);
    for cell in layout.platform_cells() {
        assert_eq!(
            scene.world().get(cell).map(|b| b.block_type()),
            Some(BlockType::DeepslateBricks)
        );
        assert_eq!(
            scene.world().get(cell).unwrap().orientation(),
            Orientation::Down
        );
    }
    println!(
        "GATE14 route: first={:?} last_in_mass={:?} rim_step={:?} platform_y={} z={}..={} supports={} dug={}",
        treads[0],
        layout.landing,
        layout.rim_step,
        layout.platform_y,
        layout.platform_z_min,
        layout.platform_z_max,
        route.supports.len(),
        route.dug.len()
    );
}

#[test]
fn no_solid_block_sits_on_the_centerline() {
    let scene = WorldScene::new();
    for (i, point) in canonical_route(&scene).iter().enumerate() {
        assert!(
            clear(&scene, *point),
            "point {i} at {point:?} is inside a solid"
        );
    }
}

#[test]
fn the_collision_volume_fits_every_segment() {
    let scene = WorldScene::new();
    walk(&scene, &canonical_route(&scene));
}

#[test]
fn the_landing_is_connected_to_the_surface() {
    let scene = WorldScene::new();
    let layout = scene.inverted_route().layout;
    let platform = layout.platform_cells();
    // The platform touches the mass face to face at its own level, so its
    // -Y face continues the rim surface (a step of zero).
    let touches_mass = platform.iter().any(|c| {
        [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| {
            let n = IVec3::new(c.x + dx, c.y, c.z + dz);
            !platform.contains(&n) && scene.world().contains(n)
        })
    });
    assert!(touches_mass, "the landing hangs free of the rim");
    // Surface cells of the underside at the platform's level lie beside it.
    let flush = scene
        .lower_surface()
        .cells
        .iter()
        .filter(|c| {
            c.y == layout.platform_y
                && (c.z - layout.platform_z_max).abs() <= 1
                && (c.x - layout.x).abs() <= 2
        })
        .count();
    assert!(flush >= 1, "no surface cell flush with the landing");
    // The support and platform are approved dark blocks only.
    for cell in &scene.inverted_route().supports {
        let t = scene.world().get(*cell).unwrap().block_type();
        assert!(
            matches!(t, BlockType::Deepslate | BlockType::DeepslateBricks),
            "{t:?}"
        );
    }
}

#[test]
fn the_canonical_route_needs_no_noclip() {
    let scene = WorldScene::new();
    let state = CollisionState::new();
    assert!(state.collision_enabled() && !state.is_noclip());
    // Walked entirely with collision resolution (never bypassed).
    walk(&scene, &canonical_route(&scene));
}

#[test]
fn the_reverse_route_works() {
    let scene = WorldScene::new();
    let mut points = canonical_route(&scene);
    points.reverse();
    walk(&scene, &points);
}

#[test]
fn the_upper_route_is_connected_to_the_portal() {
    let scene = WorldScene::new();
    let route = canonical_route(&scene);
    // From the path end down the twelve upper treads to the approach.
    let upper_end = 1 + scene.upper_descent().treads.len() + 1;
    walk(&scene, &route[..=upper_end]);
    assert_eq!(scene.upper_descent().treads.len(), 12);
    let last = scene.upper_descent().last().unwrap();
    assert_eq!(last, IVec3::new(15, -8, 18));
}

#[test]
fn the_wood_door_is_passable() {
    let scene = WorldScene::new();
    assert!(is_camera_passable(BlockType::WoodDoor));
    let outside = v(8.5, 7.5, 13.5);
    let inside = resolve(&scene, outside, v(0.0, 0.0, -3.0));
    assert!(inside.z < 12.0, "{inside:?}");
    // The interior is observable: it is open air the camera can turn in.
    for target in [v(6.5, 7.5, 9.5), v(10.5, 8.5, 9.5), v(8.5, 8.5, 11.5)] {
        assert!(clear(&scene, target));
        let reached = resolve(&scene, inside, target - inside);
        assert!((reached - target).length() < 1e-3, "{reached:?}");
    }
    let out = resolve(&scene, inside, v(0.0, 0.0, 3.0));
    assert!(out.z > 13.0, "{out:?}");
}

#[test]
fn the_house_walls_are_solid() {
    let scene = WorldScene::new();
    for cell in scene
        .house()
        .walls
        .iter()
        .chain(scene.house().corners.iter())
        .chain(scene.house().floor.iter())
    {
        assert!(
            is_camera_solid(scene.world().get(*cell).unwrap()),
            "{cell:?}"
        );
    }
    // From inside, no way out except the door: north, east, west and up.
    let inside = v(8.5, 7.5, 10.5);
    assert!(resolve(&scene, inside, v(0.0, 0.0, -3.0)).z >= 9.0 + 0.25 - 1e-3);
    assert!(resolve(&scene, inside, v(3.0, 0.0, 0.0)).x <= 11.0 - 0.25 + 1e-3);
    assert!(resolve(&scene, inside, v(-3.0, 0.0, 0.0)).x >= 6.0 + 0.25 - 1e-3);
    assert!(resolve(&scene, inside, v(0.0, 4.0, 0.0)).y <= 10.0 - 0.25 + 1e-3);
    // The glass windows too.
    for cell in &scene.house_exterior().glass {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
}

#[test]
fn the_portal_frame_is_solid() {
    let scene = WorldScene::new();
    for cell in &scene.portal().frame {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    assert!(!is_camera_passable(BlockType::PortalFrameRedObsidian));
    let to = resolve(&scene, v(16.5, -4.5, 16.5), v(0.0, 0.0, -3.0));
    assert!(to.z >= 15.25 - 1e-3, "{to:?}");
}

#[test]
fn the_portal_core_is_passable() {
    let scene = WorldScene::new();
    for cell in &scene.portal().core {
        assert!(!is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    let to = resolve(&scene, v(16.5, -7.5, 16.5), v(0.0, 0.0, -3.0));
    assert!(to.z < 14.0, "{to:?}");
}

#[test]
fn no_focal_family_geometry_was_destroyed() {
    let scene = WorldScene::new();
    for family in scene.families() {
        for cell in family
            .symbols
            .iter()
            .chain(family.accents.iter())
            .chain(family.structures.iter())
        {
            assert!(
                scene.world().contains(*cell),
                "{cell:?} of {:?} is gone",
                family.family
            );
        }
    }
    let route = scene.inverted_route();
    for cell in &route.dug {
        assert!(
            !scene.families().iter().any(|f| f.cells().contains(cell)),
            "{cell:?} was family art"
        );
        assert!(!scene.portal().core.contains(cell) && !scene.portal().frame.contains(cell));
    }
    // The amethyst and crying obsidian focal blocks are all still there.
    let mut amethyst = 0;
    let mut crying = 0;
    for (_, b) in scene.world().iter() {
        match b.block_type() {
            BlockType::AmethystCluster | BlockType::BuddingAmethyst => amethyst += 1,
            BlockType::CryingObsidianCrimson
            | BlockType::CryingObsidianOrange
            | BlockType::CryingObsidianViolet => crying += 1,
            _ => {}
        }
    }
    assert!(amethyst > 0 && crying > 0);
}

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
    #[path = "../src/scene/red_black_identity.rs"]
    pub mod red_black_identity;
    #[path = "../src/scene/red_black_maze.rs"]
    pub mod red_black_maze;
    #[path = "../src/scene/red_black_timber.rs"]
    pub mod red_black_timber;
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

use camera::portal_crossing::{PortalCrossingDetector, PortalCrossingEvent, PortalVolume};
use camera::world_collision::{
    CAMERA_COLLISION_RADIUS, CameraCollisionConfig, CollisionState, NoclipToggle,
    is_camera_passable, is_camera_solid, is_position_clear, resolve_camera_motion,
};
use camera::world_free_fly::{
    FreeFlyInput, WorldFreeFlyCameraState, WorldRealm, apply_free_fly_input,
};
use core::math::{IVec3, Vec3};
use core::ray::Ray;
use scene::block::BlockInstance;
use scene::block_type::BlockType;
use scene::orientation::Orientation;
use scene::world::{WorldScene, world_free_fly_camera};

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn cfg() -> CameraCollisionConfig {
    CameraCollisionConfig::default()
}

fn clear(scene: &WorldScene, p: Vec3) -> bool {
    is_position_clear(scene.world(), p, &cfg(), &is_camera_solid)
}

fn resolve(scene: &WorldScene, from: Vec3, delta: Vec3) -> Vec3 {
    resolve_camera_motion(scene.world(), from, delta, &cfg(), &is_camera_solid)
}

/// A cell of the given type somewhere in the world, with a camera position
/// at its center.
fn center_of(scene: &WorldScene, block_type: BlockType) -> (IVec3, Vec3) {
    let (cell, _) = scene
        .world()
        .iter()
        .find(|(_, b)| b.block_type() == block_type)
        .unwrap_or_else(|| panic!("no {block_type:?} in the world"));
    (
        *cell,
        v(
            cell.x as f32 + 0.5,
            cell.y as f32 + 0.5,
            cell.z as f32 + 0.5,
        ),
    )
}

fn assert_solid(scene: &WorldScene, block_type: BlockType) {
    let (cell, center) = center_of(scene, block_type);
    let block = scene.world().get(cell).unwrap();
    assert!(is_camera_solid(block), "{block_type:?} should be solid");
    assert!(!is_camera_passable(block_type));
    assert!(
        !clear(scene, center),
        "{block_type:?} at {cell:?} lets the camera in"
    );
}

#[test]
fn stone_is_solid() {
    assert_solid(&WorldScene::new(), BlockType::Stone);
}

#[test]
fn dirt_is_solid() {
    assert_solid(&WorldScene::new(), BlockType::Dirt);
}

#[test]
fn glass_is_solid() {
    let scene = WorldScene::new();
    assert_solid(&scene, BlockType::Glass);
    // The house windows: from inside, the camera cannot leave through them.
    let (cell, _) = center_of(&scene, BlockType::Glass);
    let block = BlockInstance::new(
        BlockType::Glass,
        scene.world().get(cell).unwrap().material_id(),
        Orientation::Up,
    );
    assert!(is_camera_solid(&block));
}

#[test]
fn leaves_are_solid() {
    assert_solid(&WorldScene::new(), BlockType::Leaves);
}

#[test]
fn the_portal_frame_is_solid() {
    let scene = WorldScene::new();
    assert_solid(&scene, BlockType::PortalFrameRedObsidian);
    // Flying at the frame's top row from the open cutaway air: stopped
    // one radius in front of it, while the membrane below would let the
    // camera through.
    let from = v(16.5, -4.5, 16.5);
    assert!(clear(&scene, from));
    let to = resolve(&scene, from, v(0.0, 0.0, -3.0));
    assert!(to.z >= 15.0 + CAMERA_COLLISION_RADIUS - 1e-3, "{to:?}");
    for cell in &scene.portal().frame {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
}

#[test]
fn the_portal_core_is_passable_but_still_visible() {
    let scene = WorldScene::new();
    assert!(is_camera_passable(BlockType::PortalCoreDarkCrimson));
    for cell in &scene.portal().core {
        let block = scene.world().get(*cell).unwrap();
        assert!(!is_camera_solid(block));
        assert_eq!(block.block_type(), BlockType::PortalCoreDarkCrimson);
        assert_eq!(
            block.material_id(),
            scene::material_gallery::portal_core_material_id()
        );
    }
    // Inside the membrane slab: clear for the camera.
    assert!(clear(&scene, v(16.5, -7.5, 14.5)));
    // Through the membrane from the approach into the exit clearance
    // behind it: the slab is passed and the camera ends in clear air on
    // the far side.
    let from = v(16.5, -7.5, 16.5);
    let to = resolve(&scene, from, v(0.0, 0.0, -2.5));
    assert!(to.z < 14.4375, "did not pass the membrane: {to:?}");
    assert!(to.z <= 14.0 + 1e-3, "{to:?}");
    assert!(clear(&scene, to));
    // Still raytraced: a ray from the approach hits the membrane first.
    let ray = Ray::new(from, v(0.0, 0.0, -1.0));
    let hit = renderer::raytracer::nearest_voxel_hit(scene.world(), &ray, 96.0).unwrap();
    assert_eq!(hit.block.block_type(), BlockType::PortalCoreDarkCrimson);
}

#[test]
fn the_wood_door_is_passable_but_still_visible() {
    let scene = WorldScene::new();
    assert!(is_camera_passable(BlockType::WoodDoor));
    let door = &scene.house_exterior().door;
    assert_eq!(door.len(), 2);
    for cell in door {
        let block = scene.world().get(*cell).unwrap();
        assert_eq!(block.block_type(), BlockType::WoodDoor);
        assert!(!is_camera_solid(block));
        assert_eq!(block.orientation(), Orientation::South);
    }
    // Materials untouched: bottom and top leaves keep their catalog ids.
    assert_eq!(
        scene.world().get(door[0]).unwrap().material_id(),
        scene::overworld_blocks::wood_door_bottom_material_id()
    );
    assert_eq!(
        scene.world().get(door[1]).unwrap().material_id(),
        scene::overworld_blocks::wood_door_top_material_id()
    );
    // Still raytraced from the porch.
    let ray = Ray::new(v(8.5, 7.5, 14.5), v(0.0, 0.0, -1.0));
    let hit = renderer::raytracer::nearest_voxel_hit(scene.world(), &ray, 96.0).unwrap();
    assert_eq!(hit.block.block_type(), BlockType::WoodDoor);
}

#[test]
fn the_doorway_can_be_walked_through_into_the_house() {
    let scene = WorldScene::new();
    let outside = v(8.5, 7.5, 13.5);
    assert!(clear(&scene, outside));
    let inside = resolve(&scene, outside, v(0.0, 0.0, -3.0));
    assert!(inside.z < 12.0, "stuck at the door: {inside:?}");
    assert!(clear(&scene, inside));
    // Now inside the house: floor below, roof above, walls around.
    assert!(inside.x > 5.5 && inside.x < 11.5 && inside.z > 8.5 && inside.z < 12.5);
    let floor_probe = resolve(&scene, inside, v(0.0, -3.0, 0.0));
    assert!(
        floor_probe.y >= 7.0 + CAMERA_COLLISION_RADIUS - 1e-3,
        "{floor_probe:?}"
    );
    let roof_probe = resolve(&scene, inside, v(0.0, 4.0, 0.0));
    assert!(
        roof_probe.y <= 10.0 - CAMERA_COLLISION_RADIUS + 1e-3,
        "{roof_probe:?}"
    );
    assert!(
        roof_probe.y > 9.0,
        "the interior is three cells tall: {roof_probe:?}"
    );
    // And back out through the door.
    let out = resolve(&scene, inside, v(0.0, 0.0, 3.0));
    assert!(out.z > 13.0, "could not leave: {out:?}");
}

#[test]
fn the_wall_beside_the_door_stays_solid() {
    let scene = WorldScene::new();
    for x in [6.5, 7.5, 9.5, 10.5] {
        let from = v(x, 7.5, 13.5);
        assert!(clear(&scene, from), "{from:?}");
        let to = resolve(&scene, from, v(0.0, 0.0, -3.0));
        assert!(
            to.z >= 13.0 + CAMERA_COLLISION_RADIUS - 1e-3,
            "wall at x={x} let the camera in: {to:?}"
        );
    }
    // The wall cells themselves.
    for cell in &scene.house().walls {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
    for cell in &scene.house().corners {
        assert!(is_camera_solid(scene.world().get(*cell).unwrap()));
    }
}

#[test]
fn n_enables_noclip() {
    let scene = WorldScene::new();
    let mut state = CollisionState::new();
    assert!(!state.is_noclip() && state.collision_enabled());
    assert_eq!(state.hud_suffix(), "");
    let at = world_free_fly_camera().position;
    assert_eq!(
        state.toggle_noclip(scene.world(), at, &cfg()),
        NoclipToggle::Enabled
    );
    assert!(state.is_noclip() && !state.collision_enabled());
    assert_eq!(state.hud_suffix(), " | Noclip: ON");
    // The viewer binds it to N and appends the suffix to the World HUD.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("rl.is_key_pressed(KeyboardKey::KEY_N)"));
    assert!(app.contains("self.collision_state.hud_suffix()"));
    assert!(app.contains("\"WASD move | Mouse look | Space/Shift up/down | N noclip | R reset\""));
}

#[test]
fn noclip_flies_through_solid_blocks() {
    let scene = WorldScene::new();
    let mut state = CollisionState::new();
    let from = v(16.5, -7.5, 16.5);
    state.toggle_noclip(scene.world(), from, &cfg());
    assert!(state.is_noclip());
    // With collision bypassed the free-fly state moves unresolved: into the
    // rock behind the wall.
    let mut camera = WorldFreeFlyCameraState::looking_at(from, v(16.5, -7.5, 10.0));
    for _ in 0..8 {
        apply_free_fly_input(
            &mut camera,
            &FreeFlyInput {
                move_forward: 1.0,
                delta_time: 0.25,
                ..Default::default()
            },
        );
    }
    assert!(camera.position.z < 12.0);
    assert!(
        !clear(&scene, camera.position),
        "expected solid rock at {:?}",
        camera.position
    );
    // The viewer only resolves collision while it is enabled.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("} else if self.collision_state.collision_enabled() {"));
}

#[test]
fn noclip_cannot_be_disabled_inside_a_solid() {
    let scene = WorldScene::new();
    let mut state = CollisionState::new();
    let inside_rock = v(16.5, -11.5, 10.5);
    assert!(!clear(&scene, inside_rock));
    assert_eq!(
        state.toggle_noclip(scene.world(), inside_rock, &cfg()),
        NoclipToggle::Enabled
    );
    assert_eq!(
        state.toggle_noclip(scene.world(), inside_rock, &cfg()),
        NoclipToggle::RefusedInsideSolid
    );
    assert!(state.is_noclip(), "noclip was switched off inside a solid");
    // Once back in the air it can be switched off.
    assert_eq!(
        state.toggle_noclip(scene.world(), v(16.5, -7.5, 16.5), &cfg()),
        NoclipToggle::Disabled
    );
    assert!(state.collision_enabled());
    // Inside the passable membrane counts as clear.
    let mut again = CollisionState::new();
    again.toggle_noclip(scene.world(), v(16.5, -7.5, 14.5), &cfg());
    assert_eq!(
        again.toggle_noclip(scene.world(), v(16.5, -7.5, 14.5), &cfg()),
        NoclipToggle::Disabled
    );
}

#[test]
fn portal_crossing_still_fires_in_noclip() {
    let scene = WorldScene::new();
    let mut detector =
        PortalCrossingDetector::new(PortalVolume::from_anchor(&scene.rhombus().portal));
    let mut state = CollisionState::new();
    let from = v(16.5, -7.5, 16.5);
    state.toggle_noclip(scene.world(), from, &cfg());
    let mut camera = WorldFreeFlyCameraState::looking_at(from, v(16.5, -7.5, 10.0));
    let mut event = None;
    for _ in 0..20 {
        let previous = camera.position;
        apply_free_fly_input(
            &mut camera,
            &FreeFlyInput {
                move_forward: 1.0,
                delta_time: 0.1,
                ..Default::default()
            },
        );
        // noclip: no collision resolution at all
        if event.is_none() {
            event = detector.detect(previous, camera.position, camera.realm);
        }
    }
    assert_eq!(event, Some(PortalCrossingEvent::OverworldToRedBlack));
    assert!(camera.begin_transition(event.unwrap()));
    assert!(camera.advance_transition(1.0));
    assert_eq!(camera.realm, WorldRealm::RedBlack);
    assert_eq!(camera.local_up, WorldRealm::RedBlack.up());
    // The viewer runs the detector regardless of the collision switch:
    // after (and outside) the collision branch, in the transition check.
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let resolve_at = app.find("resolve_camera_motion(\n").unwrap();
    let detect_at = app.find("self.portal_detector.detect(").unwrap();
    let gate_at = app
        .find("} else if self.free_fly.transition().is_none() {")
        .unwrap();
    assert!(resolve_at < gate_at && gate_at < detect_at);
}

#[test]
fn reset_restores_collision_and_disables_noclip() {
    let scene = WorldScene::new();
    let mut state = CollisionState::new();
    state.toggle_noclip(scene.world(), world_free_fly_camera().position, &cfg());
    assert!(state.is_noclip());
    state.reset();
    assert!(!state.is_noclip() && state.collision_enabled());
    assert!(clear(&scene, world_free_fly_camera().position));
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    assert!(app.contains("if input.reset {\n                self.collision_state.reset();"));
}

#[test]
fn the_catalog_controls_are_unchanged() {
    let app = std::fs::read_to_string("src/app.rs").unwrap();
    let catalog = &app[app.find("impl ViewerMode for CatalogMode").unwrap()
        ..app
            .find("/// The main world: the Overworld `WorldScene`.")
            .unwrap()];
    assert!(
        !catalog.contains("KEY_N") && !catalog.contains("noclip") && !catalog.contains("collision")
    );
    assert!(
        catalog
            .contains("\"Drag: orbit | Wheel: zoom | [ ] or Q E: select | F: focus | R: reset\"")
    );
    assert!(catalog.contains("poll_orbit_input(rl)"));
}

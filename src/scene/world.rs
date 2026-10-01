// World-stage scene: the main executable's scene (`cargo run`). It presents
// the definitive Overworld: the procedurally reliefed terrain mass built by
// the terrain generator, lit by a sun, with the definitive block materials.
// It never reuses `CatalogScene`, which stays the separate `catalog`
// binary's scene.
#![allow(dead_code)]

use crate::camera::camera::Camera;
use crate::camera::world_free_fly::{WorldFreeFlyCameraState, WorldRealm};
use crate::core::color::Color;
use crate::core::face_textures::FaceTextures;
use crate::core::math::{IVec3, Vec3};
use crate::renderer::skybox::{Background, SkyGradient};
use crate::scene::castle::{
    CastleFoundation, CastleGatehouse, CastleKeep, CastleTowers, OverworldCastleLayout,
    build_castle_foundation, build_castle_gatehouse, build_castle_keep, build_castle_towers,
};
use crate::scene::cutaway::{Cutaway, carve_cutaway};
use crate::scene::descent::{
    InvertedRoute, UpperDescent, build_inverted_route, build_upper_descent,
};
use crate::scene::environment::WorldEnvironmentProfile;
use crate::scene::expansion::{
    EXPANSION_WIDTH, ExpansionScenery, OverworldApproach, OverworldExpansion, OverworldRelief,
    RedBlackApproach, RedBlackExpansion, RedBlackSurfaceExtension, WorldExpansionLayout,
    build_overworld_expansion, build_red_black_expansion, decorate_overworld_expansion,
    decorate_red_black_expansion, extend_red_black_surface, pave_fortress_approach,
    pave_overworld_approach, shape_overworld_relief,
};
use crate::scene::light::{DirectionalLight, Light};
use crate::scene::material_gallery::{
    GalleryTextures, advanced_materials_library, deepslate_bricks_material_id, glass_material_id,
    leaves_material_id, portal_core_material_id, redstone_lamp_material_id, water_material_id,
};
use crate::scene::material_library::MaterialLibrary;
use crate::scene::overworld::{
    HouseExterior, HouseLayout, PathLayout, PondLayout, Tree, build_house, carve_pond, column_top,
    furnish_house, lay_path, plant_trees,
};
use crate::scene::overworld_blocks::{OverworldBlockTextures, insert_overworld_materials};
use crate::scene::portal::{PortalBuild, build_portal_core, build_portal_frame, portal_light};
use crate::scene::red_black_maze::{
    FamilyLayout, FamilyTransitions, LowerMass, RedBlackSurface, blend_family_transitions,
    build_lower_mass, build_red_black_surface, compose_crimson, compose_orange, compose_violet,
    family_lights,
};
use crate::scene::rhombus::{
    RhombusConfig, Silhouette, UpperTaper, build_upper_taper, finalize_silhouette,
};
use crate::scene::scene::{diagnostic_materials, grass_material_id};
use crate::scene::terrain::TerrainConfig;
use crate::scene::terrain::fbm::HeightField;
use crate::scene::terrain::generator::{TerrainBounds, generate_terrain, terrain_bounds};
use crate::scene::texture_manager::{TextureLoadError, TextureManager};
use crate::scene::voxel_world::VoxelWorld;

/// Scene range of the world: primary traversal and directional shadow rays.
pub const WORLD_MAX_DISTANCE: f32 = 96.0;

/// Ambient term of the daytime Overworld: enough to keep shadowed faces and
/// the undersides readable under the bright sky, low enough that the sun's
/// shadows stay clearly visible.
pub const WORLD_AMBIENT_FACTOR: f32 = 0.30;

/// The main world scene: the Overworld terrain and everything built on it.
pub struct WorldScene {
    config: TerrainConfig,
    field: HeightField,
    bounds: TerrainBounds,
    pond: PondLayout,
    house: HouseLayout,
    exterior: HouseExterior,
    trees: Vec<Tree>,
    path: PathLayout,
    rhombus: RhombusConfig,
    taper: UpperTaper,
    cutaway: Cutaway,
    upper_descent: UpperDescent,
    portal: PortalBuild,
    lower_mass: LowerMass,
    inverted_route: InvertedRoute,
    lower_surface: RedBlackSurface,
    families: Vec<FamilyLayout>,
    transitions: FamilyTransitions,
    expansion_layout: WorldExpansionLayout,
    overworld_expansion: OverworldExpansion,
    overworld_relief: OverworldRelief,
    overworld_approach: OverworldApproach,
    red_black_expansion: RedBlackExpansion,
    red_black_surface_extension: RedBlackSurfaceExtension,
    red_black_approach: RedBlackApproach,
    expansion_scenery: ExpansionScenery,
    silhouette: Silhouette,
    castle_layout: OverworldCastleLayout,
    castle_foundation: CastleFoundation,
    castle_towers: CastleTowers,
    castle_gatehouse: CastleGatehouse,
    castle_keep: CastleKeep,
    world: VoxelWorld,
}

impl WorldScene {
    /// Builds the official Overworld.
    pub fn new() -> Self {
        Self::with_config(TerrainConfig::official())
    }

    /// Builds the Overworld of an arbitrary terrain configuration.
    pub fn with_config(config: TerrainConfig) -> Self {
        let mut world = VoxelWorld::new();
        let field = generate_terrain(&config, &mut world);
        let bounds = terrain_bounds(&config, &field);
        let pond = carve_pond(&config, &mut world);
        let house = build_house(&config, &mut world);
        let exterior = furnish_house(&config, &mut world);
        let trees = plant_trees(&config, &mut world, &pond);
        let path = lay_path(&config, &mut world, &pond, &trees);
        // The diamond below the Overworld.
        let rhombus = RhombusConfig::derive(&config);
        let taper = build_upper_taper(&config, &rhombus, &mut world);
        let cutaway = carve_cutaway(&config, &rhombus, &mut world);
        let upper_descent = build_upper_descent(&config, &rhombus, &mut world);
        let mut portal = build_portal_frame(&rhombus, &mut world);
        build_portal_core(&rhombus, &mut world, &mut portal);
        let lower_mass = build_lower_mass(&config, &rhombus, &mut world);
        let inverted_route = build_inverted_route(&rhombus, &lower_mass, &mut world);
        let lower_surface = build_red_black_surface(&config, &rhombus, &lower_mass, &mut world);
        let families = vec![
            compose_crimson(&config, &rhombus, &lower_surface, &mut world),
            compose_orange(&config, &rhombus, &lower_surface, &mut world),
            compose_violet(&config, &rhombus, &lower_surface, &mut world),
        ];
        let transitions = blend_family_transitions(&config, &rhombus, &lower_surface, &mut world);
        // Gate 15: where both worlds grow (derived from what stands so far).
        let expansion_layout =
            WorldExpansionLayout::derive(&config, &rhombus, &inverted_route.layout, &path, &world);
        let mut overworld_expansion =
            build_overworld_expansion(&config, &expansion_layout, &mut world);
        let overworld_relief = shape_overworld_relief(
            &config,
            &expansion_layout,
            &mut overworld_expansion,
            &mut world,
        );
        let overworld_approach = pave_overworld_approach(&config, &expansion_layout, &mut world);
        let mut red_black_expansion = build_red_black_expansion(
            &config,
            &rhombus,
            &expansion_layout,
            &overworld_expansion,
            &mut world,
        );
        let red_black_surface_extension = extend_red_black_surface(
            &config,
            &rhombus,
            &expansion_layout,
            &mut red_black_expansion,
            &mut world,
        );
        let red_black_approach =
            pave_fortress_approach(&config, &rhombus, &expansion_layout, &mut world);
        let expansion_scenery = ExpansionScenery {
            upper: decorate_overworld_expansion(
                &config,
                &expansion_layout,
                &overworld_expansion,
                &overworld_approach,
                &mut world,
            ),
            lower: decorate_red_black_expansion(
                &config,
                &rhombus,
                &expansion_layout,
                &red_black_expansion,
                &red_black_approach,
                &families
                    .iter()
                    .flat_map(|f| f.symbols.iter().chain(f.accents.iter()).copied())
                    .collect::<Vec<_>>(),
                &mut world,
            ),
        };
        let silhouette = finalize_silhouette(&config, &rhombus, &mut world);
        // Gate 16: the castle contract on the pad (built after the silhouette
        // so its hollow rooms are never taken for voids).
        let castle_layout = OverworldCastleLayout::derive(&expansion_layout);
        let castle_foundation = build_castle_foundation(&config, &castle_layout, &mut world);
        let castle_towers = build_castle_towers(&config, &castle_layout, &mut world);
        let castle_gatehouse = build_castle_gatehouse(&config, &castle_layout, &mut world);
        let castle_keep = build_castle_keep(&config, &castle_layout, &mut world);
        Self {
            config,
            field,
            bounds,
            pond,
            house,
            exterior,
            trees,
            path,
            rhombus,
            taper,
            cutaway,
            upper_descent,
            portal,
            lower_mass,
            inverted_route,
            lower_surface,
            families,
            transitions,
            expansion_layout,
            overworld_expansion,
            overworld_relief,
            overworld_approach,
            red_black_expansion,
            red_black_surface_extension,
            red_black_approach,
            expansion_scenery,
            silhouette,
            castle_layout,
            castle_foundation,
            castle_towers,
            castle_gatehouse,
            castle_keep,
            world,
        }
    }

    /// The spatial contract of the diamond.
    pub fn rhombus(&self) -> &RhombusConfig {
        &self.rhombus
    }

    /// The mass added under the terrain down to the shelf.
    pub fn upper_taper(&self) -> &UpperTaper {
        &self.taper
    }

    /// The physical cutaway (what was removed).
    pub fn cutaway(&self) -> &Cutaway {
        &self.cutaway
    }

    /// The stepped route from the path endpoint down to the portal approach.
    pub fn upper_descent(&self) -> &UpperDescent {
        &self.upper_descent
    }

    /// The portal at the waist.
    pub fn portal(&self) -> &PortalBuild {
        &self.portal
    }

    /// The structural lower half under the waist.
    pub fn lower_mass(&self) -> &LowerMass {
        &self.lower_mass
    }

    /// The post-portal route: exit clearance and the inverted flight from
    /// the membrane to the Red-Black underside.
    pub fn inverted_route(&self) -> &InvertedRoute {
        &self.inverted_route
    }

    /// The -Y-facing ground of the inverted world.
    pub fn lower_surface(&self) -> &RedBlackSurface {
        &self.lower_surface
    }

    /// The visual families composed over the lower surface.
    pub fn families(&self) -> &[FamilyLayout] {
        &self.families
    }

    /// The blended boundaries between the families.
    pub fn transitions(&self) -> &FamilyTransitions {
        &self.transitions
    }

    /// The final silhouette pass (voids filled, fragments removed, tip).
    /// Where both worlds grow and where the castles will stand (Gate 15).
    pub fn expansion_layout(&self) -> &WorldExpansionLayout {
        &self.expansion_layout
    }

    /// The new upper terrain of the east lobe (Gate 15).
    pub fn overworld_expansion(&self) -> &OverworldExpansion {
        &self.overworld_expansion
    }

    /// The relief sculpted into the east lobe (Gate 15).
    pub fn overworld_relief(&self) -> &OverworldRelief {
        &self.overworld_relief
    }

    /// The paved approach to the castle site (Gate 15).
    pub fn overworld_approach(&self) -> &OverworldApproach {
        &self.overworld_approach
    }

    /// The lower shell of the east lobe (Gate 15).
    pub fn red_black_expansion(&self) -> &RedBlackExpansion {
        &self.red_black_expansion
    }

    /// The -Y-facing surface over the east lobe (Gate 15).
    pub fn red_black_surface_extension(&self) -> &RedBlackSurfaceExtension {
        &self.red_black_surface_extension
    }

    /// The paved lower approach to the fortress site (Gate 15).
    pub fn red_black_approach(&self) -> &RedBlackApproach {
        &self.red_black_approach
    }

    /// The restrained scenery of both expansions (Gate 15).
    pub fn expansion_scenery(&self) -> &ExpansionScenery {
        &self.expansion_scenery
    }

    /// The Overworld castle contract on the pad (Gate 16).
    pub fn castle_layout(&self) -> &OverworldCastleLayout {
        &self.castle_layout
    }

    /// The castle's footings, floors and curtain wall (Gate 16).
    pub fn castle_foundation(&self) -> &CastleFoundation {
        &self.castle_foundation
    }

    /// Cells the Gate 16 castle added above the pad (its ground row only
    /// replaces Gate 15 cells).
    pub fn castle_voxels(&self) -> usize {
        self.castle_cells().len()
    }

    /// Every castle cell above the pad's ground row, each once.
    pub fn castle_cells(&self) -> Vec<IVec3> {
        let mut seen = std::collections::HashSet::new();
        let ground = self.castle_layout.levels.ground_y;
        self.castle_foundation
            .walls
            .iter()
            .chain(self.castle_towers.cells().iter())
            .chain(self.castle_gatehouse.cells().iter())
            .chain(self.castle_keep.cells().iter())
            .filter(|c| c.y > ground && seen.insert(**c))
            .copied()
            .collect()
    }

    /// The gatehouse and its door (Gate 16).
    pub fn castle_gatehouse(&self) -> &CastleGatehouse {
        &self.castle_gatehouse
    }

    /// The keep (Gate 16).
    pub fn castle_keep(&self) -> &CastleKeep {
        &self.castle_keep
    }

    /// The towers and battlements (Gate 16).
    pub fn castle_towers(&self) -> &CastleTowers {
        &self.castle_towers
    }

    pub fn silhouette(&self) -> &Silhouette {
        &self.silhouette
    }

    /// The cobblestone path from the door to the descent point.
    pub fn path(&self) -> &PathLayout {
        &self.path
    }

    /// The roof, door, glass, fence and lamps of the house.
    pub fn house_exterior(&self) -> &HouseExterior {
        &self.exterior
    }

    /// The house built on the plateau.
    pub fn house(&self) -> &HouseLayout {
        &self.house
    }

    /// The trees planted on the terrain.
    pub fn trees(&self) -> &[Tree] {
        &self.trees
    }

    /// The pond dug into the terrain.
    pub fn pond(&self) -> &PondLayout {
        &self.pond
    }

    /// Highest occupied cell of column `(x, z)` (terrain or feature), or
    /// `None` for an empty column.
    pub fn column_top(&self, x: i32, z: i32) -> Option<i32> {
        column_top(&self.world, x, z, self.bounds.max.y + 16, self.bounds.min.y)
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }

    pub fn config(&self) -> &TerrainConfig {
        &self.config
    }

    /// The height field the terrain columns were generated from.
    pub fn height_field(&self) -> &HeightField {
        &self.field
    }

    /// Bounds of the terrain mass.
    pub fn bounds(&self) -> TerrainBounds {
        self.bounds
    }

    /// World-space center of the terrain mass.
    pub fn center(&self) -> Vec3 {
        let b = self.bounds;
        Vec3::new(
            (b.min.x + b.max.x + 1) as f32 / 2.0,
            (b.min.y + b.max.y + 1) as f32 / 2.0,
            (b.min.z + b.max.z + 1) as f32 / 2.0,
        )
    }
}

impl Default for WorldScene {
    fn default() -> Self {
        Self::new()
    }
}

/// Every texture the world needs, loaded once through the shared manager.
pub struct WorldTextures {
    pub blocks: OverworldBlockTextures,
    /// Grass Block faces (`top/side/bottom.png`).
    pub grass: FaceTextures,
    /// The optical specimens the world reuses (water, and later glass,
    /// leaves and the redstone lamp), loaded exactly as the catalog does.
    pub gallery: GalleryTextures,
}

impl WorldTextures {
    pub fn load(
        manager: &mut TextureManager,
        overworld_dir: &str,
        grass_dir: &str,
        portal_dir: &str,
    ) -> Result<Self, TextureLoadError> {
        let blocks = OverworldBlockTextures::load(manager, overworld_dir)?;
        let gallery = GalleryTextures::load(manager, overworld_dir, portal_dir)?;
        let top = manager.load(format!("{grass_dir}/top.png"))?;
        let side = manager.load(format!("{grass_dir}/side.png"))?;
        let bottom = manager.load(format!("{grass_dir}/bottom.png"))?;
        Ok(Self {
            blocks,
            grass: FaceTextures::new(side, side, top, bottom, side, side),
            gallery,
        })
    }
}

/// The world's materials: the same definitive block materials the catalog
/// uses, registered through the shared `insert_overworld_materials`, the
/// Grass Block exactly as the catalog presents it, and the optical
/// specimens the scene needs copied from the catalog's gallery library
/// (same ids, same profiles).
pub fn world_materials(textures: &WorldTextures) -> MaterialLibrary {
    let mut library = MaterialLibrary::new();
    insert_overworld_materials(&mut library, &textures.blocks);

    let gallery = advanced_materials_library(&textures.gallery);
    for id in [
        water_material_id(),
        leaves_material_id(),
        glass_material_id(),
        redstone_lamp_material_id(),
        deepslate_bricks_material_id(),
        portal_core_material_id(),
    ] {
        let material = gallery
            .get(id)
            .expect("the gallery defines every optical specimen")
            .clone();
        library.insert(id, material);
    }

    let grass = diagnostic_materials(textures.grass)
        .get(grass_material_id())
        .expect("the grass material is always defined")
        .clone()
        .with_reflectivity(0.01);
    library.insert(grass_material_id(), grass);

    library
}

/// Direction from any surface toward the sun: high and diagonal (from the
/// south-east and above), so every block shows one lit top, one lit side and
/// one shaded side, with legible cast shadows.
pub fn sun_direction() -> Vec3 {
    Vec3::new(0.45, 1.0, 0.35)
}

/// The sun's color: white with a faint warm tint.
pub fn sun_color() -> Color {
    Color::new(1.0, 0.97, 0.90, 1.0)
}

/// The world's lights: the sun, the portal's crimson point light at the
/// waist, and one accent light per Red-Black family under the tip. Every
/// other glow (lamps, symbols, crying obsidian, amethyst) is emissive
/// texture only, so the light count stays bounded.
pub fn world_lights() -> Vec<Light> {
    let terrain = TerrainConfig::official();
    let rhombus = RhombusConfig::derive(&terrain);
    let mut lights = vec![
        Light::Directional(DirectionalLight::new(sun_direction(), sun_color(), 1.0)),
        portal_light(&rhombus),
    ];
    lights.extend(family_lights(&terrain, &rhombus));
    lights
}

/// The clear daytime sky of the Overworld (the `Day` environment profile,
/// hung along +Y).
pub fn world_sky() -> SkyGradient {
    WorldEnvironmentProfile::DAY.sky(WorldRealm::Overworld.up())
}

/// What World rays see on a miss: the daytime sky.
pub fn world_background() -> Background {
    Background::Sky(world_sky())
}

/// World-space point the official framing looks at: the volumetric center
/// of the whole diamond (middle of the footprint, halfway between the
/// Overworld surface and the lower tip), so the upper micro-scene, the
/// cutaway, the portal waist and the inverted half balance in the frame.
pub fn world_focus() -> Vec3 {
    let config = TerrainConfig::official();
    let rhombus = RhombusConfig::derive(&config);
    Vec3::new(
        (config.width + EXPANSION_WIDTH) as f32 / 2.0,
        (rhombus.upper_surface_reference + rhombus.lower_tip_y) as f32 / 2.0 + 1.0,
        config.depth as f32 / 2.0,
    )
}

/// Eye offset of the official framing: a raised three-quarter view from the
/// south-south-east, looking into the cutaway quadrant with the pond still
/// in view on the west, far enough to hold the
/// complete floating diamond from the tree crowns to the lower tip, and
/// low enough that the exposed layers, portal and underside all read.
pub const WORLD_CAMERA_OFFSET: Vec3 = Vec3 {
    x: 12.0,
    y: 20.0,
    z: 38.0,
};

/// The World's free-fly camera at the official framing: the presentation
/// pose the diorama is first seen from and what `R` resets to.
pub fn world_free_fly_camera() -> WorldFreeFlyCameraState {
    WorldFreeFlyCameraState::looking_at(world_focus() + WORLD_CAMERA_OFFSET, world_focus())
}

/// The official diorama framing: the orbit camera's starting pose and
/// what `R` resets to.
pub fn world_camera(aspect_ratio: f32) -> Camera {
    let focus = world_focus();
    Camera::new(
        focus + WORLD_CAMERA_OFFSET,
        focus,
        Vec3::new(0.0, 1.0, 0.0),
        60.0,
        aspect_ratio,
    )
}

// Shared CPU raytracing runtime: the one module tree (math, rays, camera,
// renderer, materials, textures, voxel scene, catalog) every executable of
// the project links against. The modules stay private; each binary only
// calls one application entrypoint, so no binary can grow its own copy of
// the engine.

mod app;
mod camera;
mod config;
mod core;
mod renderer;
mod scene;

/// Opens the interactive block catalog (`CatalogScene`): every official
/// block plus the optical diagnostics, with selection, focus, orbit, zoom
/// and reset.
pub use app::run_catalog_app;

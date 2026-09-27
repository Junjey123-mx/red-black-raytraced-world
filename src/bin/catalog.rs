// Permanent block catalog viewer: `cargo run --bin catalog`. It owns no
// engine code; it launches the shared library's catalog application.
fn main() {
    red_black_raytraced_world::run_catalog_app();
}

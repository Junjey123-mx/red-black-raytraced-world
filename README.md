# Red-Black Raytraced World

> Two realms. One portal. A voxel diorama ray-traced entirely on the CPU.

![The floating diorama: the Overworld on top, the portal in the cutaway, the inverted Red-Black realm hanging underneath](docs/screenshots/readme/01-hero-world.png)

A floating island under a clouded sky, with a house, a pond and a castle. Cut into its side, a
crimson portal. Step through it and the world turns over: the same island seen from below is a
second realm, dark and inverted, where a four-tower fortress hangs from the ceiling and card
suits glow in the stone.

Every pixel of it is a ray traced in Rust on the CPU, with no rendering library behind it.

**Jump to:** [The two realms](#explore-the-two-realms) · [Run it](#run-the-project) ·
[Controls](#controls) · [Materials](#materials-that-change-the-light) ·
[Ray tracing](#ray-tracing-features) · [Performance](#how-a-cpu-ray-tracer-became-walkable) ·
[Rubric coverage](#assignment-rubric-coverage) · [Architecture](#architecture--engineering-decisions)

---

## What is Red-Black Raytraced World?

It is a small Minecraft-inspired voxel diorama built for the Computer Graphics course at
Universidad del Valle de Guatemala, and a ray tracer written from scratch to render it.

- **One world, two realms.** A single voxel grid holds both the bright Overworld and the
  inverted Red-Black realm. They are the two faces of the same floating diamond.
- **A physical portal.** The realms are joined by a real opening you fly through. Crossing it
  rolls the camera 180°, flips which way is up and swaps the sky.
- **A CPU ray tracer.** One ray per pixel, a 3D DDA through the voxel grid, textured cubes and
  partial shapes, shadows, reflection, refraction, normal mapping, emissive materials and a
  cubemap skybox.
- **Walkable.** Parallel rendering and an adaptive preview keep it interactive, with collision
  so you can walk the paths and enter the buildings.

| | |
|---|---|
| World size | 9,633 voxels, 45 block types |
| Textures | 95 PNG files, all sampled nearest-neighbor on the CPU |
| Lights | 1 directional sun, 4 point lights (portal and the three Red-Black families) |
| Tests | 1,931 passing |

---

## Explore the Two Realms

![Front view: the Overworld above, the portal in the cut, the Red-Black realm and its hanging house below](docs/screenshots/readme/14-two-worlds.png)

The island is cut open on one side so you can read it like a cross-section: grass and dirt on
top, stone and deepslate beneath, the portal at the waist, and the Red-Black realm growing
downward from the underside.

### Overworld

<table>
<tr>
<td width="50%"><img src="docs/screenshots/readme/02-overworld.png" alt="The Overworld from above: house, pond with a sandy shore, trees, cobblestone path and castle"></td>
<td width="50%"><img src="docs/screenshots/readme/16-skybox-day.png" alt="The house and the castle under the clouded day sky"></td>
</tr>
</table>

The bright half. Everything here is laid out on procedurally generated terrain.

- **Terrain** — rolling relief from fractal noise, with grass, dirt, stone and deepslate strata
  exposed on the island's flanks.
- **House** — timber walls, log corner posts, a stair-stepped roof, glass windows and two lit
  lamps by the door.
- **Pond** — refractive water in a sandy hollow.
- **Trees and paths** — leaf canopies with cut-out foliage, and a cobblestone path from the
  house to the castle.
- **Castle** — a stone keep with battlements, a furnished interior and a staircase.
- **Sky** — a cubemap of clouds with the sun where the scene's light comes from.

<table>
<tr>
<td width="50%"><img src="docs/screenshots/readme/03-overworld-castle.png" alt="The Overworld castle and the path leading to its entrance"></td>
<td width="50%"><img src="docs/screenshots/readme/18-house-glass.png" alt="The house front: glass windows, lit lamps, door and porch fence"></td>
</tr>
<tr>
<td><img src="docs/screenshots/readme/04-overworld-interior.png" alt="Inside the castle, lit by a redstone lamp"></td>
<td><img src="docs/screenshots/readme/15-pond.png" alt="The pond seen from its sandy bank"></td>
</tr>
</table>

### Cross the Portal

![The portal: an emissive crimson core in a red-obsidian frame, with the stairs leading down to it](docs/screenshots/readme/07-portal.png)

A staircase descends through the cutaway to a brick landing. In front of it stands the portal:
a frame of red obsidian around a swirling core that emits its own light and tints the stone
around it.

Fly through the core and the crossing begins:

- the camera **rolls 180°** around its own view;
- "up" becomes `-Y`, so the island's underside is now the ground over your head;
- the day sky **cross-fades** into a starred cornelian night;
- your controls stay consistent: forward is still forward.

Fly back through and everything unwinds.

### Red-Black

![The Red-Black fortress hanging beneath the island, with crimson trees beside it](docs/screenshots/readme/09-red-black-fortress.png)

The inverted half. Here the world hangs downward, and its light comes from the blocks
themselves.

- **Three families** — Crimson, Orange and Violet. Each owns a region of the underside and a
  set of glowing card-suit blocks: hearts, diamonds, clubs and spades.
- **Fortress** — four towers, a gatehouse with a corinto-wood door, a courtyard, a hall with a
  throne and a staircase to an upper room.
- **Approach** — a path of corinto planks from the portal's landing to the gate.
- **Crimson trees** — dark trunks with red foliage, hanging toward the sky below.
- **The house** — the Overworld's house rebuilt upside down in corinto timber, joined to the
  fortress by a wooden path and a flight of stairs.
- **Glowing stone** — crying obsidian in each family's colour, and amethyst clusters.

<table>
<tr>
<td width="50%"><img src="docs/screenshots/readme/08-red-black-approach.png" alt="The corinto path leading to the fortress gatehouse, with an amethyst cluster beside it"></td>
<td width="50%"><img src="docs/screenshots/readme/20-red-black-house.png" alt="The Red-Black house with its lit lamps, on the underside of the island"></td>
</tr>
<tr>
<td><img src="docs/screenshots/readme/10-red-black-interior.png" alt="Inside the fortress: timber, dark stone and glowing suit symbols"></td>
<td><img src="docs/screenshots/readme/21-red-black-house-interior.png" alt="Inside the Red-Black house: corinto walls and glass windows looking out on the realm"></td>
</tr>
</table>

![The fortress silhouetted against the eclipsed moon of the Red-Black sky](docs/screenshots/readme/17-skybox-night.png)

---

## Animated Preview

> Animated gameplay preview coming from the final recorded route.

<!-- DEMO_GIF: replace this comment with the final GIF before submission -->

## Final Video

> The final walkthrough video will be embedded here before submission.

<!-- FINAL_VIDEO_URL: paste the final YouTube/video link here before submission -->

---

## Run the Project

Requires a Rust toolchain (edition 2024) and the system packages needed to build
[raylib](https://github.com/raysan5/raylib) (a C compiler and CMake).

```bash
# Explore the world
cargo run --release

# Browse every block in the catalog viewer
cargo run --bin catalog --release

# Run the test suite
cargo test
```

Use `--release` to explore. The ray tracer is CPU-bound, and the release build is what the
performance figures below were measured with.

| Variable | Effect |
|---|---|
| `RBRW_THREADS=<n>` | Number of render workers (default: every hardware thread) |
| `RBRW_PERF=1` | Print per-frame render timing to the terminal |

---

## Controls

**World** (`cargo run --release`)

| Input | Action |
|---|---|
| `W` `A` `S` `D` | Fly forward, left, back, right |
| Mouse | Look around |
| `Q` / `E` | Orbit the camera around the diorama |
| Mouse wheel | Zoom in and out along the view |
| `Space` / `Left Shift` | Move up / down |
| `N` | Toggle noclip (collision is on by default) |
| `R` | Reset to the opening view |

The mouse is captured while the world is open; close the window to quit.

**Catalog** (`cargo run --bin catalog --release`)

| Input | Action |
|---|---|
| Left-drag or arrow keys | Orbit |
| Mouse wheel | Zoom |
| `[` `]` or `Q` `E` | Select the previous / next block |
| `F` | Focus the selected block |
| `R` | Reset the view |

---

## Block Catalog

![The catalog viewer: every block on a checkered floor](docs/screenshots/readme/13-catalog.png)

The world is built from **45 official blocks**, and all 45 are placed in the final scene. The
catalog viewer shows each one on its own, with an orbit camera, so its texture, shape and
material can be inspected in isolation.

<details>
<summary><b>Overworld terrain</b> (9)</summary>

Grass · Dirt · Stone · Cobblestone · Sand · Water · Deepslate · Log · Leaves
</details>

<details>
<summary><b>Overworld architecture</b> (8)</summary>

Deepslate Bricks · Wood Planks · Double Wood Slab · Wood Stairs · Fence · Glass ·
Redstone Lamp (lit) · Wood Door
</details>

<details>
<summary><b>Portal</b> (2)</summary>

Portal Frame (red obsidian) · Portal Core (dark crimson)
</details>

<details>
<summary><b>Red-Black base</b> (4)</summary>

Mycelium · Smooth Basalt · Budding Amethyst · Amethyst Cluster
</details>

<details>
<summary><b>Red-Black symbols and families</b> (12)</summary>

**Crimson:** Crimson Heart · Crimson Diamond · Crying Obsidian (crimson) · Nether Wart Block<br>
**Orange:** Orange Club · Orange Spade · Crying Obsidian (orange)<br>
**Violet:** Purple Heart · Purple Diamond · Purple Club · Purple Spade · Crying Obsidian (violet)
</details>

<details>
<summary><b>Red-Black structural</b> (4)</summary>

Red-Black Deepslate Bricks (crimson, orange, violet) · Polished Blackstone Bricks
</details>

<details>
<summary><b>Red-Black organic and timber</b> (6)</summary>

Red-Black Leaves · Red-Black Log · Red-Black Wood Planks · Red-Black Fence ·
Red-Black Wood Door · Red-Black Wood Stairs
</details>

Not every block is a cube. Stairs, fences, doors, the portal core and the amethyst cluster are
built from smaller boxes and intersected exactly.

---

## Materials That Change the Light

Every material carries its own texture and its own optical parameters. These seven show the
range; the values are the ones the renderer uses.

| Material | Texture | Albedo (RGB) | Specular / shininess | Transparency | Reflectivity | Special effect | Where to see it |
|---|---|---|---|---|---|---|---|
| **Water** | `overworld/water.png` | 0.20, 0.38, 0.75 | 0.55 / 80 | 0.78 | 0.18 | Refraction, index 1.33 | The pond |
| **Glass** | `overworld/glass.png` | 0.92, 0.97, 1.00 | 0.75 / 140 | 0.92 | 0.10 | Refraction, index 1.50; opaque frame from texture alpha | House, castle and Red-Black house windows |
| **Deepslate Bricks** | `deepslate_bricks/albedo.png` | 0.25, 0.25, 0.25 | 0.08 / 14 | 0 | 0.02 | Normal map | Castle base, portal landing |
| **Redstone Lamp** | `redstone_lamp/albedo.png` | 0.45, 0.35, 0.25 | 0.10 / 20 | 0 | 0 | Emissive mask, strength 1.8 | Beside every door |
| **Portal Core** | `portal/core_albedo.png` | 0.42, 0.05, 0.18 | 0.12 / 30 | 0.25 | 0.03 | Emissive mask, strength 2.5 | The portal |
| **Crying Obsidian** | `crying_obsidian/<family>_albedo.png` | 0.30, 0.03, 0.07 | 0.30 / 60 | 0 | 0.12 | Reflective, emissive veins (1.4) | Fortress towers |
| **Amethyst Cluster** | `amethyst_cluster.png` | 0.58, 0.40, 0.82 | 0.65 / 110 | 0.08 | 0.20 | Reflective, refractive (1.45), emissive (0.6) | Along the Red-Black paths |

<table>
<tr>
<td width="50%"><img src="docs/screenshots/readme/06-water-refraction.png" alt="The pond at water level: the far bank bends through the water and the house lamps reflect on its surface"></td>
<td width="50%"><img src="docs/screenshots/readme/11-emissive-materials.png" alt="The underside of the island: glowing suit symbols, crying obsidian and amethyst"></td>
</tr>
<tr>
<td><b>Refraction and reflection.</b> At water level the pond's far wall bends through the surface, and the house lamps reflect on it.</td>
<td><b>Emission.</b> Suit symbols, crying obsidian and amethyst light the Red-Black realm from within.</td>
</tr>
</table>

---

## Ray-Tracing Features

The renderer is a classic Whitted-style ray tracer specialised for a voxel grid.

| Stage | What happens |
|---|---|
| **Camera rays** | One primary ray per pixel from a pinhole camera (60° field of view) |
| **Voxel traversal** | A 3D DDA walks the grid cell by cell; rays are first clipped to the world's bounds |
| **Intersection** | Full cubes, or the exact boxes of a partial shape (stairs, fence, door, cluster) |
| **Texturing** | Per-face textures, nearest-neighbor sampling to keep the pixel art crisp |
| **Diffuse and specular** | Lambert plus a Phong highlight, per light |
| **Shadows** | A shadow ray per light; cut-out texels and transparent blocks let light through |
| **Reflection** | A mirror ray weighted by the material's reflectivity |
| **Refraction** | Snell's law with total internal reflection, for water, glass and amethyst |
| **Normal mapping** | A tangent-space normal texture perturbs the shading normal |
| **Emission** | An emissive mask adds light the surface gives off itself |
| **Transparency** | Scalar transparency, per-texel alpha blending and alpha cut-out |
| **Skybox** | Rays that hit nothing sample a six-face cubemap by direction |

Secondary rays recurse to a depth of 3.

### Normal mapping, on and off

The same view of the portal's brick landing, lit by the portal. Only the normal map differs.

<table>
<tr>
<td width="50%"><img src="docs/screenshots/readme/12-normal-map-off.png" alt="The brick landing with the normal map disabled: flat shading"></td>
<td width="50%"><img src="docs/screenshots/readme/12-material-detail.png" alt="The same bricks with the normal map: each brick's edges catch the portal's light"></td>
</tr>
<tr>
<td align="center">Normal map disabled (comparison render)</td>
<td align="center">Normal map enabled (the shipped scene)</td>
</tr>
</table>

With the map, each brick's bevel catches the portal's light and the joints fall into shadow.

### Skybox

The sky is a real cubemap: six 128×128 images per realm, sampled on the CPU by the direction
of every ray that leaves the world. Reflections and refractions see it too.

- **Overworld** — blocky clouds over a blue gradient, with a square sun placed exactly where
  the scene's directional light comes from.
- **Red-Black** — stars in the three family colours, slow nebula bands and an eclipsed moon.

The cube hangs along the realm's "up", so it turns over with the camera as you cross the
portal, and the two skies cross-fade during the roll.

---

## Procedural Terrain

![The island from the south: relief on top, strata on its flanks](docs/screenshots/readme/05-procedural-terrain.png)

The Overworld's terrain is generated, not placed by hand.

| | |
|---|---|
| Area | **24 × 24 columns** (576 columns; the assignment asks for at least 16 × 16) |
| Algorithm | Fractional Brownian motion over 2D value noise |
| Octaves | 4, with lacunarity 2.0 and gain 0.5 |
| Base wavelength | About 10 blocks, so a hill spans several columns |
| Height mapping | Base height 4, amplitude 3 |
| Strata | Grass on top, 3 layers of dirt, then stone, then deepslate from `y = -1` down |
| Determinism | Seeded integer hash: the same seed always builds the same island |

The noise decides the height of every column; the strata are filled from that height down.
The pond, the house site and the paths are then carved and levelled on top of the generated
relief. The island's tapered underside and its 13 × 12 castle lobe extend the same mass.

---

## How a CPU Ray Tracer Became Walkable

A naive version of this renderer took seconds per frame. These are the steps that made it
interactive.

- **World-bounds clipping** — every ray is clipped to the world's bounding box before the DDA
  starts, so sky rays cost almost nothing.
- **Multithreading** — the frame is split into bands of rows and traced by scoped
  `std::thread` workers, one per hardware thread (`available_parallelism`).
- **Two qualities** — *Interactive* skips the mirror bounce on barely reflective materials;
  *Full* traces everything.
- **Adaptive preview** — while the camera moves, the frame is traced at half or quarter
  resolution, chosen from the measured render time against a 50 ms budget.
- **Cancellable refinement** — when the camera rests, a full-resolution Full-quality frame is
  traced in small batches between input polls; any movement cancels it instantly.
- **Buffer reuse** — the framebuffer and the presentation texture are allocated once and
  updated in place.

### Measured performance

Release build, 20 worker threads, Interactive quality at 400×300 (the moving-camera preview).

| Scenario | Render time | Frame rate |
|---|---|---|
| Full diorama (opening view) | 26.7 ms | 36 FPS |
| Overworld with castle lobe | 28.1 ms | 35 FPS |
| Overworld castle, exterior | 19.7 ms | 48 FPS |
| Overworld castle, interior | 19.9 ms | 49 FPS |
| Portal and cutaway | 49.3 ms | 20 FPS |
| Just after crossing | 24.9 ms | 39 FPS |
| Red-Black, wide | 44.1 ms | 22 FPS |
| Fortress approach | 34.9 ms | 28 FPS |
| Fortress, exterior | 29.8 ms | 32 FPS |
| Fortress, interior | 20.9 ms | 46 FPS |
| Red-Black grove | 28.8 ms | 34 FPS |

Every scenario stays above the 15 FPS floor the project set for itself. A Full-quality frame
of the same views takes between 93 ms and 373 ms. The benchmark is part of the test suite:

```bash
cargo test --release --test gate17_5_red_black_identity_closure_tests bench_release_identity -- --ignored --nocapture
```

---

## Assignment Rubric Coverage

| Criterion | Points | Implementation | Visible evidence | Status |
|---|---|---|---|---|
| Scene complexity | 20 | Two realms in one voxel world, a portal, a castle, a fortress, two houses, interiors, paths, trees, 45 block types | [Hero](docs/screenshots/readme/01-hero-world.png), [two worlds](docs/screenshots/readme/14-two-worlds.png) | Covered |
| Visual appeal | 15 | Contrasting day and night realms, emissive Red-Black lighting, cubemap skies | [Overworld](docs/screenshots/readme/02-overworld.png), [eclipse](docs/screenshots/readme/17-skybox-night.png) | Covered |
| Parallel programming and optimisation | 10 | Scoped threads over row bands, bounds clipping, adaptive preview, cancellable refinement | [Performance table](#measured-performance) | Covered |
| Rotation and camera zoom | 10 | `Q`/`E` orbit around the diorama, mouse look, wheel zoom, `W`/`S` approach | [Controls](#controls) | Covered |
| Five different materials | 25 (5 × 5) | Seven listed, each with its own texture, albedo, specular, transparency and reflectivity | [Materials table](#materials-that-change-the-light) | Covered |
| Refraction | 10 | Snell refraction on water (1.33) and glass (1.50) | [Pond](docs/screenshots/readme/06-water-refraction.png), [windows](docs/screenshots/readme/18-house-glass.png) | Covered |
| Reflection | 5 | Mirror rays on water, glass, crying obsidian and amethyst | [Pond](docs/screenshots/readme/06-water-refraction.png) | Covered |
| Normal maps | 10 | Tangent-space normal map on Deepslate Bricks and the three Red-Black bricks | [On / off](#normal-mapping-on-and-off) | Covered |
| Emissive material | 10 | Emissive masks on lamps, the portal core, 8 suit blocks, crying obsidian and amethyst | [Underside](docs/screenshots/readme/11-emissive-materials.png), [portal](docs/screenshots/readme/07-portal.png) | Covered |
| Skybox | 10 | Six-face cubemap per realm, sampled on the CPU by ray direction | [Day](docs/screenshots/readme/16-skybox-day.png), [night](docs/screenshots/readme/17-skybox-night.png) | Covered |
| Procedural terrain (≥ 16 × 16) | 20 | 24 × 24 columns of 4-octave fBM value noise with strata | [Terrain](docs/screenshots/readme/05-procedural-terrain.png) | Covered |

The published criteria add up to 145 points and the assignment's maximum grade is 100, so the
list offers more ways to earn points than are needed.

### Delivery requirements

| Requirement | Evidence | Status / risk |
|---|---|---|
| Delivered on GitHub | This repository | Ready |
| Video of the diorama in the README | [Final Video](#final-video) placeholder | **Pending: the author's video** |
| No libraries external to the language | The only dependency is `raylib`. It opens the window, reads input, decodes PNG files and shows the finished frame. It takes no part in ray tracing. | **Open point:** whether a windowing library is allowed is the instructor's call |
| No use of the video card | Rays, traversal, intersections, shading, shadows, reflection, refraction, normal maps, emission and the skybox are all computed on the CPU. Raylib then uploads the finished image as one texture to display it. | Ray tracing is CPU-only; the final blit to the screen goes through raylib |
| Features visible and used with intent | Every feature above has a place in the scene and a screenshot | Ready |

---

## Architecture & Engineering Decisions

```text
 input ──► camera ──► primary rays ──► 3D DDA ──► intersection ──► shading ──► framebuffer ──► window
            │                            │             │              │
            │                        VoxelWorld    block shape     material, lights,
       free-fly, collision,          (one grid,    (cube or        shadows, reflection,
       portal crossing               two realms)   boxes)          refraction, normal map,
                                                                   emission, skybox
```

The code is split into four layers, each depending only on the ones above it.

| Layer | Contents |
|---|---|
| `core` | `Vec3`, `Ray`, `HitRecord`, `Aabb`, `Cube` and `Prism` intersection, `Color`, `Material`, `CpuTexture`, reflection and refraction maths |
| `scene` | `VoxelWorld`, `BlockType`, block shapes, the material library, lights, the terrain generator and the builders for every structure |
| `camera` | `Camera` and ray projection, the World's free-fly camera, collision, portal crossing, and the catalog's orbit camera |
| `renderer` | The ray tracer, 3D DDA traversal, shading, shadows, normal mapping, emission, the skybox, parallel rendering, adaptive preview, refinement and the `Framebuffer` |

`app.rs` is the only file that talks to raylib: it polls input, asks the renderer for a frame
and presents the framebuffer.

### How a frame is made

1. **Input** moves the free-fly camera; the motion is resolved against solid blocks.
2. **The portal detector** checks whether that motion crossed the portal's membrane.
3. **The parallel renderer** splits the image into row bands and hands them to worker threads.
4. **Each pixel** casts a primary ray, which the 3D DDA walks through `VoxelWorld`.
5. **Each hit** is shaded: texture, lights, shadow rays, then reflection and refraction rays.
6. **The framebuffer** is copied into one texture and drawn to the window.

### Decisions worth explaining

- **Why the CPU.** The assignment is about ray tracing itself. Doing it on the CPU keeps every
  step in plain Rust that can be read and tested: the suite checks single rays and single
  pixels.
- **Why one `VoxelWorld`.** Both realms are the same island. One grid means the portal is a
  real opening, the cutaway shows both halves at once, and light and reflections cross between
  them without special cases.
- **Why a physical portal.** A teleport would hide the trick. Flying through an actual hole
  and watching the world turn over is the point of the scene.
- **Why Red-Black uses `-Y`.** The second realm is the underside of the first, so its ground
  is the island's ceiling. Everything there is built with "up" pointing down: blocks are
  oriented downward, stairs climb toward `-Y`, and the camera's controls follow.
- **Why nearest-neighbor sampling.** The textures are 16 × 16 pixel art. Filtering would blur
  them; nearest-neighbor keeps every texel a clean square.
- **Why emission does not always create lights.** An emissive block glows by itself, but
  turning all of them into light sources would mean a shadow ray per block per pixel. Only
  five real lights exist; the rest is emissive masks, which is what keeps the realm walkable.
- **Why the symbols use masks.** A suit block is one texture with an emissive mask over the
  symbol, so the heart glows while its frame stays dark.
- **Why World and Catalog are separate.** The catalog is a neutral studio for inspecting one
  block at a time; the world is the composed scene. They share the renderer and nothing else.
- **Why collision and noclip coexist.** Collision makes the paths, doors and interiors feel
  like places. Noclip is there for inspection and for recording shots from outside.
- **Why partial shapes are boxes.** Stairs, fences and doors are a handful of axis-aligned
  boxes each, so they reuse the cube intersection and stay exact.

---

## Testing / Validation

```bash
cargo fmt -- --check
cargo check
cargo test --no-fail-fast
cargo check --bin catalog
```

The suite holds 1,931 tests in more than 200 test files. They cover the maths, single-ray
intersections, DDA stepping, material optics, the skybox, each structure of both realms,
collision and full walkthroughs of the route from the house to the fortress's upper room and
back.

The screenshots in this README were rendered by the project's own ray tracer at 1200 × 900 in
Full quality, without the HUD, and were not edited afterwards.

---

## Final Project Status

| | |
|---|---|
| World | Complete: Overworld, portal and Red-Black realm |
| Catalog | 45 of 45 official blocks |
| Tests | 1,931 passing, 0 failing |
| Animated preview | Pending |
| Final video | Pending |

Built with Rust. The only dependency is `raylib`, used for the window, input, PNG decoding and
presenting the finished frame.

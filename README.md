# Red-Black Raytraced World

CPU-based 3D raytracing project written in Rust for the Computer Graphics course at Universidad del Valle de Guatemala.

The project renders a voxel diorama inspired by Minecraft-style environments and the visual identity of Red-Black Maze. The scene is rendered entirely through a custom CPU raytracer.

## Project Goals

- Render a fully 3D voxel scene using raytracing.
- Implement ray generation and cube intersections from scratch.
- Support textured voxel blocks.
- Implement diffuse and specular lighting.
- Implement hard shadows.
- Support reflection and refraction.
- Support emissive materials.
- Support normal mapping.
- Implement a CPU-rendered skybox.
- Generate procedural voxel terrain.
- Provide an interactive orbiting camera with zoom.
- Optimize rendering through spatial traversal and CPU parallelism.

## Architecture

The project is organized into four main layers:

```text
Core
  ↓
Scene
  ↓
Camera
  ↓
Renderer
  ↓
Framebuffer
  ↓
Raylib
  ↓
Screen

```

### Core

Mathematics, rays, intersections, materials, textures and optical calculations.

### Scene

Voxel world, blocks, materials, lights, procedural terrain and scene composition.

### Camera

Projection, field of view, aspect ratio and interactive camera controls.

### Renderer

CPU raytracing, voxel traversal, texture sampling, lighting, shadows, reflection, refraction, emission and framebuffer generation.

## Technology

- Rust
- Cargo
- Raylib

Raylib is used only for infrastructure such as window creation, input, image loading and framebuffer presentation.

The raytracing pipeline, geometry, mathematics, materials, lighting and rendering algorithms are implemented manually on the CPU.

## Status

Project architecture initialized. Core raytracing implementation pending.

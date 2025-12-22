// src/wgpu_render.rs
// Basic window rendering with update and input handling.
mod wgpu_render;

fn main() {
    // Start rendering in new window, begin wait for updates and input.
    wgpu_render::run();
}
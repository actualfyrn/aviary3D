// src/wgpu_render.rs
// Handles base window rendering with update and input handling.
mod wgpu_render;

// src/audio_render.rs
// Handles base audio rendering and handling before playback.
// mod audio_render;

fn main() {
    // Start rendering in new window, begin wait for updates and input.
    wgpu_render::run();
}
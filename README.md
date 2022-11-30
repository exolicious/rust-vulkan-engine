# rust-vulkan-engine

This is a side project to learn Rust and Vulkan in a project context. I have no idea where this will go.

## Libs used
- [vulkano-rs](https://github.com/vulkano-rs): Vulkan API for Rust
- [winit](https://github.com/rust-windowing/winit): "Cross-platform window creation and management in Rust"
- other libraries: see Cargo.toml file


Current stable version 0.03 supports:
- Runtime addition of meshes to vertex buffers (1 for each frame in flight), synching between those buffers using a really simple event system.
- Dynamic uniform buffers + instanced drawing semantics.
- World ticking and graphic updates on transform uniform buffer  

Demo of runtime addition of moving entities with different meshes:

![Demo gif](./demo.gif?raw=true "Demo of version 0.03")

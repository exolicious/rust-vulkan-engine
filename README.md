# rust-vulkan-engine

A small 3D engine written from scratch in Rust on top of raw Vulkan (via
vulkano). Started as a project to learn Rust and the explicit GPU programming
model at the same time, and it has grown into a working renderer with a real
architecture: instanced drawing, correct frames-in-flight synchronization, and
an egui overlay composited in the same render pass.

Run it with `cargo run`. Space spawns cubes, which drift around with a random
walk. An egui window shows the live entity count.

Demo of runtime addition of moving entities:

![Demo gif](./demo.gif?raw=true "Demo")

## Stack

- [vulkano](https://github.com/vulkano-rs/vulkano): type-safe Rust wrapper over Vulkan
- [winit](https://github.com/rust-windowing/winit): window creation and event loop
- [egui_winit_vulkano](https://github.com/hakolao/egui_winit_vulkano): egui integration
- [glam](https://github.com/bitshifter/glam-rs): SIMD math
- rest: see Cargo.toml

## Architecture overview

The codebase is split into two halves that deliberately know very little about
each other:

- `Engine` (src/engine/): game state. Owns the entities and the active scene,
  which owns the camera. No Vulkan types appear anywhere in this half.
- `Renderer` (src/rendering/): everything Vulkan. Owns the swapchain,
  pipeline, buffers, and the per-frame fences.

Communication is one-directional through an event queue
(`VecDeque<EngineEvent>` in `Engine`). The engine pushes events during
simulation; once per frame `Engine::work_off_event_queue` drains the queue in
FIFO order into renderer handler methods:

| Event | Meaning | Renderer reaction |
|---|---|---|
| `EntityAdded(transform, mesh, index)` | new entity exists | register mesh instance, push transform |
| `EntitiesUpdated(vec)` | entities moved this tick | overwrite CPU-side model matrices |
| `ChangedActiveScene(scene)` | scene switch | store the `Arc<Scene>` (camera source) |

FIFO ordering is a correctness requirement, not a style choice: an
`EntityAdded` has to be processed before an `EntitiesUpdated` that references
the new entity's index.

Entity identity is currently just the index into `Engine::entities`. There is
no entity deletion yet; adding it would invalidate plain indices, so that
feature brings generational ids or a free list with it. This is the biggest
assumption baked into everything below.

## Frame flow (main.rs)

winit event loop, redraw requested on `MainEventsCleared`. Each frame runs
these steps in order:

1. `engine.tick()`: every entity gets `tick()`, movers report
   `TickAction::HasMoved`, and the engine batches all of them into a single
   `EntitiesUpdated` event.
2. `renderer.begin_frame()`: acquires the next swapchain image and waits on
   that image's fence (see the synchronization section). Returns `None` when
   the swapchain is stale or the window is minimized, in which case the frame
   is simply skipped.
3. `engine.work_off_event_queue(&mut renderer)`: drains the event queue.
4. egui: `gui.immediate_ui(...)` builds the UI, `gui.draw_on_subpass_image`
   returns a secondary command buffer for subpass 1.
5. `renderer.end_frame(...)`: uploads frame data, records the primary command
   buffer, submits, presents, and stores the new fence.

Resizing is handled lazily. The `Resized` event only sets a flag
(`mark_swapchain_outdated`); the actual recreation happens at the top of the
next `begin_frame`. Suboptimal or OutOfDate results from acquire/present set
the same flag. Recreation rebuilds the swapchain, framebuffers, per-image
buffers (if the image count changed), and the pipeline, since the viewport is
baked into it.

## Frames in flight and per-image buffers

The swapchain has N images (usually 3). While image A is being scanned out,
the CPU is already recording commands that read GPU buffers for image B. With
a single uniform/storage buffer, writing the next frame's data would race
against the GPU reading the previous frame's.

The scheme used here: N copies of every per-frame buffer (transforms,
view-projection), indexed by swapchain image index, plus one fence-backed
frame future per image (`Renderer::frame_futures`). Before touching image i's
buffers, `begin_frame` waits on frame future i, at which point the GPU is
guaranteed to be done with everything that frame submitted. `end_frame` chains
the previous frame's future into the new submission
(`previous_future.join(acquire_future)...then_signal_fence_and_flush`) and
stores the result in `frame_futures[i]`.

The one exception is the vertex buffer, which is shared across all images
(mesh data is static, so N copies would buy nothing). It is only written when
a new mesh type first appears, and in that case `end_frame` waits for all
in-flight frames first, accepting a full pipeline stall for a rare event.

## Per-frame data upload

There is no delta or cross-buffer sync machinery. The renderer keeps one
CPU-side authoritative array of model matrices
(`TransformBuffers::model_matrices`, indexed by entity) and writes all of them
into the acquired image's storage buffer every frame. The camera works the
same way: the active scene's projection-view matrix is written into that
image's uniform buffer each frame. At realistic entity counts for this project
the cost is negligible; dirty tracking can be added if profiling ever says so.

An earlier version wrote deltas into one image's buffer and then GPU-copied
the changes to the other images' buffers. It was clever and it was broken.
Full re-upload is simpler and correct, which is the right trade at this scale.

## Instanced drawing

One vertex buffer holds each distinct mesh's vertices exactly once. Entities
sharing a mesh become instances of a single draw call.

`MeshAccessor` is the bookkeeper. Meshes are identified by name (`"cube"`).
Per mesh it stores a `MeshGroup { mesh, first_vertex, entity_indices }`:

- `first_vertex`: offset of this mesh's vertices in the shared vertex buffer,
  cumulative across previously added meshes.
- `entity_indices`: which entities are instances of this mesh.

Draw submission (`Renderer::build_command_buffer`) walks the groups:

```
draw(vertex_count, instance_count, first_vertex, first_instance)
```

with `first_instance` accumulating across groups. The key Vulkan detail:
`gl_InstanceIndex` in the shader starts at `first_instance`, unlike OpenGL's
`gl_InstanceID`. Each group therefore indexes its own contiguous slice of the
transform storage buffer, which is why `TransformBuffers::upload` writes
matrices in `MeshAccessor::instance_order()` (grouped by mesh) rather than in
entity-index order. If the upload order and the draw order ever disagree,
entities render with each other's transforms. That invariant lives in exactly
two places, `instance_order()` and the `first_instance` accumulation loop, and
they have to change together.

Vertex shader (`src/rendering/shaders.rs`, compiled at build time by
`vulkano_shaders::shader!`):

```glsl
gl_Position = view_projection * model[gl_InstanceIndex] * vec4(position, 1.0);
```

Transforms live in a storage buffer rather than a uniform buffer: UBOs
commonly cap at 64KB (`maxUniformBufferRange`), which is only about 1000
mat4s, and SSBOs also allow an unsized array in GLSL. The view-projection
matrix stays a regular UBO since it is a single 64-byte matrix.

## BufferManager

`src/rendering/buffer_manager.rs` owns GPU memory and the things allocated
from it, nothing else:

- the three allocators (memory, descriptor set, command buffer), used by the
  renderer during recording
- `VertexBuffer`: the shared vertex buffer, the `MeshAccessor`, and a
  `pending_uploads` list. New mesh data is not written immediately on
  registration (the GPU might be reading the buffer); it is queued and flushed
  in `end_frame` after the all-fences wait described above.
- `TransformBuffers`: N storage buffers plus the CPU matrix array
- `vp_buffers`: N one-matrix uniform buffers
- `frames`: one `Frame` (image view + framebuffer) per swapchain image

`upload_frame_data(image_index, vp)` is the single "everything the GPU needs
for this frame" write: flush pending vertices, upload transforms, write the
view-projection matrix.

Command buffer recording lives in `Renderer`, not here. BufferManager hands
out buffers; it does not decide what to do with them.

Capacities are fixed and checked with a readable error instead of a slice
panic: 65536 vertices, 4096 instances. Growing a buffer at runtime would mean
allocating a bigger one, waiting on all fences, and rebinding. Not built, not
yet needed.

## Render pass and egui

One render pass, two subpasses (`ordered_passes_renderpass!` in renderer.rs):

- subpass 0: the scene (instanced draws, blue clear color)
- subpass 1: egui, via `egui_winit_vulkano`, executed as a secondary command
  buffer (`SubpassContents::SecondaryCommandBuffers`)

The UI is composited in the same pass: no extra image, no extra
synchronization. `Gui::new_with_subpass` receives subpass 1 and the swapchain
format.

Two egui integration details worth knowing:

- egui blends in linear space and expects a UNORM render target. Drivers tend
  to list sRGB surface formats first, so swapchain creation explicitly
  prefers `B8G8R8A8_UNORM`/`R8G8B8A8_UNORM`. `allow_srgb_render_target: true`
  is set as a fallback so platforms without a UNORM format don't panic, at
  the cost of slightly-off UI colors there.
- Window and popup shadows are disabled globally via the egui style in
  main.rs; drop shadows over a 3D viewport look wrong.

Input routing: `gui.update(&event)` returns true when egui consumed the
event, and the spacebar handler checks that flag so interacting with a panel
doesn't also spawn cubes.

## Coordinates and camera

- glam throughout. Left-handed projection, `Mat4::perspective_lh(fov_y, ...)`
  where the FOV argument is radians (`55f32.to_radians()`). Passing degrees
  compiles fine and renders a kaleidoscope.
- The aspect ratio is hardcoded to 16:9. Resizing updates the viewport but
  not the projection, so non-16:9 windows stretch. Fixing it means routing
  resize events to the scene's camera, which currently sits behind an
  immutable `Arc`. Known cut corner.
- `Transform` = translation + rotation (quaternion) + scale. `Default` is
  identity with scale ONE. A derived (zeroed) default scale collapses every
  mesh to a point, which is why the impl is manual.
- View matrix = inverse of the camera's model matrix. The combined
  `projection_view_matrix` is cached on the camera and recomputed on move.
- The camera sits at z = -5 looking toward +z. Depth range 1..4000, but there
  is no depth buffer yet: face order is luck plus the absence of backface
  culling (`RasterizationState::default()`). A depth attachment is the next
  real rendering feature on the list.

## File map

```
src/
  main.rs                      event loop, egui setup, frame orchestration
  engine/
    engine.rs                  Engine: entities, event queue, tick
    scene.rs                   Scene: owns the Camera
    camera.rs                  view/projection math
    general_traits.rs          Entity trait (tick / transform / mesh) + TickAction
  physics/
    physics_traits.rs          Transform + model_matrix
  rendering/
    renderer.rs                device/swapchain/pipeline setup, begin/end_frame,
                               fences, command buffer recording, event handlers
    buffer_manager.rs          owns all GPU buffers + allocators
    transform_buffers.rs       per-image SSBOs + CPU matrix array
    vertex_buffers.rs          shared vertex buffer + deferred mesh uploads
    mesh_accessor.rs           mesh dedup, instance grouping, draw-call layout
    frame.rs                   per-swapchain-image view + framebuffer
    shaders.rs                 GLSL, compiled at build time
    primitives.rs              Vertex, Mesh, Cube (corner table + index list)
  initialize/
    vulkan_instancing.rs       instance creation
```

## Platform notes

`.cargo/config.toml` forces `WINIT_UNIX_BACKEND=x11` because the Wayland
backend dies under WSLg the moment the window gains focus. The details,
including why the process exits without any error output, are documented in
that file. Harmless on platforms where winit has no Unix backend choice.

## Troubleshooting

- Nothing renders: check that the FOV is in radians, that
  `Transform::default()` has scale ONE, and the camera z sign.
- Entities have each other's transforms: the `instance_order` /
  `first_instance` invariant from the instancing section is broken.
- Panic or error on buffer `write()`: something is writing a buffer the GPU
  is still reading; a fence wait is missing or waits on the wrong image
  index.
- Washed-out or off UI colors: the swapchain picked an sRGB format (see the
  egui section).
- vulkano's runtime checks catch a lot, but they are not a substitute for the
  Khronos validation layer; test new synchronization code on a machine that
  has it installed.

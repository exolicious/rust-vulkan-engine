use std::time::Instant;

use glam::Vec3;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use engine::Engine;
use engine::scene::Scene;
use input::InputManager;
use rendering::gui::GuiOverlay;
use rendering::renderer::Renderer;

use crate::gui_state::GuiState;

pub mod engine;
pub mod gui_state;
pub mod initialize;
pub mod input;
pub mod physics;
pub mod rendering;

const CAMERA_SPEED: f32 = 5.0;
const SENSITIVITY: f32 = 0.5;

/// Tracks the time between frames, exponentially smoothed so the displayed
/// numbers are stable enough to read.
struct FrameTimer {
    last_frame: Instant,
    smoothed_frame_time: f32,
}

impl FrameTimer {
    const SMOOTHING: f32 = 0.05;

    fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            smoothed_frame_time: 0.0,
        }
    }

    /// Call once per frame; returns (frame time in ms, fps).
    fn tick(&mut self) -> (f32, f32) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.smoothed_frame_time = if self.smoothed_frame_time == 0.0 {
            frame_time
        } else {
            self.smoothed_frame_time + (frame_time - self.smoothed_frame_time) * Self::SMOOTHING
        };
        let fps = if self.smoothed_frame_time > 0.0 {
            1.0 / self.smoothed_frame_time
        } else {
            0.0
        };
        (self.smoothed_frame_time * 1000.0, fps)
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let mut engine = Engine::new();
    let mut renderer = Renderer::new(&event_loop);
    let mut gui_state = GuiState::new();

    engine.set_active_scene(Scene::new());

    let mut gui = GuiOverlay::new(&event_loop, &renderer);
    let mut frame_timer = FrameTimer::new();
    let mut input = InputManager::new();

    event_loop.run(move |event, _, control_flow|
        match event {
        Event::WindowEvent { event, .. } => {
            // The gui gets first refusal on every window event; whatever it
            // does not claim belongs to the engine.
            let gui_consumed = gui.handle_event(&event);
            input.set_gui_capture(gui_consumed, gui.pointer_over_gui());

            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    renderer.mark_swapchain_outdated()
                }
                other => input.handle_window_event(&other),
            }
        }
        // Raw mouse motion for mouse look, unaffected by the cursor stopping
        // at the edge of the screen.
        Event::DeviceEvent { event, .. } => input.handle_device_event(&event),
        Event::MainEventsCleared => renderer.window().request_redraw(),
        Event::RedrawRequested(_) => {
            let (frame_time_ms, fps) = frame_timer.tick();
            let delta_seconds = frame_time_ms / 1000.0;

            // While the pointer is over the gui it belongs to the gui: the
            // cursor stays visible and the camera stops following the mouse.
            let pointer_over_gui = gui.pointer_over_gui();
            apply_input(&input, &mut engine, delta_seconds, pointer_over_gui);
            renderer.window().set_cursor_visible(pointer_over_gui);

            engine.tick(frame_time_ms);

            let Some((image_index, acquire_future)) = renderer.begin_frame() else {
                input.end_frame();
                return;
            };

            engine.work_off_event_queue(&mut renderer);

            let gui_command_buffer = gui.draw(renderer.swapchain_extent(), |ctx| {
                gui_state.build_ui(ctx, &mut engine, frame_time_ms, fps);
            });

            renderer.end_frame(image_index, acquire_future, gui_command_buffer, engine.active_scene());

            input.end_frame();
        }
        _ => {}
    });
}

fn apply_input(
    input: &InputManager,
    engine: &mut Engine,
    delta_seconds: f32,
    pointer_over_gui: bool,
) {
    if input.was_key_pressed(VirtualKeyCode::Space) {
        engine.add_cube_to_scene(None);
        engine.add_multiple_cubes_to_scene(2000);
    }

    let horizontal = input.axis(VirtualKeyCode::D, VirtualKeyCode::A);
    let forward = input.axis(VirtualKeyCode::S, VirtualKeyCode::W);

    let Some(scene) = engine.active_scene_mut() else {
        return;
    };

    // Camera-local: +X right, +Z forward (the projection is left handed).
    let movement = Vec3::new(horizontal, 0.0, -forward).normalize_or_zero()
        * CAMERA_SPEED
        * delta_seconds;

    // The rotation maps camera-local to world, so it turns the key axes into
    // the direction the camera is actually facing.
    let world_movement = scene.camera.transform.rotation * movement;

    scene.camera.translate(world_movement);

    // Raw mouse motion is never withheld by the input manager, so the gui
    // check happens here.
    let (mouse_x, mouse_y) = input.mouse_delta();
    if !pointer_over_gui && (mouse_x != 0.0 || mouse_y != 0.0) {
        scene.camera.rotate(mouse_x * SENSITIVITY, mouse_y * SENSITIVITY);
    }
}

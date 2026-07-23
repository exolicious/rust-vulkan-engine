use std::time::Instant;

use glam::Vec3;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use engine::Engine;
use engine::scene::Scene;
use rendering::gui::GuiOverlay;
use rendering::renderer::Renderer;

use crate::gui_state::GuiState;

pub mod engine;
pub mod gui_state;
pub mod initialize;
pub mod physics;
pub mod rendering;

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
    //engine.add_cube_to_scene(Some(Vec3::new(1., 1., 2.)), None);
    

    let mut gui = GuiOverlay::new(&event_loop, &renderer);
    let mut frame_timer = FrameTimer::new();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            let gui_consumed = gui.handle_event(&event);
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(_) => renderer.mark_swapchain_outdated(),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } if !gui_consumed => match key {
                    VirtualKeyCode::Space => {
                        engine.add_cube_to_scene(None);
                        engine.add_multiple_cubes_to_scene(2000);
                    }
                    VirtualKeyCode::W => {
                        engine.active_scene_mut().unwrap().camera.translate(Vec3 { x: 0., y: 0., z: 1. });
                    }
                    VirtualKeyCode::A => {
                        engine.active_scene_mut().unwrap().camera.translate(Vec3 { x: -1., y: 0., z: 0. });
                    }
                    VirtualKeyCode::S => {
                        engine.active_scene_mut().unwrap().camera.translate(Vec3 { x: 0., y: 0., z: -1. });
                    }
                    VirtualKeyCode::D => {
                        engine.active_scene_mut().unwrap().camera.translate(Vec3 { x: 1., y: 0., z: 0. });
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Event::MainEventsCleared => renderer.window().request_redraw(),
        Event::RedrawRequested(_) => {
            let (frame_time_ms, fps) = frame_timer.tick();
            engine.tick();

            let Some((image_index, acquire_future)) = renderer.begin_frame() else {
                return;
            };

            engine.work_off_event_queue(&mut renderer);

            let gui_command_buffer = gui.draw(renderer.swapchain_extent(), |ctx| {
                gui_state.build_ui(ctx, &mut engine, frame_time_ms, fps);
            });

            renderer.end_frame(image_index, acquire_future, gui_command_buffer, engine.active_scene());
        }
        _ => {}
    });
}

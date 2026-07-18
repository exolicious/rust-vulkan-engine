use std::sync::Arc;

use egui_winit_vulkano::{egui, Gui, GuiConfig};
use glam::Vec3;
use vulkano::render_pass::Subpass;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use engine::engine::Engine;
use engine::scene::Scene;
use rendering::renderer::Renderer;

pub mod engine;
pub mod initialize;
pub mod physics;
pub mod rendering;

fn main() {
    let event_loop = EventLoop::new();

    let mut engine = Engine::new();
    let mut renderer = Renderer::new(&event_loop);

    engine.set_active_scene(Arc::new(Scene::new()));
    engine.add_cube_to_scene(Some(Vec3::new(1., 1., 2.)));

    let gui_subpass = Subpass::from(renderer.render_pass.clone(), 1).unwrap();
    let mut gui = Gui::new_with_subpass(
        &event_loop,
        renderer.surface.clone(),
        renderer.queue.clone(),
        gui_subpass,
        renderer.swapchain.image_format(),
        GuiConfig {
            allow_srgb_render_target: true,
            ..GuiConfig::default()
        },
    );

    let gui_context = gui.context();
    let mut gui_style = (*gui_context.style()).clone();
    gui_style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    gui_style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    gui_context.set_style(gui_style);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            let gui_consumed = gui.update(&event);
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(_) => renderer.mark_swapchain_outdated(),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                    ..
                } if !gui_consumed => {
                    engine.add_cube_to_scene(None);
                }
                _ => {}
            }
        }
        Event::MainEventsCleared => renderer.window().request_redraw(),
        Event::RedrawRequested(_) => {
            engine.tick();

            let Some((image_index, acquire_future)) = renderer.begin_frame() else {
                return;
            };

            engine.work_off_event_queue(&mut renderer);

            gui.immediate_ui(|gui| {
                let ctx = gui.context();
                egui::Window::new("Engine").show(&ctx, |ui| {
                    ui.label(format!("entities: {}", engine.entity_count()));
                    ui.label("space: spawn cubes");
                });
            });
            let gui_command_buffer = gui.draw_on_subpass_image(renderer.swapchain_extent());

            renderer.end_frame(image_index, acquire_future, gui_command_buffer);
        }
        _ => {}
    });
}

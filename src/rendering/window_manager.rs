use cgmath::Vector3;
use egui_winit_vulkano::{
    egui::{self, Context, RawInput, Vec2},
    Gui,
};
use rand::Rng;
use std::sync::Arc;
use vulkano::{
    swapchain,
    sync::{self, future::FenceSignalFuture, GpuFuture}, Validated, VulkanError,
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key},
    window::WindowBuilder,
};

use crate::engine::engine::Engine;

use super::renderer::{EventResolveTiming, RendererEvent};

pub struct WindowManager {
    pub engine: Engine,
    pub event_loop: EventLoop<()>,
}

impl WindowManager {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let window = WindowBuilder::new()
            .with_title("egui with winit")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(&event_loop)
            .unwrap();
        let engine = Engine::new(&event_loop, window);
        

        Self { engine, event_loop }
    }

    pub fn start_engine(mut self) -> () {
        //init scene
        self.engine.add_cube_to_scene(None);

        // Create egui context
        let mut egui_ctx = Context::default();

        //start event loop
        let frames_in_flight = self.engine.renderer.swapchain_images.len();
        let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let mut previous_fence_i = 0;

        //let mut gui = Gui::new(&self.event_loop, self.engine.renderer.surface.clone(), None, self.engine.renderer.active_queue.clone(), false);

        self.event_loop.run(move |event, control_flow| {
            match event {
                Event::AboutToWait => {
                    self.engine.tick();
                }
                Event::WindowEvent { event, .. } => {
                    //let pass_events_to_game = !gui.update(&event); // if this returns false, then egui wont have to handle the request and we can pass it to the game
                    //if pass_events_to_game {
                        match event {
                            WindowEvent::Resized(_) => {
                                self.engine
                                    .renderer
                                    .receive_event(EventResolveTiming::Immediate(
                                        RendererEvent::WindowResized,
                                    ))
                            }
                            WindowEvent::CloseRequested => control_flow.exit(),
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        logical_key: key,
                                        state: ElementState::Pressed,
                                        ..
                                    },
                                ..
                            } => match key.as_ref() {
                                Key::Character("w") => {
                                    //todo:: here we should shoot an event up to our event/input handler who holds a reference to the currently selected controller(?)
                                }
                                Key::Character(" ") => {
                                    for _ in 0..100 {
                                        let rand_x: f32 =
                                            rand::thread_rng().gen_range(-2_f32..2_f32);
                                        let rand_y: f32 =
                                            rand::thread_rng().gen_range(-2_f32..2_f32);
                                        let rand_z: f32 =
                                            rand::thread_rng().gen_range(-7_f32..-2_f32);
                                        self.engine.add_cube_to_scene(Some(Vector3 {
                                            x: rand_x,
                                            y: rand_y,
                                            z: rand_z,
                                        }));
                                    }
                                }
                                _ => (), //todo:: here we should shoot an event up to our event/input handler who holds a reference to the currently selected controller(?)
                            },
                            WindowEvent::RedrawRequested => {
                                egui_ctx.begin_frame(RawInput::default());

                                // Draw your UI here
                                egui::CentralPanel::default().show(&egui_ctx, |ui| {
                                    ui.label("Hello, egui!");
                                });

                                let output = egui_ctx.end_frame();
                                let paint_jobs = egui_ctx.tessellate(output.shapes);

                                // You should provide your own rendering code here
                                // For simplicity, let's just print out the paint jobs
                                for primitive in paint_jobs {
                                    println!("Render mesh: {:?}", primitive);
                                }

                                let (swapchain_image_index, suboptimal, acquire_future) =
                                    match swapchain::acquire_next_image(
                                        self.engine.renderer.swapchain.clone(),
                                        None,
                                    ).map_err(Validated::unwrap) {
                                        Ok(r) => r,
                                        Err(VulkanError::OutOfDate) => {
                                            self.engine.renderer.receive_event(
                                                EventResolveTiming::Immediate(
                                                    RendererEvent::RecreateSwapchain,
                                                ),
                                            );
                                            return;
                                        }
                                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                                    };

                                if suboptimal {
                                    self.engine.renderer.receive_event(
                                        EventResolveTiming::Immediate(
                                            RendererEvent::RecreateSwapchain,
                                        ),
                                    );
                                }

                                // wait for the fence related to this image to finish (normally this would be the oldest fence)
                                if let Some(image_fence) = &fences[swapchain_image_index as usize] {
                                    image_fence.wait(None).unwrap();
                                    self.engine.next_swapchain_image_index =
                                        swapchain_image_index as usize;
                                    self.engine.update_graphics();
                                }

                                let previous_future = match fences[previous_fence_i].clone() {
                                    None => {
                                        let mut now =
                                            sync::now(self.engine.renderer.device.clone());
                                        now.cleanup_finished();
                                        now.boxed()
                                    }
                                    Some(fence) => fence.boxed(),
                                };
                                let future = self.engine.renderer.get_future(
                                    previous_future,
                                    acquire_future,
                                    swapchain_image_index as usize
                                );

                                fences[swapchain_image_index as usize] = match future {
                                    Ok(value) => Some(Arc::new(value)),
                                    Err(VulkanError::OutOfDate) => {
                                        self.engine.renderer.receive_event(
                                            EventResolveTiming::Immediate(
                                                RendererEvent::RecreateSwapchain,
                                            ),
                                        );
                                        None
                                    }
                                    Err(e) => {
                                        println!("Failed to flush future: {:?}", e);
                                        None
                                    }
                                };
                                previous_fence_i = swapchain_image_index as usize;
                            }
                            _ => (),
                        }
                    //}
                }
                Event::NewEvents(_) => todo!(),
                Event::DeviceEvent { device_id, event } => todo!(),
                Event::UserEvent(_) => todo!(),
                Event::Suspended => todo!(),
                Event::Resumed => todo!(),
                Event::LoopExiting => todo!(),
                Event::MemoryWarning => todo!(),
            }
        });
    }

    fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
        ui.label(egui::RichText::new(text).size(size));
    }
}

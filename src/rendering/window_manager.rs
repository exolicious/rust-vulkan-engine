use std::{sync::Arc};
use egui_winit_vulkano::egui::{self, ScrollArea, TextEdit, TextStyle};
use rand::Rng;
use cgmath::Vector3;
use vulkano::{sync::{FenceSignalFuture, GpuFuture, self, FlushError}, swapchain::{self, AcquireError}};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, platform::run_return::EventLoopExtRunReturn};

use crate::engine::engine::Engine;

use super::renderer::{RendererEvent, EventResolveTiming};
 
pub struct WindowManager {
    pub engine: Engine,
    pub event_loop: EventLoop<()>,
}


const CODE: &str = r#"
# Some markup
```
let mut gui = Gui::new(&event_loop, renderer.surface(), None, renderer.queue());
```
"#;

impl WindowManager {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let engine = Engine::new(&event_loop);

        Self {
            engine,
            event_loop,
        }
    }

    pub fn start_engine(mut self) -> () {
        //init scene
        self.engine.add_cube_to_scene(None);
       
        //start event loop
        let frames_in_flight = self.engine.renderer.swapchain_images.len();
        let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let mut previous_fence_i = 0;

        let mut code = CODE.to_owned();

        self.event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event,
                ..
            } => {
                let pass_events_to_game = !self.engine.gui.update(&event); // if this returns false, then egui wont have to handle the request and we can pass it to the game
                
                if pass_events_to_game {
                    match event {
                        WindowEvent::Resized(_) => self.engine.renderer.receive_event(EventResolveTiming::Immediate(RendererEvent::WindowResized)),
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                            if input.scancode == 17 {
                                //todo:: here we should shoot an event up to our event/input handler who holds a reference to the currently selected controller(?)
                                
                            }
                            if input.scancode == 57 {
                                match input.state {
                                    winit::event::ElementState::Pressed => {
                                        for _ in 0..100 {
                                            let rand_x: f32 = rand::thread_rng().gen_range(-2_f32..2_f32);
                                            let rand_y: f32 = rand::thread_rng().gen_range(-2_f32..2_f32);
                                            let rand_z: f32 = rand::thread_rng().gen_range(-7_f32..-2_f32);
                                            self.engine.add_cube_to_scene(Some(Vector3{ x: rand_x, y: rand_y, z: rand_z }));
                                        }
                                    }
                                    _ => ()
                                }
                                //todo:: here we should shoot an event up to our event/input handler who holds a reference to the currently selected controller(?)
                            }
                        }
                        _ => ()
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window_id => {
                // Set immediate UI in redraw here
                self.engine.gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    egui::CentralPanel::default().show(&ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add(egui::widgets::Label::new("Hi there!"));
                            Self::sized_text(ui, "Rich Text", 32.0);
                        });
                        ui.separator();
                        ui.columns(2, |columns| {
                            ScrollArea::vertical().id_source("source").show(
                                &mut columns[0],
                                |ui| {
                                    ui.add(
                                        TextEdit::multiline(&mut code).font(TextStyle::Monospace),
                                    );
                                },
                            );
                        });
                    });
                });
            }
            Event::MainEventsCleared => {
                let (swapchain_image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(self.engine.renderer.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            self.engine.renderer.receive_event(EventResolveTiming::Immediate(RendererEvent::RecreateSwapchain));
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    self.engine.renderer.receive_event(EventResolveTiming::Immediate(RendererEvent::RecreateSwapchain));
                }

                // wait for the fence related to this image to finish (normally this would be the oldest fence)
                if let Some(image_fence) = &fences[swapchain_image_index as usize] {
                    image_fence.wait(None).unwrap();
                    self.engine.next_swapchain_image_index = swapchain_image_index as usize;
                    self.engine.update_graphics();
                } 

                let previous_future = match fences[previous_fence_i].clone() {
                    None => {
                        let mut now = sync::now(self.engine.renderer.device.clone());
                        now.cleanup_finished();
                        now.boxed()
                    }
                    Some(fence) => fence.boxed(),
                };
                let future = self.engine.renderer.get_future(previous_future, acquire_future, swapchain_image_index as usize, &mut self.engine.gui);

                fences[swapchain_image_index as usize] = match future {
                    Ok(value) => {
                        Some(Arc::new(value))
                    }
                    Err(FlushError::OutOfDate) => {
                        self.engine.renderer.receive_event(EventResolveTiming::Immediate(RendererEvent::RecreateSwapchain));
                        None
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        None
                    }
                };
                previous_fence_i = swapchain_image_index as usize;
            }
            _ => self.engine.update_engine(),
        });
    }

    fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
        ui.label(egui::RichText::new(text).size(size));
    }
}


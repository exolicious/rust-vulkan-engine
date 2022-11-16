use std::{sync::Arc};

use vulkano::{sync::{FenceSignalFuture, GpuFuture, self, FlushError}, swapchain::{self, AcquireError}};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}};

use crate::engine::engine::Engine;

use super::renderer::RendererEvent;

pub struct WindowManager {
    pub engine: Engine,
    pub event_loop: EventLoop<()>,
/*     previous_fence_i: usize,
    fences: Vec<Option<Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>, Arc<PrimaryAutoCommandBuffer>>, Window>>>>>,
    window_resized: bool,
    recreate_swapchain: bool, */
}

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
        let frames_in_flight = self.engine.renderer.swapchain_images.len();
        let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let mut previous_fence_i = 0;

        self.event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                self.engine.renderer.receive_event(RendererEvent::WindowResized);
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { device_id, input, is_synthetic },
                ..
            } => {
                if input.scancode == 17 {
                    //todo:: here we should shoot an event up to our event/input handler who holds a reference to the currently selected controller(?)
                    self.engine.update();
                }
                if input.scancode == 57 {
                    match input.state {
                        winit::event::ElementState::Pressed => {
                            println!("added cube");
                            self.engine.add_cube_to_scene();
                        }
                        _ => ()
                    }
                    
                    //todo:: here we should shoot an event up to our event/input handler who holds a reference to the currently selected controller(?)
                   
                }
            }
            Event::MainEventsCleared => {
                let (swapchain_image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(self.engine.renderer.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            self.engine.renderer.receive_event(RendererEvent::RecreateSwapchain);
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    self.engine.renderer.receive_event(RendererEvent::RecreateSwapchain);
                }

                // wait for the fence related to this image to finish (normally this would be the oldest fence)
                if let Some(image_fence) = &fences[swapchain_image_index] {
                    image_fence.wait(None).unwrap();
                    self.engine.latest_swapchain_image_index = swapchain_image_index;
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

                let future = self.engine.renderer.get_future(previous_future, acquire_future, swapchain_image_index);

                fences[swapchain_image_index] = match future {
                    Ok(value) => {
                        Some(Arc::new(value))
                    }
                    Err(FlushError::OutOfDate) => {
                        self.engine.renderer.receive_event(RendererEvent::RecreateSwapchain);
                        None
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        None
                    }
                };
                previous_fence_i = swapchain_image_index;
            }
            _ => (),
        });
    }
}
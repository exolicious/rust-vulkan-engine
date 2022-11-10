use std::{sync::Arc};

use vulkano::{sync::{FenceSignalFuture, GpuFuture, self, FlushError, JoinFuture}, swapchain::{self, AcquireError, PresentFuture, SwapchainAcquireFuture, PresentInfo, Surface}, command_buffer::{CommandBufferExecFuture, PrimaryAutoCommandBuffer}, instance::Instance};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::Window};

use crate::{Renderer};

pub struct RenderManager<T> {
    pub renderer: Renderer<T>,
    event_loop: EventLoop<()>,
/*     previous_fence_i: usize,
    fences: Vec<Option<Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>, Arc<PrimaryAutoCommandBuffer>>, Window>>>>>,
    window_resized: bool,
    recreate_swapchain: bool, */
}

impl RenderManager<Surface<Window>> {
    pub fn new() -> RenderManager<Surface<Window>>{
        let event_loop = EventLoop::new();
        let mut renderer = Renderer::new(&event_loop);
        Self {
            renderer,
            event_loop
        }
    }

    pub fn start_renderer(mut self) -> () {
        let mut window_resized = false;
        let mut recreate_swapchain = false;

        let frames_in_flight = self.renderer.swapchain_images.len();
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
                window_resized = true;
            }
            Event::MainEventsCleared => {
                if window_resized || recreate_swapchain {
                    recreate_swapchain = false;

                    self.renderer.recreate_swapchain_and_framebuffers();

                    if window_resized {
                        window_resized = false;
                        self.renderer.recreate_pipeline_and_commandbuffers();
                    }
                }

                let (image_i, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(self.renderer.swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                // wait for the fence related to this image to finish (normally this would be the oldest fence)
                if let Some(image_fence) = &fences[image_i] {
                    image_fence.wait(None).unwrap();
                }

                let previous_future = match fences[previous_fence_i].clone() {
                    // Create a NowFuture
                    None => {
                        let mut now = sync::now(self.renderer.device.clone());
                        now.cleanup_finished();

                        now.boxed()
                    }
                    // Use the existing FenceSignalFuture
                    Some(fence) => fence.boxed(),
                };

                let future = previous_future
                    .join(acquire_future)
                    .then_execute(self.renderer.active_queue.clone(), self.renderer.command_buffers.as_ref().unwrap()[image_i].clone())
                    .unwrap()
                    .then_swapchain_present(
                        self.renderer.active_queue.clone(),
                        PresentInfo {
                            index: image_i,
                            ..PresentInfo::swapchain(self.renderer.swapchain.clone())
                        },
                    )
                    .then_signal_fence_and_flush();

                fences[image_i] = match future {
                    Ok(value) => Some(Arc::new(value)),
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        None
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        None
                    }
                };

                previous_fence_i = image_i;
            }
            _ => (),
        }); 
    }
}
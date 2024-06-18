use egui_winit_vulkano::egui::{CentralPanel, Context, RawInput};
use engine::{engine::Engine, scene::Scene};
use glam::Vec3;
use physics::physics_traits::Transform;
use rendering::{renderer::Renderer};
use vulkano::{swapchain, sync::{self, future::FenceSignalFuture, GpuFuture}, Validated, VulkanError};
use winit::{event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}};

pub mod initialize;
pub mod rendering;
pub mod physics;
pub mod engine;

use std::{env, sync::Arc};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let event_loop = EventLoop::new();
    //event_loop.set_control_flow(ControlFlow::Poll);

    //let window = WindowBuilder::new()
    //    .with_title("egui with winit")
    //    .with_inner_size(LogicalSize::new(800, 600))
    //    .build(&event_loop)
    //    .unwrap();
    let mut engine = Engine::new();
    let renderer = Renderer::new(&event_loop);

    let scene_1 = Arc::new(Scene::new());
    
    engine.set_active_scene(scene_1.clone());
    let translation1 = Some(Vec3{x: 1., y: 1., z: 2.});
    engine.add_cube_to_scene(translation1);
    //let translation2 = Some(Vec3{x: -2., y: -1., z: 5.});
    //engine.add_cube_to_scene(translation2);
    //let translation3 = Some(Vec3{x: -4., y: 4., z: 2.});
    //engine.add_cube_to_scene(translation3);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);
    //engine.add_cube_to_scene(None);


    start_engine(event_loop, engine, renderer);
}

fn start_engine(event_loop: EventLoop<()>, mut engine: Engine, mut renderer: Renderer) -> () {
    //init scene
    // Create egui context
    let mut egui_ctx = Context::default();

    //start event loop
    let frames_in_flight = renderer.buffer_manager.frames.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;
    let mut window_resized = false;
    let mut recreate_swapchain = false;

    //let mut gui = Gui::new(&self.event_loop, self.engine.renderer.surface.clone(), None, self.engine.renderer.active_queue.clone(), false);

    event_loop.run(move |event, _,  control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
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
                    
                    //self.renderer.recreate_swapchain(); //this recreates the framebuffers as a sideeffect
                    if window_resized {
                        renderer.recreate_pipeline();
                    }
                }

                egui_ctx.begin_frame(RawInput::default());

                //// Draw your UI here
                //CentralPanel::default().show(&egui_ctx, |ui| {
                //    ui.label("Hello, egui!");
                //});
//
                //let output = egui_ctx.end_frame();
                //let paint_jobs = egui_ctx.tessellate(output.shapes);
//
                //// You should provide your own rendering code here
                //// For simplicity, let's just print out the paint jobs
                //for primitive in paint_jobs {
                //    println!("Render mesh: {:?}", primitive);
                //}

                println!("Trying to acquire swapchain image!");

                let (swapchain_image_index, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(
                        renderer.swapchain.clone(),
                        None,
                    ).map_err(Validated::unwrap) {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                println!("swapchain_image_index: {}", swapchain_image_index);

                // wait for the fence related to this image to finish (normally this would be the oldest fence)
                if let Some(image_fence) = &fences[swapchain_image_index as usize] {
                    image_fence.wait(None).unwrap();
                    engine.work_off_event_queue(&mut renderer, swapchain_image_index as usize);
                }

                let previous_future = match fences[previous_fence_i].clone() {
                    None => {
                        let mut now = sync::now(renderer.device.clone());
                        now.cleanup_finished();
                        now.boxed()
                    }
                    Some(fence) => fence.boxed(),
                };
                
                let future = renderer.get_future(
                    previous_future,
                    acquire_future,
                    swapchain_image_index as usize
                );

                fences[swapchain_image_index as usize] = match future {
                    Ok(value) => Some(Arc::new(value)),
                    Err(Validated) => {
                        recreate_swapchain = true;
                        None
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        None
                    }
                };
                println!("Setting previous fence index");
                previous_fence_i = swapchain_image_index as usize;
            },
            Event::WindowEvent { event, .. } => {
                //let pass_events_to_game = !gui.update(&event); // if this returns false, then egui wont have to handle the request and we can pass it to the game
                //if pass_events_to_game {
                    match event {
                        WindowEvent::CloseRequested => control_flow.set_exit(),
                        WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic,
                               
                        } => match input {
                            KeyboardInput { scancode: _, state: ElementState::Pressed, virtual_keycode: Some(key), .. } => {
                                match key {
                                    VirtualKeyCode::Space => {
                                        println!("Called the match Key Event");
                                        for _ in 0..2 {
                                            engine.add_cube_to_scene(None);
                                        }
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        },
                        _ => (),
                    }
                //}
            }
            _ => ()
        }
    });
}
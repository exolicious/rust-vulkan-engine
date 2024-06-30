use egui_winit_vulkano::{egui::{self, epaint::Primitive, pos2, Area, CentralPanel, ClippedPrimitive, Context, Label, RawInput, RichText, ScrollArea, TextEdit, TextStyle, Vec2}, Gui, GuiConfig};
use engine::{engine::Engine, scene::Scene};
use glam::Vec3;
use physics::physics_traits::Transform;
use rendering::{renderer::Renderer};
use vulkano::{format::Format, image::view::ImageView, render_pass::Subpass, single_pass_renderpass, swapchain, sync::{self, future::FenceSignalFuture, GpuFuture}, Validated, VulkanError};
use winit::{event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent}, event_loop::{ControlFlow, EventLoop}};

pub mod initialize;
pub mod rendering;
pub mod physics;
pub mod engine;

use std::{env, sync::{atomic::Ordering, Arc}};

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
    let translation1 = Some(Vec3{x: 10., y: 1., z: 2.});
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
    let mut window_resized = false;
    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(renderer.device.clone()).boxed());
    let swapchain_images_count = renderer.buffer_manager.frames.len();
    let mut frame_in_flight_index = 0;
    
    //let mut gui = Gui::new(&self.event_loop, self.engine.renderer.surface.clone(), None, self.engine.renderer.active_queue.clone(), false);
    

    let gui_subpass = Subpass::from(renderer.render_pass.clone(), 1).unwrap();
    let mut gui = Gui::new_with_subpass(
        &event_loop,
        renderer.surface.clone(),
        renderer.queue.clone(),
        gui_subpass,
        renderer.buffer_manager.gui_image_view.format(),
        GuiConfig::default(),
    );
    let mut code = CODE.to_owned();

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
            Event::RedrawRequested(_) => {
                
            },
            Event::MainEventsCleared => {
                if window_resized || recreate_swapchain {
                    recreate_swapchain = false;
                    
                    //self.renderer.recreate_swapchain(); //this recreates the framebuffers as a sideeffect
                    if window_resized {
                        renderer.recreate_pipeline();
                    }
                }

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
                //if let Some(image_fence) = &fences[swapchain_image_index as usize] {
                //    image_fence.wait(None).unwrap();
                //    engine.work_off_event_queue(&mut renderer, swapchain_image_index as usize);
                //}

                acquire_future.wait(None).unwrap();
                previous_frame_end.as_mut().unwrap().cleanup_finished();
                engine.work_off_event_queue(&mut renderer, frame_in_flight_index as usize);
                
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    let panel_width = 250.0;
                    egui::Window::new("My Window").show(&ctx, |ui| {
                        ui.label("Hello World!");
                     });
                    // Create a fixed-size area
                    Area::new("my_fixed_panel")
                        .fixed_pos(pos2(10.0, 10.0)) // Position the panel as needed
                        .show(&ctx, |ui| {
                            ui.set_min_width(panel_width);
                            ui.set_max_width(panel_width);
                        
                            ui.vertical(|ui| {
                                ui.add_sized(Vec2::new(25.0, 0.0), TextEdit::singleline(&mut String::new()));
                                
                                ui.vertical_centered(|ui| {
                                    ui.add(Label::new("Hi there!"));
                                    ui.label(RichText::new("Rich Text").size(32.0));
                                });
                            
                                ui.separator();
                            
                                ui.columns(2, |columns| {
                                    ScrollArea::vertical().id_source("source").show(&mut columns[0], |ui| {
                                        ui.add(TextEdit::multiline(&mut code).font(TextStyle::Monospace));
                                    });
                                });
                            });
                        });
                });
                println!("draw on subpass image");
                
                let image_extents = get_image_extents_2d(renderer.buffer_manager.frames[swapchain_image_index as usize].swapchain_image_view.clone());
                let gui_command_buffer = gui.draw_on_subpass_image(image_extents);
                
                println!("draw on subpass image worked!!");
                let future = renderer.get_future(
                    previous_frame_end.take().unwrap(),
                    acquire_future,
                    swapchain_image_index as usize,
                    frame_in_flight_index,
                    gui_command_buffer
                );

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(renderer.device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("failed to flush future: {e}");
                        // previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
                frame_in_flight_index = frame_in_flight_index + 1;
                frame_in_flight_index = frame_in_flight_index % swapchain_images_count;
                println!("Setting previous fence index");
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
                                        for _ in 0..100 {
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

const CODE: &str = r"
# Some markup
```
let mut gui = Gui::new(&event_loop, renderer.surface(), None, renderer.queue(), SampleCount::Sample1);
let mut gui = Gui::new(&event_loop, renderer.surface(), None, renderer.queue(), SampleCount::Sample1);
```
";

fn get_image_extents_2d(swapchain_image_view: Arc<ImageView>) -> [u32; 2] {
    [swapchain_image_view.image().extent()[0], swapchain_image_view.image().extent()[1]]
}
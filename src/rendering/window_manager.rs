//use cgmath::Vector3;
//use egui_winit_vulkano::{
//    egui::{self, Context, RawInput, Vec2},
//    Gui,
//};
//use rand::Rng;
//use std::{sync::Arc};
//use vulkano::{
//    swapchain,
//    sync::{self, future::FenceSignalFuture, GpuFuture}, Validated, VulkanError,
//};
//use winit::{
//    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
//    event_loop::{ControlFlow, EventLoop},
//};
//
//use crate::engine::{engine::Engine, scene::Scene};
//
//use super::{renderer::{EngineEvent, EntityUpdateInfo, Renderer}, rendering_traits::RenderableEntity};
//
//pub struct WindowManager {
//    pub renderer: Renderer,
//    pub engine: Engine,
//    pub event_loop: EventLoop<()>,
//}
//
//impl WindowManager {
//    pub fn new() -> Self {
//        let event_loop = EventLoop::new();
//        //event_loop.set_control_flow(ControlFlow::Poll);
//
//        //let window = WindowBuilder::new()
//        //    .with_title("egui with winit")
//        //    .with_inner_size(LogicalSize::new(800, 600))
//        //    .build(&event_loop)
//        //    .unwrap();
//        let mut engine = Engine::new();
//        let mut renderer = Renderer::new(&event_loop);
//
//        let scene_1 = Arc::new(Scene::new());
//        
//        engine.set_active_scene(scene_1.clone());
//        Self { 
//            engine, 
//            event_loop, 
//            renderer, 
//        }
//    }
//
//    pub fn work_off_engine_event_queue(&mut self) {
//        let len = self.engine.event_queue.len();
//        //work off the events
//        for _ in 0..len {
//            match self.engine.event_queue.pop() { // ToDo: decide if fifo or lifo is the right way, for now lifo seems to work
//                Some(EngineEvent::EntityAdded(entity, entity_index)) => self.entity_added_handler(entity, entity_index),
//                Some(EngineEvent::ChangedActiveScene(active_scene)) => self.changed_active_scene_handler(active_scene),
//                //Some(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)) => self.synch_buffers_handler(most_up_to_date_buffer_index, entity),
//                Some(EngineEvent::EntitiesUpdated(updated_entities_infos)) => self.entities_updated_handler(updated_entities_infos),
//                _ => ()
//            }
//        }
//    }
//
//    fn entity_added_handler(&mut self, entity: Arc<dyn RenderableEntity>, entity_index: usize) -> ()  {
//        //println!("Entity added in frame index: {}", acquired_swapchain_index);
//        match self.renderer.buffer_manager.register_entity(entity.clone(), self.renderer.currenty_not_displayed_swapchain_image_index, entity_index) {
//            Ok(()) => {
//                println!("Successfully handled EntityAdded event and added entity with id: {}", entity_index);
//            }
//            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
//        }
//    }
//
//    fn entities_updated_handler(&mut self, updated_entities_infos: Vec<EntityUpdateInfo>) -> ()  {
//        for (i, entity_update_info) in updated_entities_infos.iter().enumerate() {
//            match entity_update_info {
//                EntityUpdateInfo::HasMoved(has_moved_info) => {
//                    let mut entity_model_matrices = Vec::new();
//                    let mut last_index = 0;
//                    if has_moved_info.entity_id - last_index > 1 { 
//                        self.renderer.buffer_manager.copy_transform_data_slice_to_buffer(0, entity_model_matrices.len(), &entity_model_matrices, self.renderer.currenty_not_displayed_swapchain_image_index);
//                        entity_model_matrices.clear();
//                    }
//                    entity_model_matrices.push(has_moved_info.new_transform.model_matrix());
//                    last_index = has_moved_info.entity_id;
//                    
//                },
//                EntityUpdateInfo::ChangedVisibility(changed_visibility_info) => todo!(),
//            }
//        }
//    }
//
//    fn changed_active_scene_handler(&mut self, active_scene: Arc<Scene>) -> ()  {
//        println!("Active scene changed in frame index: {}", self.renderer.currenty_not_displayed_swapchain_image_index);
//        match self.renderer.buffer_manager.copy_vp_camera_data(&active_scene.camera, self.renderer.currenty_not_displayed_swapchain_image_index) {
//            Ok(()) => {
//                println!("Successfully handled Changed Active Scene event");
//            }
//            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
//        }
//
//        //here the buffer_manager would have to do way more after setting the camera matrix, we would have to overwrite the whole state basically.
//        //maybe an idea would be to have 1 buffer manager for each scene
//    }
////
// //   fn synch_camera_buffers_handler(&mut self, most_up_to_date_buffer_index: usize, active_scene: Arc<Scene>) -> () {
// //       if most_up_to_date_buffer_index == self.currenty_not_displayed_swapchain_image_index { 
// //           self.receive_event(EventResolveTiming::Immediate(RendererEvent::BuffersSynched));
// //           println!("all buffers are up to date"); 
// //           return; 
// //       } //if this is not equal, there is still synching to be done, until they are equal
// //       println!("Attempting camera vp buffer sync for frame index: {}", self.currenty_not_displayed_swapchain_image_index);
// //       match self.buffer_manager.unwrap().copy_vp_camera_data(&active_scene.camera, self.currenty_not_displayed_swapchain_image_index) {
// //           Ok(()) => {
// //               self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchCameraBuffers(active_scene, most_up_to_date_buffer_index))); //set the synch event with the index that is now the most up to date (regarding buffer data)
// //               println!("Successfully handled Synch Camera Buffers event");
// //           }
// //           Err(err) => println!("something went wrong while handling the EntityAdded Event"),
// //       }
// //   }
//
//    fn sized_text(ui: &mut egui::Ui, text: impl Into<String>, size: f32) {
//        ui.label(egui::RichText::new(text).size(size));
//    }
//}

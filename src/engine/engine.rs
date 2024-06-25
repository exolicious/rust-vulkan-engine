use std::sync::Arc;

use glam::Vec3;
use egui_winit_vulkano::egui::Window;
use egui_winit_vulkano::Gui;
use rand::Rng;
use winit::event_loop::{EventLoop};

use crate::physics::physics_traits::{Transform};
use crate::rendering::primitives::Mesh;
use crate::rendering::renderer::{EngineEvent, EntityUpdateInfo, HasMovedInfo};
use crate::rendering::rendering_traits::{HasMesh, RenderableEntity, Visibility};
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders};

use super::general_traits::{TickAction};
use super::scene::Scene;
use crate::physics::physics_traits::HasTransform;

pub struct EntityToBufferRegisterData {
    pub id: String,
    pub mesh: Mesh,
    pub transform: Transform
}

pub struct Engine {
    entities: Vec<Box<dyn RenderableEntity>>,
    pub next_swapchain_image_index: usize,
   // scenes: Vec<Arc<Scene>>,
    pub event_queue: Vec<EngineEvent>
}

impl Engine {
    pub fn new() -> Self {
        let entities = Vec::new();
        let event_queue = Vec::new();
        Self {
            entities,
            next_swapchain_image_index: 0,
           // scenes,
            event_queue
        }
    }

    pub fn set_active_scene(& mut self, scene: Arc<Scene>) {
        self.event_queue.push(EngineEvent::ChangedActiveScene(scene));
    }

    pub fn tick(&mut self) -> () {
        //self.renderer.camera.as_mut().unwrap().update_position();
        let mut entities_tick_infos: Vec<EntityUpdateInfo> = Vec::new();
        for (id, entity) in self.entities.iter_mut().enumerate() {
            let entity_update_info = entity.tick();
            match entity_update_info {
                Some(TickAction::HasMoved(transform)) => { 
                    let transform_buffer_info = HasMovedInfo {
                        entity_id: id,
                        new_transform: transform
                    };
                    entities_tick_infos.push(EntityUpdateInfo::HasMoved(transform_buffer_info));
                },
                Some(TickAction::ChangedVisibility(visbility)) => {
                    entities_tick_infos.push(EntityUpdateInfo::ChangedVisibility(visbility));
                }
                None => {},
            }
        }
        self.event_queue.push(EngineEvent::EntitiesUpdated(entities_tick_infos)); 
    }

    pub fn add_cube_to_scene(&mut self, translation: Option<Vec3>) -> () {
        match translation {
            Some(translation) => {
                let mut cube = Box::new(Cube::new(Vec3{ x: 0.25, y: 0.25, z: 0.25 }, Transform { translation, ..Default::default()}));
                let mesh = cube.get_mesh("Cube".to_owned());
                let entity_index = self.entities.len();
                self.event_queue.push(EngineEvent::EntityAdded(cube.get_transform(), mesh, entity_index));
                self.entities.push(cube);
            }
            None => {
                let rand_x: f32 = rand::thread_rng().gen_range(-0.5_f32..0.5_f32);
                let rand_y: f32 = rand::thread_rng().gen_range(-0.5_f32..1_f32);
                let rand_z: f32 = rand::thread_rng().gen_range(-2_f32..-0.7_f32);
                let mut cube: Box<Cube> = Box::new(Cube::new(Vec3{ x: 0.25, y: 0.25, z: 0.25 }, Transform { translation: Vec3 { x: rand_x, y: rand_y, z: rand_z }, ..Default::default() }));
                let mesh = cube.get_mesh("Cube".to_owned());
                let entity_index: usize = self.entities.len();
                self.event_queue.push(EngineEvent::EntityAdded(cube.get_transform(), mesh, entity_index));
                self.entities.push(cube);
            }
        };
    }

    pub fn add_cubes_to_scene(&mut self, translations: Vec<Option<Vec3>>) -> () {

    }

    pub fn work_off_event_queue(&mut self, renderer: & mut Renderer, swapchain_image_index: usize) {
        //println!("working of event queue for image with index: {}", self.next_swapchain_image_index);
        let len = self.event_queue.len();
        //work off the events
        for _ in 0..len {
            match self.event_queue.pop() { // ToDo: decide if fifo or lifo is the right way, for now lifo seems to work
                Some(EngineEvent::EntityAdded(entity_transform, entity_mesh, entity_index)) => renderer.entity_added_handler(entity_transform, entity_mesh, entity_index, swapchain_image_index),
                Some(EngineEvent::ChangedActiveScene(active_scene)) => renderer.changed_active_scene_handler(active_scene),
                //Some(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)) => self.synch_buffers_handler(most_up_to_date_buffer_index, entity),
                Some(EngineEvent::EntitiesUpdated(updated_entities_infos)) => renderer.entities_updated_handler(updated_entities_infos),
                _ => ()
            }
        }
    }
}

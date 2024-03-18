use std::cell::RefCell;
use std::sync::Arc;

use cgmath::Vector3;
use egui_winit_vulkano::egui::Window;
use egui_winit_vulkano::Gui;
use rand::Rng;
use winit::event_loop::{EventLoop};

use crate::physics::physics_traits::{Transform};
use crate::rendering::primitives::Mesh;
use crate::rendering::renderer::{EntityUpdateInfo, EventResolveTiming, HasMovedInfo, RendererBuilder, RendererEvent};
use crate::rendering::rendering_traits::{HasMesh, RenderableEntity, Visibility};
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders};

use super::general_traits::{TickAction};
use super::scene::Scene;

pub struct EntityToBufferRegisterData {
    pub id: String,
    pub mesh: Mesh,
    pub transform: Transform
}

pub struct Engine {
    pub renderer: Renderer,
    entities: Vec<Arc<RefCell<dyn RenderableEntity>>>,
    pub next_swapchain_image_index: usize,
    scenes: Vec<Arc<Scene>>,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>, window: Window) -> Self {
        
        let mut renderer_builder = RendererBuilder::new();
        let mut renderer = renderer_builder.get_renderer();
        renderer.set_builder(Box::new(renderer_builder));

        renderer_builder
            .build_device_extensions()
            .build_physical_device_and_queue_family_index()
            .build_device_and_queues()
            .build_swapchain_and_swapchain_images()
            .build_render_pass()
            .build_buffer_manager()
            .build_shaders()
            .build_pipeline()
            .build_frames();
        
        
        let entities = Vec::new();
        let scene_1 = Arc::new(Scene::new());
        
        renderer.set_active_scene(scene_1.clone());

        let mut scenes = Vec::new();
        scenes.push(scene_1);
        
        Self {
            renderer,
            entities,
            next_swapchain_image_index: 0,
            scenes,
        }
    }

    pub fn tick(&mut self) -> () {
        //self.renderer.camera.as_mut().unwrap().update_position();
        let mut entities_tick_infos: Vec<EntityUpdateInfo> = Vec::new();
        for (id, entity) in self.entities.iter().enumerate() {
            let mut binding = entity.borrow_mut();
            let entity_update_info = binding.tick();
            match entity_update_info {
                Some(TickAction::HasMoved(transform)) => { 
                    let transform_buffer_info = HasMovedInfo {
                        entity_id: id,
                        new_transform: transform
                    };
                    entities_tick_infos.push(EntityUpdateInfo::HasMoved(transform_buffer_info));
                },
                Some(TickAction::ChangedVisibility(Visibility)) => {
                    entities_tick_infos.push(EntityUpdateInfo::ChangedVisibility(Visibility));
                }
                None => {},
            }
        }
        self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntitiesUpdated(entities_tick_infos))); 
    }

    pub fn update_graphics(&mut self) -> () {
        self.renderer.work_off_queue(self.next_swapchain_image_index);
        //self.renderer.update_buffers(self.next_swapchain_image_index);
    }

    pub fn add_cube_to_scene(&mut self, translation: Option<Vector3<f32>>) -> () {
        match translation {
            Some(translation) => {
                let rand_x: f32 = rand::thread_rng().gen_range(-0.5_f32..0.5_f32);
                let rand_y: f32 = rand::thread_rng().gen_range(-0.5_f32..1_f32);
                let rand_z: f32 = rand::thread_rng().gen_range(-2_f32..-0.7_f32);
                let cube = Arc::new(RefCell::new(Cube::new(Vector3{ x: 0.25, y: 0.25, z: 0.25 }, Transform { translation, ..Default::default()})));
                cube.borrow_mut().set_mesh();
                self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntityAdded(cube.clone())));
                self.entities.push(cube);
            }
            None => {
                let cube = Arc::new(RefCell::new(Cube::new(Vector3{ x: 0.25, y: 0.25, z: 0.25 }, Transform { translation: Vector3 { x: 0., y: 0., z: 0. }, ..Default::default() })));
                cube.borrow_mut().set_mesh();
                self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntityAdded(cube.clone())));
                self.entities.push(cube);
            }
        };
    }

    pub fn add_cubes_to_scene(&mut self, translations: Vec<Option<Vector3<f32>>>) -> () {

    }
}

use std::cell::RefCell;
use std::sync::Arc;

use cgmath::Vector3;
use egui_winit_vulkano::Gui;
use rand::Rng;
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::physics::physics_traits::{Transform};
use crate::rendering::primitives::Mesh;
use crate::rendering::renderer::{RendererEvent, EventResolveTiming};
use crate::rendering::rendering_traits::{HasMesh, RenderableEntity};
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders};

use super::general_traits::EntityUpdateAction;
use super::scene::Scene;

pub struct EntityToBufferRegisterData {
    pub id: String,
    pub mesh: Mesh,
    pub transform: Transform
}

pub struct Engine {
    pub renderer: Renderer<Surface>,
    entities: Vec<Arc<RefCell<dyn RenderableEntity>>>,
    pub next_swapchain_image_index: usize,
    scenes: Vec<Arc<Scene>>,
    gui: Gui
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let entities = Vec::new();

        let scene_1 = Arc::new(Scene::new());

        let shaders = Shaders::load(renderer.device.clone()).unwrap();
        
        renderer.set_active_scene(scene_1.clone());
        renderer.build(shaders.vertex_shader, shaders.fragment_shader);

        let mut scenes = Vec::new();
        scenes.push(scene_1);


        let mut gui = Gui::new(&event_loop, renderer.surface, None, renderer.queue(), false);
        
        Self {
            renderer,
            entities,
            next_swapchain_image_index: 0,
            scenes,
            gui
        }
    }

    pub fn update_engine(&mut self) -> () {
        //self.renderer.camera.as_mut().unwrap().update_position();
        for entity in &self.entities {
            let mut binding = entity.borrow_mut();
            let entity_update_info = binding.update();
            match entity_update_info {
                EntityUpdateAction::HasMoved(id, transform) => { self.renderer.buffer_manager.entites_to_update.insert(id, transform); }
                EntityUpdateAction::None => todo!(),
            }
        }
    }

    pub fn update_graphics(&mut self) -> () {
        self.renderer.work_off_queue(self.next_swapchain_image_index);
        self.renderer.buffer_manager.update_buffers(self.next_swapchain_image_index);
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
}

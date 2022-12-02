use std::cell::RefCell;
use std::sync::Arc;

use cgmath::Vector3;
use rand::Rng;
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::physics::physics_traits::{Transform};
use crate::rendering::primitives::Mesh;
use crate::rendering::renderer::{RendererEvent, EventResolveTiming};
use crate::rendering::rendering_traits::{HasMesh, RenderableEntity};
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders};

pub struct EntityToBufferRegisterData {
    pub id: String,
    pub mesh: Mesh,
    pub transform: Transform
}

pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    entities: Vec<Arc<RefCell<dyn RenderableEntity>>>,
    pub next_swapchain_image_index: usize,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let entities = Vec::new();

        let camera = Camera::new();

        let shaders = Shaders::load(renderer.device.clone()).unwrap();
        
        renderer.use_camera(camera);
        renderer.build(shaders.vertex_shader, shaders.fragment_shader);
        
        Self {
            renderer,
            entities,
            next_swapchain_image_index: 0,
        }
    }

    pub fn update_engine(&mut self) -> () {
        //self.renderer.camera.as_mut().unwrap().update_position();

        for entity in &self.entities {
            entity.borrow_mut().update();
        }
    }

    pub fn update_graphics(&mut self) -> () {
        self.renderer.work_off_queue(self.next_swapchain_image_index);
        for entity in &self.entities {
            self.renderer.buffer_manager.update_entity_transform_buffer(entity.borrow().get_id(), entity.borrow().get_transform(), self.next_swapchain_image_index);
        }
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

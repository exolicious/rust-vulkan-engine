use cgmath::Vector3;
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::physics::physics_traits::{Movable, Transform, HasTransform};
use crate::rendering::primitives::Mesh;
use crate::rendering::renderer::{RendererEvent, EventResolveTiming};
use crate::rendering::rendering_traits::{HasMesh, RenderableEntity};
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders};

use super::general_traits::Entity;

pub struct EntityToBufferRegisterData {
    pub id: String,
    pub mesh: Mesh,
    pub transform: Transform
}

pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    entities: Vec<Box<dyn RenderableEntity>>,
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

    pub fn update_engine(&mut self, next_swapchain_image_index: usize) -> () {
        println!("_____________UPDATE_____________");
        self.renderer.camera.as_mut().unwrap().update_position();

        for entity in self.entities.iter_mut() {
            entity.update();
            self.renderer.buffer_manager.borrow_mut().update_entity_transform_buffer(entity.get_id().to_string(), &entity.get_transform(), next_swapchain_image_index);
        }
    }

    pub fn add_cube_to_scene(&mut self, translation: Option<Vector3<f32>>) -> () {
        match translation {
            Some(translation) => {
                let mut cube = Box::new(Cube::new(Vector3{ x: 0.2, y: 0.3, z: 0.2 }, Transform { translation, ..Default::default()}));
                cube.set_mesh();
                let cube_mesh = cube.get_mesh();
                let cube_id = cube.get_id();
                let cube_transform = cube.get_transform();
                let cube_buffer_register_data = EntityToBufferRegisterData {
                    id: cube_id.to_string(), 
                    transform: cube_transform, 
                    mesh: cube_mesh
                };
                self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntityAdded(cube_buffer_register_data)));
                self.entities.push(cube);
            }
            None => {
                let mut cube = Box::new(Cube::new(Vector3{ x: 0.25, y: 0.25, z: 0.25 }, Transform { translation: Vector3 { x: 0., y: 0., z: 0. }, ..Default::default() }));
                cube.set_mesh();
                let cube_mesh = cube.get_mesh();
                let cube_id = cube.get_id();
                let cube_transform = cube.get_transform();
                let cube_buffer_register_data = EntityToBufferRegisterData {
                    id: cube_id.to_string(), 
                    transform: cube_transform, 
                    mesh: cube_mesh
                };
                self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntityAdded(cube_buffer_register_data)));
                self.entities.push(cube);
            }
        };
    }
}

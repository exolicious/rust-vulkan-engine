use std::sync::Arc;

use cgmath::Vector3;
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::physics::physics_traits::{Movable, Transform};
use crate::rendering::entities::Entities;
use crate::rendering::renderer::{RendererEvent, EventResolveTiming};
use crate::rendering::rendering_traits::HasMesh;
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders};

pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    entities: Entities,
    pub next_swapchain_image_index: usize,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let entities = Entities::new();

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

    pub fn update(&mut self) -> () {
        self.renderer.camera.as_mut().unwrap().update_position();

        for entity in &self.entities.entities {
            entity.update();
            self.renderer.buffer_manager.borrow().update_entity_transform_buffer(entity, self.next_swapchain_image_index);
        }
    }

    pub fn add_cube_to_scene(&mut self, translation: Option<Vector3<f32>>) -> () {
        match translation {
            Some(translation) => {
                let mut cube = Cube::new(Vector3{ x: 0.2, y: 0.3, z: 0.2 }, Transform { translation, ..Default::default()});
                cube.set_mesh();
                let wrapped_cube = Arc::new(cube);
                self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntityAdded(wrapped_cube.clone())));
                self.entities.push(wrapped_cube);
            }
            None => {
                let mut cube = Cube::new(Vector3{ x: 0.25, y: 0.25, z: 0.25 }, Transform { translation: Vector3 { x: 0., y: 0., z: 0. }, ..Default::default() });
                cube.set_mesh();
                let wrapped_cube = Arc::new(cube);
                self.renderer.receive_event(EventResolveTiming::NextImage(RendererEvent::EntityAdded(wrapped_cube.clone())));
                self.entities.push(wrapped_cube);
            }
        };
    }
}

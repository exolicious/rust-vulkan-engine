use cgmath::Vector3;
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::physics::physics_traits::{Movable, Transform};
use crate::rendering::entities::Entities;
use crate::rendering::renderer::RendererEvent;
use crate::rendering::rendering_traits::UpdateGraphics;
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders, rendering_traits::{HasMesh}, buffer_manager::{BufferManager}};

pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    entities: Entities,
    pub latest_swapchain_image_index: usize,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let entities = Entities::new();

        let camera = Camera::new(&renderer);
        let shaders = Shaders::load(renderer.device.clone()).unwrap();

        renderer.use_camera(camera);
        renderer.build(shaders.vertex_shader, shaders.fragment_shader);
        
        Self {
            renderer,
            entities,
            latest_swapchain_image_index: 0,
        }
    }

    pub fn update_graphics(&mut self) -> () {
        self.renderer.work_off_queue();
        self.renderer.camera.as_mut().unwrap().update_graphics(self.latest_swapchain_image_index);
        for entity in &self.entities.entities {
            entity.update_graphics(self.latest_swapchain_image_index);
        }
    }

    pub fn update(&mut self) -> () {
        self.renderer.camera.as_mut().unwrap().update_position();
    }

    pub fn add_cube_to_scene(&mut self, origin: Option<Vector3<f32>>){
        match origin {
            Some(origin ) => {
                let mut cube = Cube::new(Vector3{x: 0.2, y: 0.3, z: 0.2}, Transform { position: origin, ..Default::default()});
                cube.generate_mesh();
                self.entities.push(Box::new(cube));
                self.renderer.receive_event(RendererEvent::EntityAdded);
            }
            None => {
                let mut cube = Cube::new(Vector3{x: 0.2, y: 0.3, z: 0.2}, Transform::default());
                cube.generate_mesh(); 
                self.entities.push(Box::new(cube));
                self.renderer.receive_event(RendererEvent::EntityAdded);
            }
        };

    }

}

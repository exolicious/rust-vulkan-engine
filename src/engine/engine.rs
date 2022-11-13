use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::engine::general_traits::UniformBufferOwner;
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders, rendering_traits::Mesh};

use super::general_traits::Update;

pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    pub entities: Vec<Box<dyn Update>>,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let mut entities: Vec<Box<dyn Update>> = Vec::new();

        let cube = Box::new(Cube::new());
        let camera = Box::new(Camera::new(&renderer));

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            renderer.device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            cube.unwrap_vertices().into_iter(),
        ).unwrap();

        
        let shaders = Shaders::load(renderer.device.clone()).unwrap();
        renderer.build(shaders.vertex_shader, shaders.fragment_shader, camera.get_uniform_buffer(), vertex_buffer);

        entities.push(cube);
        entities.push(camera);
        Self {
            renderer,
            entities
        }
    }

    pub fn update(& mut self) {
        for entity in & mut self.entities {
            entity.update();
        }
    }
}
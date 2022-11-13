use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::swapchain::{Surface, self, AcquireError};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::engine::general_traits::UniformBufferOwner;
use crate::rendering::primitives::{Cube, Mesh};
use crate::rendering::renderer::Renderer;
use crate::rendering::shaders::Shaders;

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
        renderer.init_shaders(shaders.vertex_shader.clone(), shaders.fragment_shader.clone());
        renderer.init_uniform_buffers(camera.get_uniform_buffer().clone());
        renderer.init_vertex_buffers(vertex_buffer.clone());
        renderer.create_command_buffers();

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
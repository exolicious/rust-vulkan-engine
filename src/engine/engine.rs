use std::borrow::{BorrowMut, Cow};
use std::cell::RefCell;
use std::rc::Rc;

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::physics::physics_traits::Movable;
use crate::rendering::rendering_traits::UniformBufferOwner;
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders, rendering_traits::Mesh};

use crate::rendering::rendering_traits::UpdateGraphics;

pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    pub entities: Vec<Rc<RefCell<dyn UpdateGraphics>>>,
    pub camera: Rc<RefCell<Camera>>,
    pub latest_swapchain_image_index: usize,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let mut entities: Vec<Rc<RefCell<dyn UpdateGraphics>>> = Vec::new();

        let cube = Rc::new(RefCell::new(Cube::new()));
        let camera = Rc::new(RefCell::new(Camera::new(&renderer)));

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            renderer.device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            cube.borrow().unwrap_vertices().into_iter(),
        ).unwrap();

        
        let shaders = Shaders::load(renderer.device.clone()).unwrap();
        renderer.build(shaders.vertex_shader, shaders.fragment_shader, camera.borrow().get_uniform_buffers(), vertex_buffer);

        entities.push(cube.clone());
        entities.push(camera.clone());
        Self {
            renderer,
            entities,
            camera: camera,
            latest_swapchain_image_index: 0,
        }
    }

    pub fn update_graphics(&mut self) -> () {
        for entity in & mut self.entities {
            entity.try_borrow_mut().unwrap().update_graphics(self.latest_swapchain_image_index);
        }
    }

    pub fn update(&mut self) -> () {
        self.camera.try_borrow_mut().unwrap().update_position();
    }

}
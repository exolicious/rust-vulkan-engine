use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use cgmath::Vector3;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::swapchain::{Surface};
use winit::event_loop::{EventLoop};
use winit::window::Window;

use crate::camera::camera::Camera;
use crate::physics::physics_traits::{Movable, Transform};
use crate::rendering::primitives::Vertex;
use crate::rendering::renderer::RendererEvent;
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

        let cube = Cube::default();
        
        let camera = Rc::new(RefCell::new(Camera::new(&renderer)));
        let vertex_buffer = Engine::create_vertex_buffer(&cube, &renderer);
        let mut initial_vertex_buffers = Vec::new();
        initial_vertex_buffers.push(vertex_buffer);
        
        let shaders = Shaders::load(renderer.device.clone()).unwrap();
        renderer.build(shaders.vertex_shader, shaders.fragment_shader, camera.borrow().get_uniform_buffers(), Some(initial_vertex_buffers));
        
        let wrapped_cube = Rc::new(RefCell::new(cube));
        entities.push(wrapped_cube.clone());
        entities.push(camera.clone());
        Self {
            renderer,
            entities,
            camera: camera,
            latest_swapchain_image_index: 0,
        }
    }

    pub fn update_graphics(&mut self) -> () {
        self.renderer.work_off_queue();
        for entity in & mut self.entities {
            entity.try_borrow_mut().unwrap().update_graphics(self.latest_swapchain_image_index);
        }
    }

    pub fn update(&mut self) -> () {
        self.camera.try_borrow_mut().unwrap().update_position();
    }

    pub fn add_cube_to_scene(&mut self){
        let mut cube = Cube::new(Vector3{x: 0.2, y: 0.3, z: 0.2}, Transform::default());
        let vertex_buffer = Engine::create_vertex_buffer(&cube, &self.renderer);

        cube.update_position();
        let wrapped_cube = Rc::new(RefCell::new(cube));
        
        self.entities.push(wrapped_cube);
        self.renderer.receive_event(RendererEvent::EntityAdded(vertex_buffer));
        
    }

    pub fn create_vertex_buffer(object: &Cube, renderer: &Renderer<Surface<Window>>) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        /* let mut temp_vec = Vec::new(); */
        /* let vertex_buffer =  */
        CpuAccessibleBuffer::from_iter(
            renderer.device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            object.unwrap_vertices().into_iter(),
        )
        .unwrap()
/* 
        temp_vec.push(vertex_buffer);
        Some(temp_vec) */
    }
}
use std::borrow::BorrowMut;
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
use crate::rendering::entities::Entities;
use crate::rendering::primitives::Vertex;
use crate::rendering::renderer::RendererEvent;
use crate::rendering::{{primitives::Cube}, renderer::Renderer, shaders::Shaders, rendering_traits::{HasMesh}};


pub struct Engine {
    pub renderer: Renderer<Surface<Window>>,
    entities: Entities,
    pub latest_swapchain_image_index: usize,
}

impl Engine {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut renderer = Renderer::new(&event_loop);
        let mut entities = Entities::new();

        let mut cube = Box::new(Cube::default());
        cube.generate_mesh();
        let camera = Camera::new(&renderer);

        
        let vertex_buffer = Engine::create_vertex_buffer(&cube, &renderer);
        let mut initial_vertex_buffers = Vec::new();
        initial_vertex_buffers.push(vertex_buffer);
        let shaders = Shaders::load(renderer.device.clone()).unwrap();

        renderer.use_camera(camera);
        renderer.build(shaders.vertex_shader, shaders.fragment_shader, Some(initial_vertex_buffers));

        entities.push(cube);
        
        Self {
            renderer,
            entities,
            latest_swapchain_image_index: 0,
        }
    }

    pub fn update_graphics(&mut self) -> () {
        self.renderer.work_off_queue();
        for entity in &self.entities.entities {
            entity.update_graphics(self.latest_swapchain_image_index);
        }
    }

    pub fn update(&mut self) -> () {
        self.renderer.camera.as_mut().unwrap().update_position();
    }

    pub fn add_cube_to_scene(&mut self){
        let mut cube = Cube::new(Vector3{x: 0.2, y: 0.3, z: 0.2}, Transform::default());
        cube.generate_mesh();
        let vertex_buffer = Engine::create_vertex_buffer(&cube, &self.renderer);
        cube.update_position();
        let wrapped_cube = Box::new(cube);
        
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

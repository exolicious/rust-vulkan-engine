use std::{collections::HashMap, sync::Arc, rc::Rc};

use vulkano::{buffer::{BufferAccess, CpuAccessibleBuffer, BufferUsage}, swapchain::Surface};
use winit::window::Window;

use super::{rendering_traits::HasMesh, renderer::Renderer, primitives::Vertex};


pub struct BufferManager {
    model_blueprints: HashMap<u64, Box<dyn HasMesh>>,
    buffers: Vec<Arc<dyn BufferAccess>>
}

impl BufferManager {
    pub fn new() -> Self {
        let buffers = Vec::new();
        let model_blueprints = HashMap::new();
        Self {
            model_blueprints,
            buffers
        }
    }

    pub fn set_up_vertex_buffer(&self, entities_to_render: Vec<&dyn HasMesh>) {
        
    }
    
    pub fn create_vertex_buffer(&self, object: &dyn HasMesh, renderer: &Renderer<Surface<Window>>) -> () {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            renderer.device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            object.unwrap_vertices().into_iter(),
        )
        .unwrap();
        self.buffers.push(vertex_buffer);
    }

}
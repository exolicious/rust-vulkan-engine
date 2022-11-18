use std::{collections::HashMap, sync::Arc};

use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage}, device::Device};

use super::{rendering_traits::HasMesh, primitives::Vertex};

struct EntityBufferAccessor {
    pub buffer_offset: u32,
    pub vertex_count: u64,
}

struct ModelSpaceCounter<'a> {
    pub accessor: &'a EntityBufferAccessor,
    pub first_index: u32,
    pub instance_counter: u32,
}

impl ModelSpaceCounter<'_> {
    fn required_space(&self) -> u64 {
        self.accessor.vertex_count * self.instance_counter as u64
    }
}

pub struct BufferManager {
    model_blueprints: HashMap<String, Arc<dyn HasMesh>>,
    renderer_device:  Arc<Device>,
    entity_id_accessor_map: HashMap<String, EntityBufferAccessor>,
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>
}

impl BufferManager {
    pub fn new(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Self {
        let vertex_buffer = Self::initialize_vertex_buffer(renderer_device.clone());
        let model_blueprints = HashMap::new();
        let entity_id_accessor_map: HashMap<String, EntityBufferAccessor> = HashMap::new();
        Self {
            model_blueprints,
            renderer_device,
            entity_id_accessor_map,
            vertex_buffer
        }
    }

    pub fn add_to_buffer(& self, id: &String, data: Vec<Vertex>) {
        match self.vertex_buffer.write() {
            Err(_) => println!("Error"),
            Ok(mut write_lock) => { 
                *write_lock[0..2] = 
            }
        };
    }

    pub fn set_up_vertex_buffer(& self, entities_to_render: Vec<&dyn HasMesh>) {
        /* let vertex_buffer = CpuAccessibleBuffer::from_iter(
            renderer.device.clone(),
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            object.unwrap_vertices().into_iter(),
        )
        .unwrap(); */
    }
    
    fn initialize_vertex_buffer(renderer_device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        let initializer_data: Vec<Vertex> = vec![Vertex{position: [0.,0.,0.]}; 2_i32.pow(8).try_into().unwrap()];
        CpuAccessibleBuffer::from_iter(
            renderer_device,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            initializer_data.into_iter()
        )
        .unwrap()
    }
    
}
use std::{collections::HashMap, error::Error, ops::Index, sync::Arc};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

use super::{mesh_accessor::{MeshAccessor, MeshAccessorAddEntityResult}, primitives::{Mesh, Vertex}};

pub struct VertexBuffers {
    vertex_buffers: Vec<Subbuffer<[Vertex]>>,
    pub mesh_accessor: MeshAccessor,
    pub newly_added_mesh_first_and_last_vertex_index: Option<(usize, usize)>
}
const INITIAL_VERTEX_BUFFER_SIZE: usize = 2_i32.pow(16) as usize; 

impl VertexBuffers {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_images_length: usize) -> Self {
        let mut vertex_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let initializer_data = vec![Vertex{position: [0.,0.,0.]}; INITIAL_VERTEX_BUFFER_SIZE];
            let vertex_buffer = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                initializer_data.into_iter()
            )
            .unwrap();
            vertex_buffers.push(vertex_buffer);

        }   
        let mesh_accessor = MeshAccessor::default();

        Self {
            vertex_buffers,
            mesh_accessor,
            newly_added_mesh_first_and_last_vertex_index: None
        }
    }

    pub fn bind_entity_mesh(&mut self, entity_mesh: Mesh, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let first_index = self.mesh_accessor.get_last_vertex_index();
        let entity_add_result: MeshAccessorAddEntityResult = self.mesh_accessor.add_entity(entity_mesh);
        match entity_add_result {
            MeshAccessorAddEntityResult::AppendedToExistingMesh => {},
            MeshAccessorAddEntityResult::CreatedNewMesh(mesh) => {
                self.copy_blueprint_mesh_data_to_vertex_buffer(first_index, &mesh.data, next_swapchain_image_index)?;
                self.newly_added_mesh_first_and_last_vertex_index = Some((first_index, self.mesh_accessor.get_last_vertex_index()));
            }
        }
        Ok(())
    }

    fn copy_blueprint_mesh_data_to_vertex_buffer(& self, first_index: usize, mesh_data: &Vec<Vertex>, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let offline_vertex_buffer = self.vertex_buffers[next_swapchain_image_index].clone();
        let mut write_lock =  offline_vertex_buffer.write()?;
        write_lock[first_index..mesh_data.iter().len()].copy_from_slice(mesh_data.as_slice());
        //println!("Successfully copied mesh data: {:?} to vertex buffer with index: {}", mesh_data.as_slice(), next_swapchain_image_index);
        Ok(())
    }

    pub fn get_synch_info(& self, unsynched_ahead_buffer_index: usize) -> (Subbuffer<[Vertex]>, Vec<Subbuffer<[Vertex]>>) {
        let most_up_to_date_buffer = &self.vertex_buffers[unsynched_ahead_buffer_index];
        let mut buffers_to_update = Vec::new();
        for (i, transform_buffer) in self.vertex_buffers.iter().enumerate() {
            buffers_to_update.push(transform_buffer.clone());
        }
        return (most_up_to_date_buffer.clone(), buffers_to_update)
    }
    
}

impl Index<usize> for VertexBuffers {
    type Output = Subbuffer<[Vertex]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vertex_buffers[index]
    }
}
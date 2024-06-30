use std::{error::Error, ops::Index, sync::Arc};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

use crate::physics::physics_traits::Transform;


pub struct TransformBuffers {
    transform_buffers: Vec<Subbuffer<[[[f32; 4]; 4]]>>,
    entity_to_transform_buffer_index: Vec<usize>,
    synch_source_buffer: Option<Subbuffer<[[[f32; 4]; 4]]>>,
    pub newly_added_transform_indexes: Vec<usize>,
    out_of_synch_buffers_count: usize
}

pub struct TransformBufferCopyPayload {
    pub src_buffer: Subbuffer<[[[f32; 4]; 4]]>, 
    pub target_buffer: Subbuffer<[[[f32; 4]; 4]]>, 
    pub newly_added_transform_indexes: Vec<usize>
}

const INITIAL_TRANSFORM_BUFFER_SIZE: usize = 2_i32.pow(12) as usize; // 32 instances

impl TransformBuffers {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_images_length: usize) -> Self {
        let mut transform_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let transform_initial_data: [[[f32; 4]; 4]; INITIAL_TRANSFORM_BUFFER_SIZE] = [[[0_f32; 4]; 4]; INITIAL_TRANSFORM_BUFFER_SIZE];
            let uniform_buffer = Buffer::from_iter(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_SRC | BufferUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                transform_initial_data.into_iter()
            )
            .unwrap();
            transform_buffers.push(uniform_buffer);
        }
        let entity_to_transform_buffer_index = Vec::new();
        let newly_added_transform_indexes = Vec::new();
        Self {
            transform_buffers,
            entity_to_transform_buffer_index,
            synch_source_buffer: None,
            out_of_synch_buffers_count: 0,
            newly_added_transform_indexes
        }
    }

    pub fn bind_entity_transform(&mut self, entity_transform: Transform, entity_id: usize, frame_in_flight_index: usize) -> Result<(), Box<dyn Error>> {
        let entity_transform_index = self.entity_to_transform_buffer_index.len();
        self.entity_to_transform_buffer_index.push(entity_id);
        self.newly_added_transform_indexes.push(entity_transform_index);
        self.out_of_synch_buffers_count = self.transform_buffers.len() - 1;
        let synch_source_buffer = self.transform_buffers[frame_in_flight_index].clone();
        self.synch_source_buffer = Some(synch_source_buffer);
        self.copy_transform_data_to_buffer(entity_transform_index, &entity_transform, frame_in_flight_index)
    }

 // pub fn get_synch_slice(&mut self) -> &[usize] {
 //     self.newly_added_transform_indexes.unwrap().as_slice()
 // }

    pub fn update_entity_transform(& self, entity_transform_index: usize, entity_transform: &Transform, frame_in_flight_index: usize) -> Result<(), Box<dyn Error>> {
        self.copy_transform_data_to_buffer(entity_transform_index, entity_transform, frame_in_flight_index)
    }

    fn copy_transform_data_to_buffer(& self, entity_transform_index: usize, entity_transform: &Transform, frame_in_flight_index: usize) -> Result<(), Box<dyn Error>> {
        println!("DEBUG");
        let mut write_lock =  self.transform_buffers[frame_in_flight_index].write()?;
        write_lock[entity_transform_index] = entity_transform.model_matrix();
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn copy_transform_data_slice_to_buffer(& self, entity_transforms_first_index: usize, entity_transforms_last_index: usize, entity_model_matrices: &[[[f32; 4]; 4]], frame_in_flight_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock =  self.transform_buffers[frame_in_flight_index].write()?;
        write_lock[entity_transforms_first_index..entity_transforms_last_index].copy_from_slice(entity_model_matrices);
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn get_tansform_buffer_copy_payload(& mut self, frame_in_flight_index: usize) -> Option<TransformBufferCopyPayload> {
        if self.out_of_synch_buffers_count == 0 {
            return None
        }
        match self.synch_source_buffer.clone() {
            Some(synch_source_buffer) => {
                let target_buffer_index = (frame_in_flight_index + 1) % self.transform_buffers.len();
                let target_buffer = self.transform_buffers[target_buffer_index].clone();
                let newly_added_transform_indexes = self.newly_added_transform_indexes.clone();
                println!("Synching {} newly added transforms", newly_added_transform_indexes.len());
                self.out_of_synch_buffers_count = self.out_of_synch_buffers_count - 1;
                return Some(TransformBufferCopyPayload {
                    src_buffer: synch_source_buffer.clone(), 
                    target_buffer, 
                    newly_added_transform_indexes
                })
            }
            None => return None
        }
    }
}

impl Index<usize> for TransformBuffers {
    type Output = Subbuffer<[[[f32; 4]; 4]]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.transform_buffers[index]
    }
}
use std::{error::Error, ops::Index, sync::Arc};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

use crate::physics::physics_traits::Transform;


pub struct TransformBuffers {
    transform_buffers: Vec<Subbuffer<[[[f32; 4]; 4]]>>,
    entity_to_transform_buffer_index: Vec<usize>,
    pub newly_added_transform_indexes: Vec<usize>,
}

pub struct TransformBufferCopyPayload {
    pub src_buffer: Subbuffer<[[[f32; 4]; 4]]>, 
    pub target_buffers: Vec<Subbuffer<[[[f32; 4]; 4]]>>, 
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
            newly_added_transform_indexes
        }
    }

    pub fn bind_entity_transform(&mut self, entity_transform: Transform, entity_id: usize, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let entity_transform_index = self.entity_to_transform_buffer_index.len();
        self.entity_to_transform_buffer_index.push(entity_id);
        self.newly_added_transform_indexes.push(entity_transform_index);
        self.copy_transform_data_to_buffer(entity_transform_index, &entity_transform, next_swapchain_image_index)
    }

 // pub fn get_synch_slice(&mut self) -> &[usize] {
 //     self.newly_added_transform_indexes.unwrap().as_slice()
 // }

    pub fn update_entity_transform(& self, entity_transform_index: usize, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        self.copy_transform_data_to_buffer(entity_transform_index, entity_transform, next_swapchain_image_index)
    }

    fn copy_transform_data_to_buffer(& self, entity_transform_index: usize, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        println!("DEBUG");
        let mut write_lock =  self.transform_buffers[next_swapchain_image_index].write()?;
        write_lock[entity_transform_index] = entity_transform.model_matrix();
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn copy_transform_data_slice_to_buffer(& self, entity_transforms_first_index: usize, entity_transforms_last_index: usize, entity_model_matrices: &[[[f32; 4]; 4]], next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock =  self.transform_buffers[next_swapchain_image_index].write()?;
        write_lock[entity_transforms_first_index..entity_transforms_last_index].copy_from_slice(entity_model_matrices);
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn get_tansform_buffer_copy_payload(& mut self, unsynched_ahead_buffer_index: usize) -> Option<TransformBufferCopyPayload> {
        let newly_added_transform_indexes = self.newly_added_transform_indexes.clone();
        if newly_added_transform_indexes.len() < 1 {
            return None
        }
        println!("Synching {} newly added transforms", newly_added_transform_indexes.len());
        let most_up_to_date_buffer = &self.transform_buffers[unsynched_ahead_buffer_index];
        let mut buffers_to_update = Vec::new();
        println!("Source buffer index: {}", unsynched_ahead_buffer_index);
        for (i, transform_buffer) in self.transform_buffers.iter().enumerate() {
            if i != unsynched_ahead_buffer_index {
                println!("Adding buffer with index: {} to TransformBufferCopyPayload", i);
                buffers_to_update.push(transform_buffer.clone());
            }
        }
        self.newly_added_transform_indexes.clear();
        return Some(TransformBufferCopyPayload {
            src_buffer: most_up_to_date_buffer.clone(), 
            target_buffers: buffers_to_update, 
            newly_added_transform_indexes
        })
    }

    pub fn clear_newly_added_transform_indexes(& mut self) -> () {
        self.newly_added_transform_indexes.clear();
    }
    
}

impl Index<usize> for TransformBuffers {
    type Output = Subbuffer<[[[f32; 4]; 4]]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.transform_buffers[index]
    }
}
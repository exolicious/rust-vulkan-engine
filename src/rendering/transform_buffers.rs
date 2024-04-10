use std::{error::Error, ops::Index, sync::Arc};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}};

use crate::physics::physics_traits::Transform;


pub struct TransformBuffers {
    transform_buffers: Vec<Subbuffer<[[[f32; 4]; 4]]>>,
    entity_to_transform_buffer_index: Vec<usize>,
    pub newly_added_transform_indexes: Option<Vec<usize>>,
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
                    usage: BufferUsage::UNIFORM_BUFFER,
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

        Self {
            transform_buffers,
            entity_to_transform_buffer_index,
            newly_added_transform_indexes: None
        }
    }

    pub fn bind_entity_transform(&mut self, entity_transform: Transform, entity_id: usize, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        self.entity_to_transform_buffer_index.push(entity_id);
        let entity_transform_index = self.entity_to_transform_buffer_index.len();
        match & mut self.newly_added_transform_indexes {
            Some(vec) => vec.push(entity_transform_index),
            None => { self.newly_added_transform_indexes = Some(Vec::new()) }
        }
        self.copy_transform_data_to_buffer(entity_transform_index, &entity_transform, next_swapchain_image_index)
    }

 // pub fn get_synch_slice(&mut self) -> &[usize] {
 //     self.newly_added_transform_indexes.unwrap().as_slice()
 // }

    pub fn update_entity_transform(& self, entity_transform_index: usize, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        self.copy_transform_data_to_buffer(entity_transform_index, entity_transform, next_swapchain_image_index)
    }

    fn copy_transform_data_to_buffer(& self, entity_transform_index: usize, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
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

    pub fn get_synch_info(& self, unsynched_ahead_buffer_index: usize) -> (Subbuffer<[[[f32; 4]; 4]]>, Vec<Subbuffer<[[[f32; 4]; 4]]>>) {
        let most_up_to_date_buffer = &self.transform_buffers[unsynched_ahead_buffer_index];
        let mut buffers_to_update = Vec::new();
        for (i, transform_buffer) in self.transform_buffers.iter().enumerate() {
            buffers_to_update.push(transform_buffer.clone());
        }
        return (most_up_to_date_buffer.clone(), buffers_to_update)
    }

    pub fn get_newly_added_transform_indexes(& mut self) -> Option<&Vec<usize>> {
        self.newly_added_transform_indexes.as_ref()
    }
    
}

impl Index<usize> for TransformBuffers {
    type Output = Subbuffer<[[[f32; 4]; 4]]>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.transform_buffers[index]
    }
}
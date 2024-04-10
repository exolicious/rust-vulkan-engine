use std::{collections::{HashMap}, sync::Arc, cell::RefCell};
use cgmath::{Matrix4, SquareMatrix};
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, BufferCopy, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassEndInfo}, descriptor_set::{allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, CopyDescriptorSet, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, image::Image, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint}, render_pass::{Framebuffer, RenderPass}};
use crate::{engine::camera::Camera, physics::physics_traits::Transform};
use super::{frame::Frame, primitives::Mesh, rendering_traits::RenderableEntity, transform_buffers::TransformBuffers, vertex_buffers::VertexBuffers};
use std::error::Error;
use core::fmt::Error as ErrorVal;



pub struct BufferManager {
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    /*     renderer_device:  Arc<Device>, */
   /*  entity_transform_buffer_entries: HashMap<u64, Vec<EntityAccessor>>, */
    pub vertex_buffers: VertexBuffers,
    pub frames: Vec<Frame>,
    queue_family_index: u32,
    pub transform_buffers: TransformBuffers,
    vp_camera_buffers:  Vec<Subbuffer<[[f32; 4]; 4]>>, // needs to be a push constant sooner or later
    pub entities_transform_ids: Vec<String>,
    pub entites_to_update: HashMap<String, Transform>,
    pipeline: Arc<GraphicsPipeline>,
}

impl BufferManager {
    pub fn new(device: Arc<Device>, pipeline: Arc<GraphicsPipeline>, swapchain_images: Vec<Arc<Image>>, render_pass: Arc<RenderPass>, queue_family_index: u32) -> Self {
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
            device.clone(), 
            StandardDescriptorSetAllocatorCreateInfo::default());
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let frames = BufferManager::build_frames(device.clone(), pipeline.clone(), swapchain_images.clone(), render_pass.clone(), queue_family_index);
        let vertex_buffers = VertexBuffers::new(memory_allocator.clone(), swapchain_images.len());
        let transform_buffers = TransformBuffers::new(memory_allocator.clone(), swapchain_images.len());
        let vp_camera_buffers = Self::initialize_vp_camera_buffers(memory_allocator.clone(), swapchain_images.len());

        let entities_transform_ids = Vec::new();
        let entites_to_update = HashMap::new();

        Self {
            vertex_buffers,
            transform_buffers,
            vp_camera_buffers,
            entities_transform_ids,
            descriptor_set_allocator,
            frames,
            command_buffer_allocator,
            memory_allocator,
            entites_to_update,
            pipeline,
            queue_family_index,
        }
    }

    //has to be called again, when its buffers are out of date (re-allocated due to being too small), or when the swapchain gets updated (window gets resized, or old swapchain was suboptimal )
    pub fn build_frames(device: Arc<Device>, pipeline: Arc<GraphicsPipeline>, swapchain_images: Vec<Arc<Image>>, render_pass: Arc<RenderPass>, queue_family_index: u32) -> Vec<Frame> {
        let mut temp_frames = Vec::new();
        for (swapchain_image_index, swapchain_image) in swapchain_images.iter().enumerate() {
            let mut temp_frame = Frame::new(
                swapchain_image.clone(), 
                device.clone(), 
                swapchain_image_index
            );
            temp_frame.init_framebuffer(render_pass.clone());
            //temp_frame.init_command_buffer(queue_family_index, buffer_manager, 0);
            temp_frames.push(temp_frame);
        }
        temp_frames
    }

    fn initialize_vp_camera_buffers(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_images_length: usize) -> Vec<Subbuffer<[[f32; 4]; 4]>> {
        let mut vp_matrix_buffers = Vec::new();
        let projection_view_matrix: Matrix4<f32> = Matrix4::identity();
        for _ in 0..swapchain_images_length {
            let uniform_buffer = Buffer::from_data(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                projection_view_matrix.into(),
            )
            .unwrap();
            vp_matrix_buffers.push(uniform_buffer);
        }
        vp_matrix_buffers
    }

    pub fn register_entity(&mut self, entity_transform: Transform, entity_mesh: Mesh, next_swapchain_image_index: usize, entity_index: usize) -> Result<(), Box<dyn Error>> {
        self.vertex_buffers.bind_entity_mesh(entity_mesh, next_swapchain_image_index)?;
        self.transform_buffers.bind_entity_transform(entity_transform, next_swapchain_image_index, entity_index);
        Ok(())
    }

    pub fn update_buffers(&mut self, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut entity_model_matrices = Vec::new();
        let mut last_index = 0;
        for (i, (id, transform)) in self.entites_to_update.iter().enumerate() {
            if i - last_index > 1 { 
                self.copy_transform_data_slice_to_buffer(0, entity_model_matrices.len(), &entity_model_matrices, next_swapchain_image_index)?;
            }
            entity_model_matrices.push(transform.model_matrix());
            last_index = i;
        }
        Ok(())
    }

    pub fn copy_transform_data_slice_to_buffer(& self,entity_transforms_first_index: usize, entity_transforms_last_index: usize, entity_model_matrices: &[[[f32; 4]; 4]], next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock =  self.transform_buffers[next_swapchain_image_index].write()?;
        write_lock[entity_transforms_first_index..entity_transforms_last_index].copy_from_slice(entity_model_matrices);
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn update_entity_transform_buffer(& self, entity_id: &String, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        match self.entities_transform_ids.iter().position(|existing_entity_id| existing_entity_id == entity_id) {
            Some(entity_transform_index) => {
                self.transform_buffers.update_entity_transform(entity_transform_index, entity_transform, next_swapchain_image_index)?;
                Ok(())
            }
            None => Err(Box::new(ErrorVal))
        }
    }

    pub fn copy_vp_camera_data(& self, camera: &Camera, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock = self.vp_camera_buffers[next_swapchain_image_index].write()?;
        *write_lock = camera.projection_view_matrix.into();
        println!("Successfully copied camera vp_matrix: {:?} to vp buffer with index: {}", camera.projection_view_matrix, next_swapchain_image_index);
        Ok(())
    }

    pub fn get_vp_matrix_buffer_descriptor_set(& self, next_swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, self.vp_camera_buffers[next_swapchain_image_index].clone())], // 0 is the binding
            []
        )
        .unwrap()
    }

    pub fn get_transform_buffer_descriptor_set(& self, next_swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let layout = self.pipeline.layout().set_layouts().get(1).unwrap();
        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, self.transform_buffers[next_swapchain_image_index].clone())],
            []
        )
        .unwrap()
    }

    pub fn build_command_buffer(& mut self, acquired_swapchain_image: usize) -> Arc<PrimaryAutoCommandBuffer> {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue_family_index,
            CommandBufferUsage::MultipleSubmit,
        ) 
        .unwrap();

        let mut descriptor_sets = Vec::new();
        descriptor_sets.push(self.get_vp_matrix_buffer_descriptor_set(acquired_swapchain_image).clone());
        descriptor_sets.push(self.get_transform_buffer_descriptor_set(acquired_swapchain_image).clone());
        
        let vertex_buffer = self.vertex_buffers[acquired_swapchain_image].clone();

        let builder = command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.frames[acquired_swapchain_image].framebuffer.as_ref().unwrap().clone())
                },
                vulkano::command_buffer::SubpassBeginInfo { ..Default::default() }
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, vertex_buffer)
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_sets,
            )
            .unwrap();

           //let newly_added_tranform_indexes = self.transform_buffers.get_newly_added_transform_indexes();

           //let builder = match newly_added_tranform_indexes {
           //    Some(newly_added_tranform_indexes) => self.build_synch_transform_buffers_commands(builder, acquired_swapchain_image, &newly_added_tranform_indexes),
           //    None => builder, 
           //};

           //let builder = match self.vertex_buffers.newly_added_mesh_first_and_last_vertex_index {
           //    Some((first_vertex_index, last_vertex_index)) => { 
           //        let builder = self.build_synch_vertex_buffers_commands(builder, acquired_swapchain_image, first_vertex_index, last_vertex_index);
           //        self.vertex_buffers.newly_added_mesh_first_and_last_vertex_index = None;
           //        builder
           //    }
           //    None => {builder}
           //};

            let mesh_accessor = & self.vertex_buffers.mesh_accessor;
            print!("-----------------------mesh accessor.meshes length is: {} ----------------", mesh_accessor.meshes.len());
            if mesh_accessor.meshes.len() > 0 {
                let builder = self.vertex_buffers.mesh_accessor.meshes.iter().fold(builder, |builder, mesh| {
                    let instances_count = mesh_accessor.mesh_name_instance_count_map.get(&mesh.name).unwrap();
                    let meshes_first_vertex_index = mesh_accessor.mesh_name_first_vertex_index_map.get(&mesh.name).unwrap();
                    builder
                        .draw(mesh.data.len() as u32, *instances_count as u32, *meshes_first_vertex_index as u32, 0)
                        .unwrap()
                });
                builder
                .end_render_pass(SubpassEndInfo::default())
                .unwrap();
            }
            else {
                builder
                .end_render_pass(SubpassEndInfo::default())
                .unwrap();
            }

        let command_buffer = command_buffer_builder.build().unwrap();
        command_buffer
    }

    fn build_synch_transform_buffers_commands<'a>(&'a mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, unsynched_ahead_buffer_index: usize, newly_added_transform_indexes: &Vec<usize>) -> & mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        let (most_up_to_date_buffer, buffers_to_update) = self.transform_buffers.get_synch_info(unsynched_ahead_buffer_index);
        for buffer_to_update in buffers_to_update.iter() {
            let mut copy_info = CopyBufferInfo::buffers(most_up_to_date_buffer.clone(), buffer_to_update.clone());
            let first_index_of_newly_added_transforms = *newly_added_transform_indexes.first().unwrap() as u64;
            let buffer_copy_info = BufferCopy {
                src_offset: first_index_of_newly_added_transforms,
                dst_offset: first_index_of_newly_added_transforms,
                size: newly_added_transform_indexes.len() as u64,
                ..Default::default()
            };
            let mut small_vec = Vec::new();
            small_vec.push(buffer_copy_info);
            copy_info.regions = small_vec.into();
            builder.copy_buffer(copy_info).unwrap();
        }
        builder
    }
    
    fn build_synch_vertex_buffers_commands<'a>(&'a mut self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, unsynched_ahead_buffer_index: usize, first_vertex_index: usize, last_vertex_index: usize) -> &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        let (most_up_to_date_buffer, buffers_to_update) = self.vertex_buffers.get_synch_info(unsynched_ahead_buffer_index);
        for buffer_to_update in buffers_to_update.iter() {
            let mut copy_info = CopyBufferInfo::buffers(most_up_to_date_buffer.clone(), buffer_to_update.clone());
            let buffer_copy_info = BufferCopy {
                src_offset: first_vertex_index as u64,
                dst_offset: first_vertex_index as u64,
                size: (last_vertex_index - first_vertex_index) as u64,
                ..Default::default()
            };
            let mut small_vec = Vec::new();
            small_vec.push(buffer_copy_info);
            copy_info.regions = small_vec.into();
            builder.copy_buffer(copy_info).unwrap();
        }
        builder
    }

    pub fn get_command_buffer(&mut self, acquired_swapchain_image: usize) -> Arc<PrimaryAutoCommandBuffer> {
        self.build_command_buffer(acquired_swapchain_image)
    }
}
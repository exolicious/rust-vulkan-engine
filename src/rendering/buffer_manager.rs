use std::{borrow::Borrow, cell::RefCell, collections::HashMap, mem::size_of, sync::Arc};
use egui_winit_vulkano::egui::{epaint::{self, Primitive}, ClippedPrimitive};
use glam::Mat4;
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, BufferCopy, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SecondaryAutoCommandBuffer, SubpassBeginInfo, SubpassContents, SubpassEndInfo}, descriptor_set::{allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, CopyDescriptorSet, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint}, render_pass::{Framebuffer, RenderPass, RenderPassCreateInfo, Subpass}};
use crate::{engine::camera::Camera, physics::physics_traits::Transform};
use super::{frame::Frame, primitives::Mesh, transform_buffers::TransformBuffers, vertex_buffers::VertexBuffer};
use std::error::Error;
use core::fmt::Error as ErrorVal;
use vulkano::format::Format;


pub struct BufferManager {
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub vertex_buffer: VertexBuffer,
    pub frames: Vec<Frame>,
    queue_family_index: u32,
    pub transform_buffers: RefCell<TransformBuffers>,
    vp_camera_buffers: Vec<Subbuffer<[[f32; 4]; 4]>>, // needs to be a push constant sooner or later
    pub entities_transform_ids: Vec<String>,
    pub entites_to_update: HashMap<String, Transform>,
    pipeline: Arc<GraphicsPipeline>,
    gui_image: Arc<Image>,
    pub gui_image_view: Arc<ImageView>,
}

impl BufferManager {
    pub fn new(device: Arc<Device>, pipeline: Arc<GraphicsPipeline>, swapchain_images: Vec<Arc<Image>>, render_pass: Arc<RenderPass>, queue_family_index: u32) -> Self {
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
            device.clone(), 
            StandardDescriptorSetAllocatorCreateInfo::default()
        );

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo {
                secondary_buffer_count: 32,
                ..Default::default()
            }
        );

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let frames = BufferManager::build_frames(device.clone(), pipeline.clone(), swapchain_images.clone(), render_pass.clone(), queue_family_index);
        let vertex_buffer = VertexBuffer::new(memory_allocator.clone());
        let transform_buffers = RefCell::new(TransformBuffers::new(memory_allocator.clone(), swapchain_images.len()));
        let vp_camera_buffers = Self::initialize_vp_camera_buffers(memory_allocator.clone(), swapchain_images.len());

        let entities_transform_ids = Vec::new();
        let entites_to_update = HashMap::new();

        let gui_image: Arc<Image> = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [150, 150, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::SAMPLED | ImageUsage::COLOR_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .unwrap();
        let gui_image_view = ImageView::new_default(gui_image.clone()).unwrap();

        Self {
            vertex_buffer,
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
            gui_image,
            gui_image_view
        }
    }

    //has to be called again, when its buffers are out of date (re-allocated due to being too small), or when the swapchain gets updated (window gets resized, or old swapchain was suboptimal )
    pub fn build_frames(device: Arc<Device>, pipeline: Arc<GraphicsPipeline>, swapchain_images: Vec<Arc<Image>>, render_pass: Arc<RenderPass>, queue_family_index: u32) -> Vec<Frame> {
        let mut temp_frames = Vec::new();
        for (swapchain_image_index, swapchain_image) in swapchain_images.iter().enumerate() {
            let mut temp_frame = Frame::new(
                swapchain_image.clone(), 
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
        let projection_view_matrix: Mat4 = Mat4::IDENTITY;
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
                projection_view_matrix.to_cols_array_2d(),
            )
            .unwrap();
            vp_matrix_buffers.push(uniform_buffer);
        }
        vp_matrix_buffers
    }

    pub fn register_entity(&mut self, entity_transform: Transform, entity_mesh: Mesh, next_swapchain_image_index: usize, entity_index: usize) -> Result<(), Box<dyn Error>> {
        println!("Trying to register entity in frame {}", next_swapchain_image_index);
        self.vertex_buffer.bind_entity_mesh(entity_mesh, next_swapchain_image_index)?;
        self.transform_buffers.borrow_mut().bind_entity_transform(entity_transform, entity_index, next_swapchain_image_index).unwrap();
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

    pub fn copy_transform_data_slice_to_buffer(& self, entity_transforms_first_index: usize, entity_transforms_last_index: usize, entity_model_matrices: &[[[f32; 4]; 4]], next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let binding = self.transform_buffers.borrow();
        let mut write_lock =  binding[next_swapchain_image_index].write()?;
        write_lock[entity_transforms_first_index..entity_transforms_last_index].copy_from_slice(entity_model_matrices);
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn update_entity_transform_buffer(& self, entity_id: &String, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        println!("entity id: {entity_id}");
        match self.entities_transform_ids.iter().position(|existing_entity_id| existing_entity_id == entity_id) {
            Some(entity_transform_index) => {
                let binding = self.transform_buffers.borrow();
                println!("entity transform index: {entity_transform_index}");
                binding.borrow().update_entity_transform(entity_transform_index, entity_transform, next_swapchain_image_index)?;
                Ok(())
            }
            None => Err(Box::new(ErrorVal))
        }
    }

    pub fn copy_vp_camera_data(& self, camera: &Camera, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        println!("{:?}", camera.projection_view_matrix);
        let mut write_lock = self.vp_camera_buffers[0].write()?;
        *write_lock = camera.projection_view_matrix.to_cols_array_2d();
        write_lock = self.vp_camera_buffers[1].write()?;
        *write_lock = camera.projection_view_matrix.to_cols_array_2d();
        write_lock = self.vp_camera_buffers[2].write()?;
        *write_lock = camera.projection_view_matrix.to_cols_array_2d();
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
            [WriteDescriptorSet::buffer(0, self.transform_buffers.borrow()[next_swapchain_image_index].clone())],
            []
        )
        .unwrap()
    }

    pub fn build_command_buffer(& self, acquired_swapchain_image: usize, gui_command_buffer: Arc<SecondaryAutoCommandBuffer>) -> Arc<PrimaryAutoCommandBuffer> {
        //println!("Bulding command buffer for index: {}", acquired_swapchain_image);
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        ) 
        .unwrap();

        let mut descriptor_sets = Vec::new();
        descriptor_sets.push(self.get_vp_matrix_buffer_descriptor_set(acquired_swapchain_image).clone());
        descriptor_sets.push(self.get_transform_buffer_descriptor_set(acquired_swapchain_image).clone());
        
        let builder = self.copy_transform_buffer_data(& mut command_buffer_builder, acquired_swapchain_image);
        
        let vertex_buffer = self.vertex_buffer.vertex_buffer.clone();
        // println!("Vertex buffer with index {acquired_swapchain_image} has the following data {a}");
        let builder = builder
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
        
        for mesh in self.vertex_buffer.mesh_accessor.meshes.iter() {
            let instances_count = self.vertex_buffer.mesh_accessor.mesh_name_instance_count_map.get(&mesh.name).unwrap();
            let vertex_count = mesh.data.len() as u32;
            let meshes_first_vertex_index = self.vertex_buffer.mesh_accessor.mesh_name_first_vertex_index_map.get(&mesh.name).unwrap();
            //println!("adding draw call for mesh \n instance count: {} \n vertex count: {}", instances_count, vertex_count);
            builder
                .draw(vertex_count, *instances_count as u32, *meshes_first_vertex_index as u32, 0)
                .unwrap();
            //println!("added draw call to command buffer successfully");
        }

        builder
            .next_subpass(Default::default(), SubpassBeginInfo {
                contents: SubpassContents::SecondaryCommandBuffers,
                ..Default::default()
            }
            )
            .unwrap();
        println!("tryna add execution of gui secondary command buffer in subpass");
        builder
        .execute_commands(gui_command_buffer)
        .unwrap()
        .end_render_pass(SubpassEndInfo::default())
        .unwrap();
        
        let command_buffer = command_buffer_builder.build().unwrap();
        
        command_buffer
    }

    //todo: make this work with fragmented buffers...
    fn copy_transform_buffer_data<'a>(&'a self, builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>, acquired_swapchain_image: usize) -> & mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        match self.transform_buffers.borrow_mut().get_tansform_buffer_copy_payload(acquired_swapchain_image) {
            None => return builder,
            Some(payload) => {
                println!("DOING TRANSFORM BUFFER SYNC");
                for target_buffer in payload.target_buffers.iter() {
                    let mut copy_info = CopyBufferInfo::buffers(payload.src_buffer.clone(), target_buffer.clone());
                    let first_index_of_newly_added_transforms = *payload.newly_added_transform_indexes.first().unwrap() as u64;
                    println!("SIZE OF TRANSFORM IN BYTES : {}", size_of::<Mat4>() as u64);
                    copy_info.regions[0].src_offset = first_index_of_newly_added_transforms * size_of::<Mat4>() as u64;
                    copy_info.regions[0].dst_offset = first_index_of_newly_added_transforms * size_of::<Mat4>() as u64;
                    copy_info.regions[0].size = (size_of::<Mat4>() * payload.newly_added_transform_indexes.len()) as u64;
                    builder.copy_buffer(copy_info).unwrap();
                }
                builder
            }
        }
    }
}
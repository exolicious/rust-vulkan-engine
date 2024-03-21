use std::{sync::Arc, cell::RefCell};

use vulkano::{command_buffer::{allocator::{CommandBufferAllocator, StandardCommandBufferAllocator}, AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents, SubpassEndInfo}, device::Device, image::{view::ImageView, Image}, pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}};
use winit::window::Window;

use super::{buffer_manager::BufferManager};

pub struct Frame {
    swapchain_image: Arc<Image>,
    pub swapchain_image_view: Arc<ImageView>,
    device: Arc<Device>, 
    swapchain_image_index: usize,
    pipeline: Arc<GraphicsPipeline>, 
    framebuffer: Option<Arc<Framebuffer>>,
    pub draw_command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>,
}

impl Frame {
    pub fn new(swapchain_image: Arc<Image>, device: Arc<Device>, pipeline: Arc<GraphicsPipeline>, swapchain_image_index: usize) -> Self {
        let swapchain_image_view =  ImageView::new_default(swapchain_image.clone()).unwrap();
        Self {
            swapchain_image,    
            swapchain_image_view,
            device,
            swapchain_image_index,
            pipeline,
            framebuffer: None,
            draw_command_buffer: None,
        }
    }

    pub fn init(&mut self, render_pass: Arc<RenderPass>, active_queue_family_index: u32, buffer_manager: &BufferManager) {
        self.init_framebuffer(render_pass);
        self.init_command_buffer(active_queue_family_index, buffer_manager);
    }

    fn init_framebuffer(&mut self, render_pass: Arc<RenderPass>) -> () {
        let view = ImageView::new_default(self.swapchain_image.clone()).unwrap();
        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            })
            .unwrap();
        self.framebuffer = Some(framebuffer);
    }

    pub fn init_command_buffer(&mut self, active_queue_family_index: u32, buffer_manager: &BufferManager) -> () {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &buffer_manager.command_buffer_allocator,
            active_queue_family_index,
            CommandBufferUsage::MultipleSubmit,
        ) 
        .unwrap();

        let mut descriptor_sets = Vec::new();
        descriptor_sets.push(buffer_manager.get_vp_matrix_buffer_descriptor_set(self.pipeline.clone(), self.swapchain_image_index).clone());
        descriptor_sets.push(buffer_manager.get_transform_buffer_descriptor_set(self.pipeline.clone(), self.swapchain_image_index).clone());
        
        let vertex_buffer = buffer_manager.vertex_buffers[self.swapchain_image_index].clone();

        let builder = command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffer.as_ref().unwrap().clone())
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
            );

            if buffer_manager.mesh_accessors.len() > 0 {
                let builder = buffer_manager.mesh_accessor.meshes.iter().fold(builder, |builder, mesh_accessor| {
                    println!("instance count: {}, first index: {}, last index: {}", mesh_accessor.instance_count, mesh_accessor.first_index, mesh_accessor.last_index);
                    builder
                        .unwrap()
                        .draw(mesh_accessor.meshes.try_into().unwrap(), mesh_accessor.instance_count.try_into().unwrap(), mesh_accessor.first_index.try_into().unwrap(), mesh_accessor.first_instance.try_into().unwrap())
                });
                builder
                .unwrap()
                .end_render_pass(SubpassEndInfo::default())
                .unwrap();
            }
            else {
                builder
                .unwrap()
                .end_render_pass(SubpassEndInfo::default())
                .unwrap();
            }
        let command_buffer = command_buffer_builder.build().unwrap();
        self.draw_command_buffer = Some(command_buffer);
    }
}
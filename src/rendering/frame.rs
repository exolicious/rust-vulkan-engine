use std::{sync::Arc, cell::RefCell};

use vulkano::{image::{SwapchainImage, view::ImageView}, render_pass::{Framebuffer, RenderPass, FramebufferCreateInfo}, device::Device, 
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, CommandBufferUsage, RenderPassBeginInfo, SubpassContents}, 
    pipeline::{GraphicsPipeline, PipelineBindPoint, Pipeline, graphics::render_pass}};
use winit::window::Window;

use super::{buffer_manager::BufferManager};

pub struct Frame {
    swapchain_image: Arc<SwapchainImage<Window>>,
    device: Arc<Device>, 
    swapchain_image_index: usize,
    pipeline: Arc<GraphicsPipeline>, 
    framebuffer: Option<Arc<Framebuffer>>,
    pub draw_command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>,
    buffer_manager: Arc<RefCell<BufferManager>>
}

impl Frame {
    pub fn new(swapchain_image: Arc<SwapchainImage<Window>>, device: Arc<Device>, pipeline: Arc<GraphicsPipeline>, buffer_manager: Arc<RefCell<BufferManager>>, swapchain_image_index: usize) -> Self {
        Self {
            swapchain_image,    
            device,
            swapchain_image_index,
            pipeline,
            buffer_manager,
            framebuffer: None,
            draw_command_buffer: None,
        }
    }

    pub fn init(&mut self, render_pass: Arc<RenderPass>, active_queue_family_index: u32) {
        self.init_framebuffer(render_pass);
        self.init_draw_command_buffer(active_queue_family_index);
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

    pub fn init_draw_command_buffer(&mut self, active_queue_family_index: u32) -> () {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            active_queue_family_index,
            CommandBufferUsage::MultipleSubmit,
        ) 
        .unwrap();

        let mut descriptor_sets = Vec::new();
        descriptor_sets.push(self.buffer_manager.borrow().get_vp_matrix_buffer_descriptor_set(self.pipeline.clone(), self.swapchain_image_index).clone());
        descriptor_sets.push(self.buffer_manager.borrow().get_transform_buffer_descriptor_set(self.pipeline.clone(), self.swapchain_image_index).clone());
        
        let vertex_buffer = self.buffer_manager.borrow().vertex_buffers[self.swapchain_image_index].clone();

        let builder = command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffer.as_ref().unwrap().clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, vertex_buffer)
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_sets,
            );

            if self.buffer_manager.borrow().mesh_accessors.len() > 0 {
                let builder = self.buffer_manager.borrow().mesh_accessors.iter().fold(builder, |builder, mesh_accessor| {
                    //println!("instance count: {}, first index: {}, last index: {}", mesh_accessor.instance_count, mesh_accessor.first_index, mesh_accessor.last_index);
                    builder
                        .draw(mesh_accessor.vertex_count.try_into().unwrap(), mesh_accessor.instance_count.try_into().unwrap(), mesh_accessor.first_index.try_into().unwrap(), mesh_accessor.first_instance.try_into().unwrap())
                        .unwrap()
                });
                builder
                .end_render_pass()
                .unwrap();
            }
            else {
                builder
                .draw(0, 0, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();
            }
        let command_buffer = Arc::new(command_buffer_builder.build().unwrap());
        self.draw_command_buffer = Some(command_buffer);
    }
}
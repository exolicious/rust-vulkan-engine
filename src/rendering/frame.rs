use std::{sync::Arc};

use image::buffer;
use vulkano::{command_buffer::{allocator::{CommandBufferAllocator, StandardCommandBufferAllocator}, AutoCommandBufferBuilder, BufferCopy, CommandBufferUsage, CopyBufferInfo, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassContents, SubpassEndInfo}, device::Device, image::{view::ImageView, Image}, pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint}, render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass}, NonExhaustive, ValidationError};
use winit::window::Window;

use super::{buffer_manager::BufferManager, mesh_accessor};

pub struct Frame {
    swapchain_image: Arc<Image>,
    pub swapchain_image_view: Arc<ImageView>,
    device: Arc<Device>, 
    swapchain_image_index: usize,
    pub framebuffer: Option<Arc<Framebuffer>>,
    pub draw_command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>,
}

impl Frame {
    pub fn new(swapchain_image: Arc<Image>, device: Arc<Device>, swapchain_image_index: usize) -> Self {
        let swapchain_image_view =  ImageView::new_default(swapchain_image.clone()).unwrap();
        Self {
            swapchain_image,    
            swapchain_image_view,
            device,
            swapchain_image_index,
            framebuffer: None,
            draw_command_buffer: None,
        }
    }

    pub fn init_framebuffer(&mut self, render_pass: Arc<RenderPass>) -> () {
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
}
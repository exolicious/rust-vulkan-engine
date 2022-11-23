use std::{sync::Arc, cell::RefCell};

use vulkano::{image::{SwapchainImage, view::ImageView}, render_pass::{Framebuffer, RenderPass, FramebufferCreateInfo}, device::Device, 
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, CommandBufferUsage, RenderPassBeginInfo, SubpassContents}, 
    pipeline::{GraphicsPipeline, PipelineBindPoint, Pipeline}};
use winit::window::Window;

use super::{buffer_manager::BufferManager};

pub struct Frame {
    swapchain_image: Arc<SwapchainImage<Window>>,
    render_pass: Arc<RenderPass>,
    device: Arc<Device>, 
    active_queue_family_index: u32, 
    swapchain_image_index: usize,
    pipeline: Arc<GraphicsPipeline>, 
    framebuffer: Option<Arc<Framebuffer>>,
    pub command_buffer: Option<Arc<PrimaryAutoCommandBuffer>>,
    buffer_manager: Arc<RefCell<BufferManager>>
}

impl Frame {
    pub fn new(swapchain_image: Arc<SwapchainImage<Window>>, render_pass: Arc<RenderPass>, device: Arc<Device>, active_queue_family_index: u32, 
            pipeline: Arc<GraphicsPipeline>, buffer_manager: Arc<RefCell<BufferManager>>, swapchain_image_index: usize) -> Self {
        Self {
            swapchain_image,    
            render_pass,
            device,
            active_queue_family_index,
            swapchain_image_index,
            pipeline,
            buffer_manager,
            framebuffer: None,
            command_buffer: None,
        }
    }

    pub fn init(&mut self) {
        self.init_framebuffer();
        self.init_command_buffer();
    }

    fn init_framebuffer(&mut self) -> () {
        let view = ImageView::new_default(self.swapchain_image.clone()).unwrap();
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            })
            .unwrap();
        self.framebuffer = Some(framebuffer);
    }

    pub fn init_command_buffer(&mut self) -> () {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.active_queue_family_index,
            CommandBufferUsage::MultipleSubmit,
        ) 
        .unwrap();

        let mut descriptor_sets = Vec::new();
        descriptor_sets.push(self.buffer_manager.borrow().get_vp_matrix_buffer_descriptor_set(self.pipeline.clone(), self.swapchain_image_index).clone());
        descriptor_sets.push(self.buffer_manager.borrow().get_transform_buffer_descriptor_set(self.pipeline.clone(), self.swapchain_image_index).clone());
        
        let vertex_buffer = self.buffer_manager.borrow().vertex_buffers[self.swapchain_image_index].clone();
        
      /*   match vertex_buffer.read() {
            Ok(read_lock) => println!("{:?}",read_lock),
            Err(_) => ()
        } */

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

            if self.buffer_manager.borrow().blueprint_accessors.len() > 0 {
                let builder = self.buffer_manager.borrow().blueprint_accessors.iter().fold(builder, |builder, blueprint_accessor| {
                    println!("entity count : {}", self.buffer_manager.borrow().blueprint_accessors[0].instance_count);
                    println!("Vertex count : {}", self.buffer_manager.borrow().blueprint_accessors[0].vertex_count);
                    builder
                        .draw(blueprint_accessor.vertex_count.try_into().unwrap(), blueprint_accessor.instance_count.try_into().unwrap(), 0, 0)
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
        self.command_buffer = Some(command_buffer);
    }
}
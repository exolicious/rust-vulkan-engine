use std::sync::Arc;

use vulkano::image::view::ImageView;
use vulkano::image::Image;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};

pub struct Frame {
    pub swapchain_image_view: Arc<ImageView>,
    pub framebuffer: Arc<Framebuffer>,
}

impl Frame {
    pub fn new(swapchain_image: Arc<Image>, render_pass: Arc<RenderPass>) -> Self {
        let swapchain_image_view = ImageView::new_default(swapchain_image).unwrap();
        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![swapchain_image_view.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        Self {
            swapchain_image_view,
            framebuffer,
        }
    }
}

use std::sync::Arc;
use vulkano::{instance::{Instance, InstanceCreateInfo}, swapchain::Surface};
use winit::event_loop::EventLoop;

pub fn get_vulkan_instance(event_loop: &EventLoop<()>) -> Arc<Instance> {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let required_extensions = Surface::required_extensions(&event_loop);
    return Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");
}

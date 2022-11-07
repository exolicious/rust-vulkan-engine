use std::sync::Arc;
use vulkano::instance::{Instance, InstanceCreateInfo};

pub fn get_vulkan_instance() -> Arc<Instance> {
    let library = vulkano::VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let required_extensions = vulkano_win::required_extensions(&library);

    return Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create instance");
}


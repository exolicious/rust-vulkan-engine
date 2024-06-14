use std::sync::Arc;

use vulkano::{device::Device, shader::ShaderModule, Validated, VulkanError};

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 position;

            layout(set = 0, binding = 0) uniform UniformBufferObject {
                mat4 u_projection_view_matrix;
            } ubo;

            layout(set = 1, binding = 0) uniform TransformBufferObject {
                mat4 u_transform_matrix[1000000000];
            } tbo;
            
            void main() {
                gl_Position = ubo.u_projection_view_matrix * tbo.u_transform_matrix[gl_InstanceIndex] * vec4(position, 1.0);
            }",
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450
            layout(location = 0) out vec4 f_color;
            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }"
    }
}

pub struct Shaders {
    pub vertex_shader: Arc<ShaderModule>,
    pub fragment_shader: Arc<ShaderModule>,
}

impl Shaders {
    pub fn load(device: Arc<Device>) -> Result<Self, Validated<VulkanError>> {
        Ok(Self {
            vertex_shader: vertex_shader::load(device.clone())?,
            fragment_shader: fragment_shader::load(device.clone())?
        })
    }
}
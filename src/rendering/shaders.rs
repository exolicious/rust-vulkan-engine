use std::sync::Arc;

use vulkano::{shader::{ShaderModule, ShaderCreationError}, device::Device};

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(binding = 0) uniform UniformBufferObject {
                mat4 u_view_projection_matrix;
            } ubo;

            layout(binding = 1) uniform TransformBufferObject {
                mat4 u_transform_matrix;
            } tbo;
            
            void main() {
                gl_Position = ubo.u_view_projection_matrix * tbo.u_transform_matrix[0];
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
    pub fn load(device: Arc<Device>) -> Result<Self, ShaderCreationError> {
        Ok(Self {
            vertex_shader: vertex_shader::load(device.clone())?,
            fragment_shader: fragment_shader::load(device.clone())?
        })
    }
}
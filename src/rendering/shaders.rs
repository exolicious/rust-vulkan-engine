use std::sync::Arc;

use vulkano::{shader::{ShaderModule, ShaderCreationError}, device::Device};

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec3 position;
            void main() {
            gl_Position = vec4(position, 1.0);
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
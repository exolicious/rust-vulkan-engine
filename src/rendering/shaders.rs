use std::sync::Arc;

use vulkano::{device::Device, shader::ShaderModule, Validated, VulkanError};

mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;

            layout(set = 0, binding = 0) uniform ViewProjection {
                mat4 view_projection;
            };

            layout(set = 1, binding = 0) readonly buffer ModelMatrices {
                mat4 model[];
            };

            layout(location = 0) out vec3 v_normal;

            void main() {
                v_normal = mat3(model[gl_InstanceIndex]) * normal; //mat3 because we want to ignore translation and scale for normals
                gl_Position = view_projection * model[gl_InstanceIndex] * vec4(position, 1.0);
            }",
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) out vec4 f_color;
            layout(location = 0) in vec3 v_normal;

            void main() {
                float ambient = 0.5;
                vec3 light_dir = vec3(0, 1.0, 0.0);
                float diffuse = max(dot(normalize(v_normal), -light_dir), 0.0);

                f_color = vec4(1.0, 0.0, 0.0, 1.0) * (ambient + diffuse);
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
            fragment_shader: fragment_shader::load(device)?,
        })
    }
}

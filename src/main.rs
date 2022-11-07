use std::env;
use bytemuck::{Pod, Zeroable};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use winit::event_loop::{EventLoop};

use crate::initialize::vulkan_instancing::*;
pub mod initialize;

use crate::renderer::renderer::Renderer;
use crate::renderer::render_manager::RenderManager;
pub mod renderer;


#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec2 position;
            void main() {
            gl_Position = vec4(position, 0.0, 1.0);
            }"
    }
}

mod fs {
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

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    
    let instance = get_vulkan_instance();

    let event_loop = EventLoop::new();
    let mut renderer = Renderer::new(instance, &event_loop);
    
    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };

    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };

    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };
    
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        renderer.device.clone(),
        BufferUsage {
            vertex_buffer: true,
            ..Default::default()
        },
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

    let vertex_shader = vs::load(renderer.device.clone()).expect("failed to create shader module");
    let fragment_shader = fs::load(renderer.device.clone()).expect("failed to create shader module");

    renderer.init_shaders(vertex_shader.clone(), fragment_shader.clone());
    renderer.create_pipeline();
    renderer.init_vertex_buffers(vertex_buffer.clone());
    renderer.create_command_buffers();

    RenderManager::start_renderer(renderer, event_loop);

}

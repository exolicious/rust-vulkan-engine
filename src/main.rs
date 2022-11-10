use std::env;
use rendering::primitives::Vertex;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};

pub mod initialize;

pub mod rendering;
use crate::rendering::renderer::Renderer;
use crate::rendering::render_manager::RenderManager;
use crate::rendering::shaders::Shaders;

use crate::rendering::primitives::{Triangle, Cube, Mesh};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    
    let mut render_manager = RenderManager::new();

    let mut cube = Cube::new([0.5,0.5,0.5], [0.,0.,0.]);
    cube.generate_mesh();

    let vertex1 = Vertex {
        position: [-0.9, -0.9, 0.0],
    };

    let vertex2 = Vertex {
        position: [0.5, -0.25, 0.0],
    };

    let vertex3 = Vertex {
        position: [0.0, 0.5, 0.0],
    };

    let triangle = Triangle::new(vertex1, vertex2, vertex3);

    
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        render_manager.renderer.device.clone(),
        BufferUsage {
            vertex_buffer: true,
            ..Default::default()
        },
        false,
        cube.unwrap_vertices().into_iter(),
    )
    .unwrap();

    let shaders = Shaders::load(render_manager.renderer.device.clone()).unwrap();

    render_manager.renderer.init_shaders(shaders.vertex_shader.clone(), shaders.fragment_shader.clone());
    render_manager.renderer.create_pipeline();
    render_manager.renderer.init_vertex_buffers(vertex_buffer.clone());
    render_manager.renderer.create_command_buffers();

    render_manager.start_renderer();

}

use std::sync::Arc;

use crate::{physics::physics_traits::{Transform, Movable}, engine::general_traits::{Update, UniformBufferOwner}, rendering::renderer::Renderer};

use cgmath::{Vector3, Matrix4, perspective, SquareMatrix, Deg};
use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage}, swapchain::Surface};
use winit::window::Window;

pub struct Camera {
    transform: Transform,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    pub projection_view_matrix: Matrix4<f32>,
    uniform_buffer: Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>>
}

impl Camera {
    pub fn new(renderer: &Renderer<Surface<Window>>) -> Self {
        let projection_view_matrix = Matrix4::identity();
        let uniform_buffer: Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>> = CpuAccessibleBuffer::from_data(
            renderer.device.clone(),
            BufferUsage {
                uniform_buffer: true,
                ..Default::default()
            },
            false,
            projection_view_matrix.into(),
        )
        .unwrap();
        Self {
            transform: Transform { ..Default::default() },
            projection_matrix: perspective(Deg{0: 45.}, 16./9. , 1., 1000.),
            view_matrix: Matrix4::identity(),
            projection_view_matrix: projection_view_matrix,
            uniform_buffer
        }
    }

    pub fn recalculate_view_matrix(&mut self) -> () {
        let translation_matrix = Matrix4::from_translation(self.transform.position);
        //println!("translation Matrix: {:?}", translation_matrix);
        let orientation_matrix = Matrix4::from_axis_angle(self.transform.rotation.v, Deg {0: self.transform.rotation.s});
        //println!("orientation Matrix: {:?}", orientation_matrix);
        self.view_matrix = (translation_matrix * orientation_matrix).invert().unwrap();
        self.projection_view_matrix = self.projection_matrix * self.view_matrix;
    }

    pub fn flush_uniform_buffer(& mut self) {
        match self.uniform_buffer.write() {
            Err(_) => print!("Error"),
            Ok(mut write_lock) => { 
                print!("Success");
                *write_lock = self.projection_view_matrix.into();
            }
        };
    }
}

impl Update for Camera {
    fn update(& mut self) -> () {
        //self.move_y(0.000005);
        self.flush_uniform_buffer();
    }
}

impl Movable for Camera {
    fn on_move(& mut self) {
        self.recalculate_view_matrix();
    }

    fn move_xyz(& mut self, amount: Vector3<f32>) -> () {
        self.move_x(amount.x);
        self.move_y(amount.y);
        self.move_z(amount.z);
        self.on_move();
    }
    fn move_x(& mut self, amount: f32) -> () {
        self.transform.position.x += amount;
        self.on_move();
    }
    fn move_y(& mut self, amount: f32) -> () {
        self.transform.position.y += amount;
        self.on_move();
    }
    fn move_z(& mut self, amount: f32) -> () {
        self.transform.position.z += amount;
        self.on_move();
    }
}

impl UniformBufferOwner<[[f32; 4]; 4]> for Camera {
    fn get_uniform_buffer(& self) -> Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>> {
        self.uniform_buffer.clone()
    }
}
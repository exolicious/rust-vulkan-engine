use crate::{physics::physics_traits::{Transform, Movable}, engine::general_traits::Entity};

use glam::{Mat4, Vec3};
use nanoid::nanoid;
use vulkano::buffer::view;

use super::general_traits::{TickAction};

#[derive(Debug, Clone)]
pub struct Camera {
    transform: Transform,
    projection_matrix: Mat4,
    view_matrix: Mat4,
    pub projection_view_matrix: Mat4,
    id: String
}

impl Camera {
    pub fn new(transform: Transform, projection_matrix: Mat4) -> Self {
        println!("Camera translation: {:?}", transform.translation);
        let translation_matrix = Mat4::from_translation(transform.translation);
        println!("Camera translation amtrix : {:?}", translation_matrix);
        
        let orientation_matrix = Mat4::from_quat(transform.rotation);
        println!("quat angle: {:?}", transform.rotation.to_axis_angle().1);
        println!("orientation amtrix : {:?}", orientation_matrix);
        let view_matrix = (translation_matrix * orientation_matrix).inverse();
        println!("view matrix: {:?}", view_matrix);
        let projection_view_matrix =  projection_matrix * view_matrix;
        println!("projection_view_matrix matrix : {:?}", projection_view_matrix);
        Self {
            transform: transform,
            projection_matrix: projection_matrix,
            view_matrix: view_matrix,
            projection_view_matrix,
            id: nanoid!()
        }
    }

    pub fn recalculate_projection_view_matrix(&mut self) -> () {
        let translation_matrix = Mat4::from_translation(self.transform.translation);
        let orientation_matrix = Mat4::from_axis_angle(self.transform.rotation.xyz().normalize(), self.transform.rotation.to_axis_angle().1);

        self.view_matrix = (translation_matrix * orientation_matrix).inverse();
        self.projection_view_matrix = self.projection_matrix * self.view_matrix;
    }
}

impl Entity for Camera {
    fn tick(self: &mut Camera) -> Option<TickAction> {
        None
    }
}

impl Movable for Camera {
    fn update_position(&mut self) -> () {
        self.move_y(0.005);
    }

    fn on_move(& mut self) -> () {
        self.recalculate_projection_view_matrix();
    }

    fn move_xyz(& mut self, amount: Vec3) -> () {
        self.move_x(amount.x);
        self.move_y(amount.y);
        self.move_z(amount.z);
        self.on_move();
    }
    fn move_x(& mut self, amount: f32) -> () {
        self.transform.translation.x += amount;
        self.on_move();
    }
    fn move_y(& mut self, amount: f32) -> () {
        self.transform.translation.y += amount;
        self.on_move();
    }
    fn move_z(& mut self, amount: f32) -> () {
        self.transform.translation.z += amount;
        self.on_move();
    }
}


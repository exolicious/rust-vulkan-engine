use glam::{Mat4, Vec3};

use crate::physics::Transform;

#[derive(Debug, Clone)]
pub struct Camera {
    transform: Transform,
    projection_matrix: Mat4,
    pub projection_view_matrix: Mat4,
}

impl Camera {
    pub fn new(transform: Transform, projection_matrix: Mat4) -> Self {
        let mut camera = Self {
            transform,
            projection_matrix,
            projection_view_matrix: Mat4::IDENTITY,
        };
        camera.recalculate_projection_view_matrix();
        camera
    }

    pub fn translate(&mut self, delta: Vec3) {
        self.transform.translation += delta;
        self.recalculate_projection_view_matrix();
    }

    fn recalculate_projection_view_matrix(&mut self) {
        let view_matrix = (Mat4::from_translation(self.transform.translation)
            * Mat4::from_quat(self.transform.rotation))
        .inverse();
        self.projection_view_matrix = self.projection_matrix * view_matrix;
    }
}

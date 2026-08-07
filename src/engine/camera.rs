use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::physics::Transform;

#[derive(Debug, Clone)]
pub struct Camera {
    pub transform: Transform,
    yaw: f32,
    pitch: f32,
    projection_matrix: Mat4,
    pub projection_view_matrix: Mat4,
}

impl Camera {
    const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.02;

    pub fn new(transform: Transform, projection_matrix: Mat4) -> Self {
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let mut camera = Self {
            transform,
            yaw,
            pitch,
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

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta.to_radians();
        self.pitch = (self.pitch - pitch_delta.to_radians())
            .clamp(-Self::PITCH_LIMIT, Self::PITCH_LIMIT);
        self.transform.rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);
        self.recalculate_projection_view_matrix();
    }

    fn recalculate_projection_view_matrix(&mut self) {
        let view_matrix = (Mat4::from_translation(self.transform.translation)
            * Mat4::from_quat(self.transform.rotation))
        .inverse();
        self.projection_view_matrix = self.projection_matrix * view_matrix;
    }
}

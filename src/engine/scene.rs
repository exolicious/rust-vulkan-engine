use glam::{Mat4, Vec3};

use crate::{engine::point_light::PointLight, physics::Transform};

use super::camera::Camera;

pub struct Scene {
    pub camera: Camera,
    point_lights: Vec<Box<PointLight>>,
}

impl Scene {
    pub fn new() -> Self {
        let transform = Transform {
            translation: Vec3::new(0., 0., -5.),
            ..Default::default()
        };
        let projection_matrix = Mat4::perspective_lh(50_f32.to_radians(), 16. / 9., 1., 4000.);
        Self {
            camera: Camera::new(transform, projection_matrix),
            point_lights: Vec::new(),
        }
    }
}

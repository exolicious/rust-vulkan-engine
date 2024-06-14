use glam::{Mat4, Vec3};

use crate::physics::physics_traits::Transform;

use super::camera::Camera;

pub struct Scene {
    pub camera: Camera
}


impl Scene {
    pub fn new() -> Self {
        let transform = Transform { translation: Vec3 { x: 0., y: 0., z: -5. }, ..Default::default() };
        println!("initial camera transform: {:?}", transform);
        let projection_matrix = Mat4::perspective_lh(55., 16./9., 1., 4000.);
        
        let camera = Camera::new(transform, projection_matrix);

        Self {
            camera
        }
    }
}
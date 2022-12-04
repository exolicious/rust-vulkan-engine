use cgmath::{Vector3, perspective, Deg};

use crate::physics::physics_traits::Transform;

use super::camera::Camera;

pub struct Scene {
    pub camera: Camera
}


impl Scene {
    pub fn new() -> Self {
        let transform = Transform { translation: Vector3 { x: 0., y: 0., z: 2. }, ..Default::default() };
        let projection_matrix = perspective(Deg{ 0: 55.}, 16./9. , 1., 4000.);
        
        let camera = Camera::new(transform, projection_matrix);

        Self {
            camera
        }
    }
}
use cgmath::{Vector3, Quaternion, Matrix4, Deg};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new(translation: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>,) -> Self {
        Self {
            translation,
            rotation,
            scale
        }
    }

    pub fn model_matrix(&self) -> [[f32; 4]; 4] {
        let rotation_matrix = Matrix4::from_axis_angle(self.rotation.v, Deg {0: self.rotation.s});
        let scale_matrix = Matrix4::from_scale(1.);
        let translation_matrix = Matrix4::from_translation(self.translation);
        (translation_matrix * rotation_matrix * scale_matrix).into()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3 {x: 0., y: 0., z: 0. },
            rotation: Quaternion {v: Vector3 {x: 0., y: 0., z: 1.}, s: 0.},
            scale: Vector3 {x: 1., y: 1., z: 1.},
        }
    }
}


pub trait Movable {
    fn update_position(&mut self) -> ();
    fn on_move(&mut self) -> ();
    fn move_xyz(&mut self, amount: Vector3<f32>) -> ();
    fn move_x(&mut self, amount: f32) -> ();
    fn move_y(&mut self, amount: f32) -> ();
    fn move_z(&mut self, amount: f32) -> ();
}

pub trait HasTransform {
    fn get_transform(&self) -> &Transform;
}
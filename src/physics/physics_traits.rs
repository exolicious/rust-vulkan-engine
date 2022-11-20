use cgmath::{Vector3, Vector4, Quaternion, Matrix4};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vector3 {x: 0., y: 0., z: 2. },
            rotation: Quaternion {v: Vector3 {x: 0., y: 0., z: 1.}, s: 0.},
            scale: Vector3 {x: 1., y: 1., z: 1.},
        }
    }
}

impl Into<[[f32; 4];4]> for &Transform {
    fn into(self) -> [[f32; 4]; 4] {
        Matrix4::from_cols(
            Vector4 {x: self.translation.x, y: self.translation.y, z: self.translation.z, w: 0.}, 
            Vector4 {x: self.rotation.v.x, y: self.rotation.v.y, z: self.rotation.v.z, w: self.rotation.s}, 
            Vector4 {x: self.scale.x, y: self.scale.y, z: self.scale.z, w: 0.}, 
            Vector4 { x: 0., y: 0., z: 0., w: 0. }).into()
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
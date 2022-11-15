use cgmath::{Vector3, Quaternion};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vector3<f32>,
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
            position: Vector3 {x: 0., y: 0., z: 2. },
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
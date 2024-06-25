use glam::{Mat4, Quat, Vec3};



#[derive(Debug, Default, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale
        }
    }

    pub fn model_matrix(&self) -> [[f32; 4]; 4] {
        let rotation_matrix = Mat4::from_quat(self.rotation);
        let scale_matrix = Mat4::from_scale(Vec3{ x: 1. , y: 1., z: 1.});
        let translation_matrix = Mat4::from_translation( self.translation);
        println!("TRANSLATION MATRIX: {:?}", translation_matrix);
        let model_matrix = translation_matrix * rotation_matrix * scale_matrix;
        println!("MODEL MATRIX {:?}", model_matrix);
        
        // Ensure the model_matrix is converted properly to [[f32; 4]; 4]
        let model_array: [[f32; 4]; 4] = model_matrix.to_cols_array_2d();
        
        // Verify the array format
        for row in &model_array {
            println!("{:?}", row);
        }
        model_array
    }
}


pub trait Movable {
    fn update_position(&mut self) -> ();
    fn on_move(&mut self) -> ();
    fn move_xyz(&mut self, amount: Vec3) -> ();
    fn move_x(&mut self, amount: f32) -> ();
    fn move_y(&mut self, amount: f32) -> ();
    fn move_z(&mut self, amount: f32) -> ();
}

pub trait HasTransform {
    fn get_transform(&self) -> Transform;
}
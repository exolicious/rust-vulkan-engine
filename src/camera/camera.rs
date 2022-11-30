use crate::{physics::physics_traits::{Transform, Movable}, engine::general_traits::Entity};

use cgmath::{Vector3, Matrix4, perspective, SquareMatrix, Deg, InnerSpace};

use nanoid::nanoid;

#[derive(Debug, Clone)]
pub struct Camera {
    transform: Transform,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    pub projection_view_matrix: Matrix4<f32>,
    id: String
}

impl Camera {
    pub fn new() -> Self {
        let transform = Transform { translation: Vector3 { x: 0., y: 0., z: 2. }, ..Default::default() };
        let projection_matrix = perspective(Deg{ 0: 55.}, 16./9. , 1., 4000.);

        let translation_matrix = Matrix4::from_translation(transform.translation);
        println!("translation amtrix : {:?}", translation_matrix);
        let orientation_matrix = Matrix4::from_axis_angle(transform.rotation.v.normalize(), Deg { 0: transform.rotation.s });
        println!("orientation amtrix : {:?}", orientation_matrix);
        let view_matrix = (translation_matrix * orientation_matrix).invert().unwrap();
        let projection_view_matrix =  projection_matrix * view_matrix;
        println!("projection_view_matrix amtrix : {:?}", projection_view_matrix);
        Self {
            transform: transform,
            projection_matrix: projection_matrix,
            view_matrix: view_matrix,
            projection_view_matrix,
            id: nanoid!()
        }
    }

    pub fn recalculate_projection_view_matrix(&mut self) -> () {
        let translation_matrix = Matrix4::from_translation(self.transform.translation);
        let orientation_matrix = Matrix4::from_axis_angle(self.transform.rotation.v.normalize(), Deg {0: self.transform.rotation.s});

        self.view_matrix = (translation_matrix * orientation_matrix).invert().unwrap();
        self.projection_view_matrix = self.projection_matrix * self.view_matrix;
    }
}

impl Entity for Camera {
    fn get_id(&self) -> &String {
        &self.id
    }
    fn update(self: &mut Camera) -> () {
       return;
    }
}

impl Movable for Camera {
    fn update_position(&mut self) -> () {
        self.move_y(0.005);
    }

    fn on_move(& mut self) -> () {
        self.recalculate_projection_view_matrix();
    }

    fn move_xyz(& mut self, amount: Vector3<f32>) -> () {
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


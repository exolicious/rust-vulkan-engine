use std::{collections::hash_map::DefaultHasher, hash::Hasher, ops::{Deref, DerefMut}};

use bytemuck::{Zeroable, Pod};
use glam::Vec3;
use rand::Rng;
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex as VertexMacro};

use crate::{engine::general_traits::{Entity, TickAction}, physics::physics_traits::{HasTransform, Movable, Transform}};
use core::hash::Hash;
use super::rendering_traits::{HasMesh, RenderableEntity};

use nanoid::nanoid;



#[repr(C)]
#[derive(Default, Copy, Clone, Debug, BufferContents, VertexMacro)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    //pub color: [f32; 4]
}

impl Deref for Vertex {
    type Target = [f32; 3];

    fn deref(&self) -> &Self::Target {
        &self.position
    }

}

impl DerefMut for Vertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.position
    }
}


#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub data: Vec<Vertex>,
}

impl Mesh {
    pub fn new(data: Vec<Vertex>, name: String) -> Self {
        Self {
            name,
            data,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        let mut hasher = DefaultHasher::new();
        self.hash(& mut hasher) == other.hash(& mut hasher)
    }
}

impl Eq for Mesh {

}

impl Hash for Mesh {
    fn hash<H>(&self, state: &mut H) where H: Hasher { 
        let mut result = Vec::new();
        for triangle in &self.data {
            for j in triangle.position {
                let rounded_coord =  (j * 100_f32) as u8;
                result.push(rounded_coord);
            }
        }
        state.write(&result);
        state.finish();
    }
}

pub struct Triangle {
    pub vertices: [Vertex; 3]
}

impl Triangle {
    pub fn new(v1: Vertex, v2: Vertex, v3: Vertex) -> Self {
        Self {
            vertices: [v1, v2, v3]
        }
    }

    pub fn vertices(&self) -> Vec<f32> {
        let mut res = vec![0_f32;6];
        for vertex in self.vertices {
            res.append(& mut vertex.position.to_vec());
        }
        res
    }
}

#[derive(Debug, Clone)]
pub struct Cube {
    pub bounds: Vec3,
    transform: Transform,
    mesh: Option<Mesh>,
    id: String,
}

impl Cube {
    pub fn new(bounds: Vec3, transform: Transform) -> Self {
        Self {
            bounds,
            transform,
            mesh: None,
            id: nanoid!()
        }
    }
}

impl RenderableEntity for Cube {}

impl HasTransform for Cube {
    fn get_transform(&self) -> Transform {
        self.transform
    }
}

impl Entity for Cube {
    fn tick(&mut self) -> Option<TickAction> {
        let amount: f32 = rand::thread_rng().gen_range(-0.02_f32..0.02_f32);
        self.move_x(amount);
        Some(TickAction::HasMoved(self.transform))
    }
}

impl Default for Cube {
    fn default() -> Self {
        let bounds = Vec3 { x: 0.25, y: 0.125, z: 0.25 };
        
        Self {
            bounds : bounds,
            transform: Transform::default(),
            mesh: None,
            id: nanoid!()
        }
    }
}

impl Movable for Cube {
    fn update_position(&mut self) -> () {
        let amount: f32 = rand::thread_rng().gen_range(-2_f32..2_f32);
        self.move_x(amount);
    }

    fn move_xyz(&mut self, amount: Vec3) -> () {
        self.move_x(amount.x);
        self.move_y(amount.y);
        self.move_z(amount.z);
        self.on_move();
    }
    fn move_x(&mut self, amount: f32) -> () {
        self.transform.translation.x += amount;
        self.on_move();
    }
    fn move_y(&mut self, amount: f32) -> () {
        self.transform.translation.y += amount;
        self.on_move();
    }
    fn move_z(&mut self, amount: f32) -> () {
        self.transform.translation.z += amount;
        self.on_move();
    }

    fn on_move(&mut self) -> () {
        return
    }

}

impl HasMesh for Cube {
    fn get_mesh(&mut self, name: String) -> Mesh {
        let mut result = Vec::new();
        for triangle in self.get_data() {
            for i in 0..triangle.vertices.len() {
                result.push(triangle.vertices[i])
            }
        }
        Mesh::new(result, name)
    }

    fn get_data(& self) -> Vec<Triangle> {
        let mut resulting_mesh: Vec<Triangle> = Vec::new();
        let (x_bounds, y_bounds, z_bounds) = (self.bounds[0], self.bounds[1], self.bounds[2]);

        let temp = [[x_bounds/2., y_bounds/2., z_bounds/2.]; 8];
        let mut temp_vertices: Vec<Vertex> = Vec::new();

        //terrible, creates x,y,z arrays for the points in this order, right_top_front, left_top_front, right_bottom_front, 
        //right_top_back, right_bottom_back, left_top_back, left_bottom_front, left_bottom_back
        for i in 0..temp.len() {
            let mut temp_sub = temp[i];
            if i > 0 && i < 4 {
                temp_sub[i-1] = temp_sub[i-1] * -1.
            }
            else if i >= 4 && i <= 7 {
                for j in 0..temp_sub.len() {
                    temp_sub[j] = temp_sub[j] * -1.;
                }
                if i == 7 {
                    temp_vertices.push(Vertex{position: temp_sub});
                    break;
                }
                temp_sub[i - temp_sub.len()-1] = temp_sub[i - temp_sub.len()-1] * -1.
            }
            temp_vertices.push(Vertex{position: temp_sub});
        }

        let triangle_1 = Triangle{vertices: [temp_vertices[0], temp_vertices[3], temp_vertices[5]]};
        let triangle_2 = Triangle{vertices: [temp_vertices[5], temp_vertices[1], temp_vertices[0]]};
        let triangle_3 = Triangle{vertices: [temp_vertices[3], temp_vertices[4], temp_vertices[7]]};
        let triangle_4 = Triangle{vertices: [temp_vertices[7], temp_vertices[3], temp_vertices[5]]};
        let triangle_5 = Triangle{vertices: [temp_vertices[1], temp_vertices[5], temp_vertices[7]]};
        let triangle_6 = Triangle{vertices: [temp_vertices[6], temp_vertices[1], temp_vertices[7]]};
        let triangle_7 = Triangle{vertices: [temp_vertices[6], temp_vertices[0], temp_vertices[1]]};
        let triangle_8 = Triangle{vertices: [temp_vertices[2], temp_vertices[0], temp_vertices[6]]};
        let triangle_9 = Triangle{vertices: [temp_vertices[2], temp_vertices[3], temp_vertices[0]]};
        let triangle_10 = Triangle{vertices: [temp_vertices[4], temp_vertices[3], temp_vertices[2]]};
        let triangle_11 = Triangle{vertices: [temp_vertices[6], temp_vertices[2], temp_vertices[7]]};
        let triangle_12 = Triangle{vertices: [temp_vertices[2], temp_vertices[4], temp_vertices[7]]};
        
        resulting_mesh.push(triangle_1);
        resulting_mesh.push(triangle_2);
        resulting_mesh.push(triangle_3);
        resulting_mesh.push(triangle_4);
        resulting_mesh.push(triangle_5);
        resulting_mesh.push(triangle_6);
        resulting_mesh.push(triangle_7);
        resulting_mesh.push(triangle_8);
        resulting_mesh.push(triangle_9);
        resulting_mesh.push(triangle_10);
        resulting_mesh.push(triangle_11);
        resulting_mesh.push(triangle_12);

        resulting_mesh
    }
}
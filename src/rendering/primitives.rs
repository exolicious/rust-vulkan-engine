
use bytemuck::{Zeroable, Pod};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    //pub color: [f32; 4]
}
vulkano::impl_vertex!(Vertex, position);

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

trait BasicObject{}
impl BasicObject for Cube {}

pub struct Cube {
    pub bounds: [f32; 3], 
    pub position: [f32; 3],
    mesh: Option<Vec<Triangle>>
}

impl Cube {
    pub fn new (bounds: [f32; 3], position: [f32; 3]) -> Self {
        Self {
            bounds,
            position,
            mesh: None
        }
    }
}

impl Mesh for Cube {
    fn generate_mesh(& mut self) -> () {
        let mut resulting_mesh: Vec<Triangle> = Vec::new();
        let (x_bounds, y_bounds, z_bounds) = (self.bounds[0], self.bounds[1], self.bounds[2]);

        let temp = [[x_bounds/2., y_bounds/2., z_bounds/2.]; 8];
        let mut temp_vertices: Vec<Vertex> = Vec::new();

        //terrible, creates x,y,z arrays for the points in this order, right_top_front, left_top_front, right_bottom_front, 
        //right_top_back, right_bottom_back, left_top_back, left_bottom_front, left_bottom_back
        for i in 0..temp.len() {
            let mut temp_sub = temp[i];
            println!("Iteration number {}", i);
            if(i > 0 && i < 4) {
                temp_sub[i-1] = temp_sub[i-1] * -1.
            }
            else if(i >= 4 && i <= 7) {
                for j in 0..temp_sub.len() {
                    temp_sub[j] = temp_sub[j] * -1.;
                }
                if (i == 7) {
                    temp_vertices.push(Vertex{position: temp_sub});
                    break;
                }
                temp_sub[i - temp_sub.len()-1] = temp_sub[i - temp_sub.len()-1] * -1.
            }
            temp_vertices.push(Vertex{position: temp_sub});
        }
        print!("temp vertices: {:?}", temp_vertices);

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

        self.mesh = Some(resulting_mesh);

    }

    fn unwrap_vertices(&self) -> Vec<Vertex> {
        let mut result = Vec::new();
        for triangle in self.mesh.as_ref().unwrap() {
            for i in 0..triangle.vertices.len() {
                result.push(triangle.vertices[i])
            }
        }
        result
    }

    fn mesh_helper(&self) {}
    

}

pub trait Mesh {
    fn generate_mesh(& mut self) -> ();
    fn unwrap_vertices(&self) -> Vec<Vertex>;
    fn mesh_helper(&self) {}
}

use std::collections::HashMap;

use super::primitives::Mesh;

#[derive(Debug, Clone, Default)]
pub struct MeshAccessor {
    pub meshes: Vec<Mesh>,
    pub mesh_instance_count_map: HashMap<String, usize>,
    pub first_index: usize,
    pub first_instance: usize,
}

pub enum MeshAccessorAddEntityResult {
    AppendedToExistingMesh,
    CreatedNewMesh(Mesh)
}

impl MeshAccessor {
    pub fn new() {

    }

    pub fn add_entity(&mut self, entity_mesh: Mesh) -> MeshAccessorAddEntityResult {
        match self.mesh_instance_count_map.contains_key(&entity_mesh.name) {
            true => {
                *self.mesh_instance_count_map.get(&entity_mesh.name).unwrap() += 1;
                return MeshAccessorAddEntityResult::AppendedToExistingMesh;
            },
            false => { 
                self.add_new_mesh(entity_mesh);
                return MeshAccessorAddEntityResult::CreatedNewMesh(entity_mesh);
            }
        }
    }

    fn add_new_mesh(&mut self, entity_mesh: Mesh) {
        self.mesh_instance_count_map.insert(entity_mesh.name, 0usize);
        self.meshes.push(entity_mesh);
        
    }

    pub fn get_last_vertex_index(&self) -> usize {
        self.meshes.iter().fold(0, |acc, mesh| {
            mesh.data.iter().count()
        })
    }
}
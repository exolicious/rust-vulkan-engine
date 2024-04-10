use std::collections::HashMap;

use super::primitives::Mesh;

#[derive(Debug, Clone, Default)]
pub struct MeshAccessor {
    pub meshes: Vec<Mesh>,
    pub mesh_name_instance_count_map: HashMap<String, usize>,
    pub mesh_name_first_vertex_index_map: HashMap<String, usize>,
}

pub enum MeshAccessorAddEntityResult {
    AppendedToExistingMesh,
    CreatedNewMesh(Mesh)
}

impl MeshAccessor {
    pub fn new() -> Self {
        let meshes = Vec::new();
        let mesh_name_instance_count_map = HashMap::new();
        let mesh_name_first_vertex_index_map = HashMap::new();
        Self {
            meshes,
            mesh_name_instance_count_map,
            mesh_name_first_vertex_index_map
        }
    }

    pub fn add_entity(&mut self, entity_mesh: Mesh) -> MeshAccessorAddEntityResult {
        match self.mesh_name_instance_count_map.contains_key(entity_mesh.get_name()) {
            true => {
                *self.mesh_name_instance_count_map.get_mut(entity_mesh.get_name()).unwrap() += 1;
                return MeshAccessorAddEntityResult::AppendedToExistingMesh;
            },
            false => { 
                self.add_new_mesh(entity_mesh.clone());
                return MeshAccessorAddEntityResult::CreatedNewMesh(entity_mesh);
            }
        }
    }

    fn add_new_mesh(&mut self, entity_mesh: Mesh) {
        self.mesh_name_instance_count_map.insert(entity_mesh.get_name().to_string(), 0usize);
        self.meshes.push(entity_mesh);
    }

    pub fn get_last_vertex_index(&self) -> usize {
        self.meshes.iter().fold(0, |acc, mesh| {
            mesh.data.iter().count()
        })
    }
}
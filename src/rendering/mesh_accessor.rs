use super::primitives::Mesh;

pub struct MeshGroup {
    pub mesh: Mesh,
    pub first_vertex: usize,
    pub entity_indices: Vec<usize>,
}

#[derive(Default)]
pub struct MeshAccessor {
    pub groups: Vec<MeshGroup>,
    vertex_count: usize,
}

pub enum AddEntityResult {
    AppendedToExistingMesh,
    CreatedNewMesh { first_vertex: usize },
}

impl MeshAccessor {
    pub fn contains_mesh(&self, name: &str) -> bool {
        self.groups.iter().any(|group| group.mesh.name == name)
    }

    pub fn add_entity(&mut self, mesh: Mesh, entity_index: usize) -> AddEntityResult {
        if let Some(group) = self.groups.iter_mut().find(|group| group.mesh.name == mesh.name) {
            group.entity_indices.push(entity_index);
            return AddEntityResult::AppendedToExistingMesh;
        }
        let first_vertex = self.vertex_count;
        self.vertex_count += mesh.data.len();
        self.groups.push(MeshGroup {
            mesh,
            first_vertex,
            entity_indices: vec![entity_index],
        });
        AddEntityResult::CreatedNewMesh { first_vertex }
    }

    /// Entity indices in the order their transforms must be laid out in the
    /// transform buffer: grouped by mesh, matching the per-group draw calls.
    pub fn instance_order(&self) -> impl Iterator<Item = usize> + '_ {
        self.groups
            .iter()
            .flat_map(|group| group.entity_indices.iter().copied())
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
}

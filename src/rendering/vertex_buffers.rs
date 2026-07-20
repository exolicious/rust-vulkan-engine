use std::error::Error;
use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};

use super::mesh_accessor::{AddEntityResult, MeshAccessor};
use super::primitives::{Mesh, Vertex};

const VERTEX_BUFFER_CAPACITY: usize = 1 << 16;

pub struct VertexBuffer {
    pub buffer: Subbuffer<[Vertex]>,
    pub mesh_accessor: MeshAccessor,
    pending_uploads: Vec<(usize, Vec<Vertex>)>,
}

impl VertexBuffer {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        let buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![Vertex::default(); VERTEX_BUFFER_CAPACITY],
        )
        .unwrap();

        Self {
            buffer,
            mesh_accessor: MeshAccessor::default(),
            pending_uploads: Vec::new(),
        }
    }

    pub fn register_mesh_instance(
        &mut self,
        mesh: Mesh,
        entity_index: usize,
    ) -> Result<(), Box<dyn Error>> {
        // Check capacity before touching the accessor, so a failed
        // registration leaves no half-registered mesh group behind.
        let is_new_mesh = !self.mesh_accessor.contains_mesh(&mesh.name);
        if is_new_mesh && self.mesh_accessor.vertex_count() + mesh.data.len() > VERTEX_BUFFER_CAPACITY
        {
            return Err(format!(
                "vertex buffer capacity ({VERTEX_BUFFER_CAPACITY}) exceeded"
            )
            .into());
        }
        if let AddEntityResult::CreatedNewMesh { first_vertex } =
            self.mesh_accessor.add_entity(mesh, entity_index)
        {
            let data = self.mesh_accessor.groups.last().unwrap().mesh.data.clone();
            self.pending_uploads.push((first_vertex, data));
        }
        Ok(())
    }

    pub fn has_pending_uploads(&self) -> bool {
        !self.pending_uploads.is_empty()
    }

    /// Writes newly registered mesh data into the vertex buffer. The buffer is
    /// shared by all frames in flight, so the caller must make sure the GPU is
    /// not reading it (wait for all in-flight frame futures) before flushing.
    pub fn flush_pending_uploads(&mut self) -> Result<(), Box<dyn Error>> {
        if self.pending_uploads.is_empty() {
            return Ok(());
        }
        let mut write_lock = self.buffer.write()?;
        for (first_vertex, data) in self.pending_uploads.drain(..) {
            write_lock[first_vertex..first_vertex + data.len()].copy_from_slice(&data);
        }
        Ok(())
    }
}

use std::error::Error;

use glam::Mat4;
use vulkano::buffer::Subbuffer;

pub const TRANSFORM_BUFFER_CAPACITY: usize = 1000002;

/// CPU-side authoritative model matrices, indexed by entity. Each frame they
/// are written into the transform buffer of the frame in flight being
/// recorded, so the per-frame buffers never need to be synchronized with each
/// other.
pub struct Transforms {
    model_matrices: Vec<Mat4>,
}

impl Transforms {
    pub fn new() -> Self {
        Self {
            model_matrices: Vec::new(),
        }
    }

    pub fn check_capacity(&self) -> Result<(), Box<dyn Error>> {
        if self.model_matrices.len() >= TRANSFORM_BUFFER_CAPACITY {
            return Err(format!(
                "transform buffer capacity ({TRANSFORM_BUFFER_CAPACITY}) exceeded"
            )
            .into());
        }
        Ok(())
    }

    pub fn push(&mut self, model_matrix: Mat4) {
        self.model_matrices.push(model_matrix);
    }

    pub fn set(&mut self, entity_index: usize, model_matrix: Mat4) {
        self.model_matrices[entity_index] = model_matrix;
    }

    pub fn write_into(
        &self,
        buffer: &Subbuffer<[[[f32; 4]; 4]]>,
        order: impl Iterator<Item = usize>,
    ) -> Result<(), Box<dyn Error>> {
        let mut write_lock = buffer.write()?;
        for (slot, entity_index) in order.enumerate() {
            write_lock[slot] = self.model_matrices[entity_index].to_cols_array_2d();
        }
        Ok(())
    }
}

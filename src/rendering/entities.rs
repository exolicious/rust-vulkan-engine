use std::sync::Arc;

use super::rendering_traits::RenderableEntity;

pub struct Entities {
    pub entities: Vec<Arc<dyn RenderableEntity>>,
}

impl Entities {
    pub fn new() -> Self {
        let entities: Vec<Arc<dyn RenderableEntity>> = Vec::new();
        Self {
            entities
        }
    }

    pub fn push(&mut self, entity: Arc<dyn RenderableEntity>) {

        self.entities.push(entity);
    }
}

impl IntoIterator for Entities {
    type Item = Arc<dyn RenderableEntity>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.into_iter()
    }
}

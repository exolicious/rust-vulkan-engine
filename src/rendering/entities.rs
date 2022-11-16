use std::{cell::RefCell, rc::Rc};

use crate::engine::general_traits::Entity;

use super::{rendering_traits::UpdateGraphics, primitives::RenderableEntity};

pub struct Entities {
    pub entities: Vec<Box<dyn RenderableEntity>>,
}

impl Entities {
    pub fn new() -> Self {
        let entities: Vec<Box<dyn RenderableEntity>> = Vec::new();
        Self {
            entities
        }
    }

    pub fn push(&mut self, entity: Box<dyn RenderableEntity>) {
        self.entities.push(entity);
    }
}

impl IntoIterator for Entities {
    type Item = Box<dyn RenderableEntity>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entities.into_iter()
    }
}

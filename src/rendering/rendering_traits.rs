use crate::{engine::general_traits::Entity, physics::physics_traits::HasTransform};

use super::primitives::{Triangle, Mesh};

pub trait HasMesh : Entity  {
    fn get_mesh(& mut self, name: String) -> Mesh;
    fn get_data(& self) -> Vec<Triangle>;
}

pub enum Visibility {
    Visible,
    Invisible
}

pub trait UpdateGraphics : Entity {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}
pub trait RenderableEntity : Entity + HasMesh + HasTransform {}

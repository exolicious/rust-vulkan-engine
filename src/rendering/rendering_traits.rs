use crate::{engine::general_traits::Entity, physics::physics_traits::HasTransform};

use super::primitives::{Triangle, Mesh};

pub trait HasMesh : Entity  {
    fn set_mesh(&mut self) -> ();
    fn get_data(& self) -> Vec<Triangle>;
    fn get_mesh(& self) -> &Mesh;
}

pub trait UpdateGraphics : Entity {
    fn update_graphics(& self, swapchain_image_index: usize) -> ();
}
pub trait RenderableEntity : Entity + HasMesh + HasTransform {}

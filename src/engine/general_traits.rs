use crate::physics::physics_traits::Transform;
use crate::rendering::primitives::Mesh;

pub enum TickAction {
    HasMoved(Transform),
}

pub trait Entity {
    fn tick(&mut self) -> Option<TickAction>;
    fn transform(&self) -> Transform;
    fn mesh(&self) -> Mesh;
}

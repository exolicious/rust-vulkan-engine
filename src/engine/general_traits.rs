use crate::physics::Transform;
use crate::rendering::primitives::Mesh;

pub enum TickAction {
    HasMoved(Transform),
}

pub trait Entity {
    fn tick(&mut self, frame_time: f32) -> Option<TickAction>;
    fn transform(&self) -> Transform;
    fn mesh(&self) -> Mesh;
}

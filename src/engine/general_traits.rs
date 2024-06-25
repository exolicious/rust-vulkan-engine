use std::{error::Error, sync::Arc};

use crate::{physics::physics_traits::Transform, rendering::{buffer_manager::BufferManager, rendering_traits::Visibility}};

pub enum TickAction {
    HasMoved(Transform),
    ChangedVisibility(Visibility)
}

pub trait Entity {
    fn tick(&mut self) -> Option<TickAction>;
}

pub trait RegisterToBuffer {
    fn register(& self, f: fn(&BufferManager, entity: Arc<dyn Entity>, next_swapchain_image_index: usize, ) -> Result<(), Box<dyn Error>>) -> ();
}
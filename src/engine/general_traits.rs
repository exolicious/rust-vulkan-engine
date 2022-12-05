use std::{error::Error, sync::Arc};

use crate::{rendering::buffer_manager::BufferManager, physics::physics_traits::Transform};

pub enum EntityUpdateAction {
    None,
    HasMoved(String, Transform),
}

pub trait Entity {
    fn get_id(& self) -> &String;
    fn update(&mut self) -> EntityUpdateAction;
}

pub trait RegisterToBuffer {
    fn register(& self, f: fn(&BufferManager, entity: Arc<dyn Entity>, next_swapchain_image_index: usize, ) -> Result<(), Box<dyn Error>>) -> ();
}
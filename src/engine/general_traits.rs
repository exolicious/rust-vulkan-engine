use std::{error::Error, sync::Arc};

use crate::rendering::buffer_manager::BufferManager;

pub trait Entity {
    fn get_id(& self) -> &String;
    fn update(&mut self) -> ();
}

pub trait RegisterToBuffer {
    fn register(& self, f: fn(&BufferManager, entity: Arc<dyn Entity>, next_swapchain_image_index: usize, ) -> Result<(), Box<dyn Error>>) -> ();
}
use std::collections::HashMap;

use super::rendering_traits::HasMesh;

trait Buffer {
    
}

struct BufferManager {
    model_blueprints: HashMap<i64, Box<dyn HasMesh>>,
    buffers: Vec<Box<dyn Buffer>>
}
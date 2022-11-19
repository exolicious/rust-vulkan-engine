use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::Arc, hash::Hasher};

use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage}, device::Device, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::{Pipeline, GraphicsPipeline}};

use super::{rendering_traits::{HasMesh, RenderableEntity}, primitives::{Vertex, Mesh}};

pub struct EntityTransformAccessor {
    pub index: usize,
}

struct BlueprintAccessor {
    pub mesh_hash: u64,
    pub first_index: usize,
    pub entity_counter: usize,
    pub vertex_count: usize,
    pub last_index: usize
}

impl Default for BlueprintAccessor {
    fn default() -> Self {
        Self {mesh_hash: Default::default(), first_index: Default::default(), entity_counter: Default::default(), vertex_count: Default::default(), last_index: Default::default()}
    }
}

const INITIAL_VERTEX_BUFFER_SIZE: usize = 2_i32.pow(8) as usize; // 256, enough for 32 instances of cubes, with 8 vertices; 32*8 = 256
const INITIAL_TRANSFORM_BUFFER_SIZE: usize = 2_i32.pow(4) as usize; // 32 instances

pub struct BufferManager {
    /*     renderer_device:  Arc<Device>, */
    blueprint_accessors: Vec<BlueprintAccessor>,
    entity_id_transform_accessor_map: HashMap<String, EntityTransformAccessor>,
   /*  entity_transform_buffer_entries: HashMap<u64, Vec<EntityAccessor>>, */
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub transform_buffers: Vec<Arc<CpuAccessibleBuffer<[[[f32; 4]; 4]]>>>,
    needs_reallocation: bool
}

impl BufferManager {
    pub fn new(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Self {

        let vertex_buffer = Self::initialize_vertex_buffer(renderer_device.clone());
        let transform_buffers = Self::initialize_transform_buffers(renderer_device, swapchain_images_length);

        let entity_id_transform_accessor_map = HashMap::new();
/*         let entity_transform_buffer_entries: HashMap<u64, Vec<EntityAccessor>> = HashMap::new(); */
        let blueprint_accessors = Vec::new();
        Self {
            /* renderer_device, */
            entity_id_transform_accessor_map,
            blueprint_accessors,
            vertex_buffer,
            transform_buffers,
            /* entity_transform_buffer_entries, */
            needs_reallocation: false
        }
    }
    
    fn initialize_vertex_buffer(renderer_device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[Vertex]>> {
        let initializer_data: Vec<Vertex> = vec![Vertex{position: [0.,0.,0.]}; INITIAL_VERTEX_BUFFER_SIZE];
        CpuAccessibleBuffer::from_iter(
            renderer_device,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            initializer_data
        )
        .unwrap()
    }

    fn initialize_transform_buffers(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Vec<Arc<CpuAccessibleBuffer<[[[f32; 4]; 4]]>>> {
        let mut transform_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let data = vec![[[0_f32; 4]; 4]; INITIAL_TRANSFORM_BUFFER_SIZE];
            let uniform_buffer = CpuAccessibleBuffer::from_iter(
                renderer_device.clone(),
                BufferUsage {
                    uniform_buffer: true,
                    ..Default::default()
                },
                false,
                data,
            )
            .unwrap();
            transform_buffers.push(uniform_buffer);
        }
        transform_buffers
    }

    pub fn register_entity_to_buffer(&mut self, entity: Arc<dyn RenderableEntity>, swapchain_image_index: usize) -> () {
        let mesh = entity.generate_mesh(); // this is stupid
        let mesh_hash = self.get_mesh_hash(&mesh); // this is stupid, but can be useful later on, when the flow will be; check if mesh exists with the filepath entry of the model, if not load the mesh, etc.
        let blueprint_accessors_length = self.blueprint_accessors.len();
        let entity_transform_accessor = match blueprint_accessors_length > 0 {
            true => {
                match self.blueprint_accessors.iter_mut().find(|accessor| accessor.mesh_hash == mesh_hash) {
                    Some(existing_blueprint_accessor) => {
                        let entity_transform_accessor = EntityTransformAccessor {index: existing_blueprint_accessor.entity_counter /* * size of the data held in the uniform buffer */};
                        existing_blueprint_accessor.entity_counter += 1;
                        entity_transform_accessor
                    }
                    None => {
                        let previous_accessor = self.blueprint_accessors.iter().last().unwrap();
                        let previous_accessor_last_index = previous_accessor.first_index + previous_accessor.vertex_count * previous_accessor.entity_counter;
                        let mut blueprint_accessor = BlueprintAccessor { mesh_hash, first_index: previous_accessor_last_index, vertex_count: mesh.vertex_count, entity_counter: 0, last_index: previous_accessor_last_index + mesh.vertex_count};
                        let entity_transform_accessor = EntityTransformAccessor {index: blueprint_accessor.entity_counter};
                            blueprint_accessor.entity_counter += 1;

                        self.copy_blueprint_mesh_data_to_vertex_buffer(&blueprint_accessor, mesh.data);
                        self.blueprint_accessors.push(blueprint_accessor);
                        
                        entity_transform_accessor
                    }
                }
            }
            false => {
                let mut blueprint_accessor = BlueprintAccessor { mesh_hash, first_index: 0, vertex_count: mesh.vertex_count, entity_counter: 0, last_index: mesh.vertex_count};
                let entity_transform_accessor = EntityTransformAccessor {index: blueprint_accessor.entity_counter /* * size of the data held in the uniform buffer */};
                    blueprint_accessor.entity_counter += 1;

                self.copy_blueprint_mesh_data_to_vertex_buffer(&blueprint_accessor, mesh.data);
                self.blueprint_accessors.push(blueprint_accessor);

                entity_transform_accessor
            }
        };
        self.entity_id_transform_accessor_map.entry(entity.get_id().to_string()).or_insert(entity_transform_accessor);
        self.copy_transform_data_to_buffer(entity, swapchain_image_index);
    }

    fn copy_blueprint_mesh_data_to_vertex_buffer(&mut self, blueprint_accessor: &BlueprintAccessor, mesh_data: Vec<Vertex>) {
        match self.vertex_buffer.write() {
            Err(_) => println!("Error"),
            Ok(mut write_lock) => { 
                 write_lock[blueprint_accessor.first_index..blueprint_accessor.last_index].copy_from_slice(&mesh_data.as_slice());
            }
        };
    }

    pub fn copy_transform_data_to_buffer(&mut self, entity: Arc<dyn RenderableEntity>, swapchain_image_index: usize) {
        let entity_transform_accessor = self.entity_id_transform_accessor_map.get(entity.get_id()).expect("somehow entity id is not registered in the transform accessors map inside the buffer manager");
        match self.transform_buffers[swapchain_image_index].write() {
            Err(_) => println!("Error"),
            Ok(mut write_lock) => { 
                write_lock[entity_transform_accessor.index] = entity.get_transform().into();
            }
        };
    }   

    pub fn get_transform_buffer_descriptor_set(& self, pipeline: Arc<GraphicsPipeline>, swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let binding = pipeline.clone();
        let layout = binding.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(1, Arc::new(self.transform_buffers[swapchain_image_index].clone()))],
        )
        .unwrap()
    }

    fn get_mesh_hash(& self, mesh: &Mesh) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        let mut result = Vec::new();
        for triangle in &mesh.data {
            for j in triangle.position {
                let rounded_coord =  (j * 100_f32) as u8;
                result.push(rounded_coord);
            }
        }
        hasher.write(&result);
        hasher.finish()
    }
    
}
use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::Arc, hash::{Hasher, Hash}, cell::RefCell};
use cgmath::{Matrix4, SquareMatrix};
use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage}, device::Device, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, pipeline::{Pipeline, GraphicsPipeline}};
use crate::{camera::camera::Camera, physics::physics_traits::Transform, rendering::renderer::RendererEvent};
use super::{rendering_traits::{RenderableEntity}, primitives::{Vertex, Mesh}};

pub struct EntityTransformBufferIndexMap {
    pub count: usize,
    pub entity_id_transform_accessor_map: HashMap<String, usize>,
}

impl EntityTransformBufferIndexMap {
    pub fn new() -> Self {
        Self {
            count: 0,
            entity_id_transform_accessor_map: HashMap::new()
        }
    }

    pub fn add_entity(&mut self, entity_id: String) -> () {
        self.entity_id_transform_accessor_map.insert(entity_id, self.count);
        self.count += 1;
    }

    pub fn get_transform_buffer_index(&self, entity_id: &String) -> usize {
        self.entity_id_transform_accessor_map[entity_id]
    }
}

pub struct MeshAccessor {
    pub mesh_hash: u64,
    pub first_index: usize,
    pub first_instance: usize,
    pub instance_count: usize,
    pub vertex_count: usize,
    pub last_index: usize,
}

impl MeshAccessor {
    pub fn add_entity(&mut self) {
        self.instance_count += 1;
    }
}

impl Default for MeshAccessor {
    fn default() -> Self {
        Self {
           mesh_hash: 0,
           first_index: 0,
           first_instance: 0,
           instance_count: 0,
           vertex_count: 0,
           last_index: 0
        }
    }
}

const INITIAL_VERTEX_BUFFER_SIZE: usize = 2_i32.pow(8) as usize; // 256, enough for 32 instances of cubes, with 8 vertices; 32*8 = 256
const INITIAL_TRANSFORM_BUFFER_SIZE: usize = 2_i32.pow(4) as usize; // 32 instances

pub struct BufferManager {
    /*     renderer_device:  Arc<Device>, */
    pub mesh_accessors: Vec<MeshAccessor>,
   /*  entity_transform_buffer_entries: HashMap<u64, Vec<EntityAccessor>>, */
    pub vertex_buffers: Vec<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub transform_buffers: Vec<Arc<CpuAccessibleBuffer<[[[f32; 4]; 4]]>>>,
    vp_camera_buffers:  Vec<Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>>>,
    entity_transform_buffer_index_map: EntityTransformBufferIndexMap,
    needs_reallocation: bool,
    ahead_buffers_index: Option<usize>
}

impl BufferManager {
    pub fn new(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Self {
        let vertex_buffers = Self::initialize_vertex_buffers(renderer_device.clone(), swapchain_images_length);
        let transform_buffers = Self::initialize_transform_buffers(renderer_device.clone(), swapchain_images_length);
        let vp_camera_buffers = Self::initialize_vp_camera_buffers(renderer_device.clone(), swapchain_images_length);
        let mesh_accessors = Vec::new();
        let entity_transform_buffer_index_map = EntityTransformBufferIndexMap::new();
        Self {
            /* renderer_device, */
            mesh_accessors,
            vertex_buffers,
            transform_buffers,
            vp_camera_buffers,
            entity_transform_buffer_index_map,
            /* entity_transform_buffer_entries, */
            needs_reallocation: false,
            ahead_buffers_index: None
        }
    }

    fn initialize_vp_camera_buffers(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Vec<Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>>> {
        let mut vp_matrix_buffers = Vec::new();
        let projection_view_matrix: Matrix4<f32> = Matrix4::identity();
        for _ in 0..swapchain_images_length {
            let uniform_buffer: Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>> = CpuAccessibleBuffer::from_data(
                renderer_device.clone(),
                BufferUsage {
                    uniform_buffer: true,
                    ..Default::default()
                },
                false,
                projection_view_matrix.into(),
            )
            .unwrap();
            vp_matrix_buffers.push(uniform_buffer);
        }
        vp_matrix_buffers
    }
    
    fn initialize_vertex_buffers(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Vec<Arc<CpuAccessibleBuffer<[Vertex]>>> {
        let mut vertex_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let initializer_data = vec![Vertex{position: [0.,0.,0.]}; INITIAL_VERTEX_BUFFER_SIZE];
            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                renderer_device.clone(),
                BufferUsage {
                    vertex_buffer: true,
                    ..Default::default()
                },
                false,
                initializer_data.into_iter()
            )
            .unwrap();
            vertex_buffers.push(vertex_buffer);
        }
        vertex_buffers
    }

    fn initialize_transform_buffers(renderer_device: Arc<Device>, swapchain_images_length: usize) -> Vec<Arc<CpuAccessibleBuffer<[[[f32; 4]; 4]]>>> {
        let mut transform_buffers = Vec::new();
        for _ in 0..swapchain_images_length {
            let transform_initial_data: [[[f32; 4]; 4]; 16] = [[[0_f32; 4]; 4]; 16];
            let uniform_buffer = CpuAccessibleBuffer::from_iter(
                renderer_device.clone(),
                BufferUsage {
                    uniform_buffer: true,
                    ..Default::default()
                },
                false,
                transform_initial_data.into_iter()
            )
            .unwrap();
            transform_buffers.push(uniform_buffer);
        }
        transform_buffers
    }

    pub fn sync_buffers(&mut self, entity: Arc<RefCell<dyn RenderableEntity>>, next_swapchain_image_index: usize) -> () {
        let binding = entity.borrow();
        let entity_mesh = binding.get_mesh();
        let entity_id = binding.get_id();
        let entity_transform = binding.get_transform();

        match self.mesh_accessors.iter().find(|accessor| accessor.mesh_hash == entity_mesh.hash) {
            Some(existing_mesh_accessor) => {
                self.copy_blueprint_mesh_data_to_vertex_buffer(&existing_mesh_accessor, &entity_mesh.data, next_swapchain_image_index);
                self.copy_transform_data_to_buffer(entity_id, entity_transform, next_swapchain_image_index);
            }
            _ => (),
        };
    }

    pub fn register_entity(&mut self, entity: Arc<RefCell<dyn RenderableEntity>>, next_swapchain_image_index: usize) -> () {
        let binding = entity.borrow();
        let entity_mesh = binding.get_mesh();
        let entity_id = binding.get_id();
        let entity_transform = binding.get_transform();

        match self.mesh_accessors.iter_mut().find(|accessor| accessor.mesh_hash == entity_mesh.hash) {
            Some(existing_mesh_accessor) => {
                existing_mesh_accessor.add_entity();
            }
            None => {
                match self.mesh_accessors.iter().last() {
                    Some(previous_accessor) => {
                        let mut mesh_acessor = MeshAccessor {
                            mesh_hash: entity_mesh.hash, 
                            first_index: previous_accessor.last_index, 
                            vertex_count: entity_mesh.vertex_count, 
                            last_index: previous_accessor.last_index + entity_mesh.vertex_count,
                            first_instance: previous_accessor.first_instance + previous_accessor.instance_count,
                            ..Default::default()
                        };
                        self.copy_blueprint_mesh_data_to_vertex_buffer(&mesh_acessor, &entity_mesh.data, next_swapchain_image_index);
                        mesh_acessor.add_entity();
                        self.mesh_accessors.push(mesh_acessor);
                    }
                    None =>  {
                        let mut mesh_acessor = MeshAccessor { 
                            mesh_hash: entity_mesh.hash, 
                            vertex_count: entity_mesh.vertex_count, 
                            last_index: entity_mesh.vertex_count,
                            ..Default::default()
                        };
                        self.copy_blueprint_mesh_data_to_vertex_buffer(&mesh_acessor, &entity_mesh.data, next_swapchain_image_index);
                        mesh_acessor.add_entity();
                        self.mesh_accessors.push(mesh_acessor);
                    }
                }
            }
        }
        self.entity_transform_buffer_index_map.add_entity(entity_id.to_string());
        self.copy_transform_data_to_buffer(entity_id, entity_transform, next_swapchain_image_index);
        self.ahead_buffers_index = Some(next_swapchain_image_index);
    }
    

    fn copy_blueprint_mesh_data_to_vertex_buffer(& self, mesh_acessor: &MeshAccessor, mesh_data: &Vec<Vertex>, next_swapchain_image_index: usize) {
        println!("first index: {} \n last index: {} ", mesh_acessor.first_index, mesh_acessor.last_index);
        let main_vertex_buffer = self.vertex_buffers[next_swapchain_image_index].clone();
        match main_vertex_buffer.write() {
            Err(_) => println!("Error writing onto mesh buffer"),
            Ok(mut write_lock) => { 
                println!("Vertex buffer with index [{}] has been filled with mesh data", next_swapchain_image_index);
                write_lock[mesh_acessor.first_index..mesh_acessor.last_index].copy_from_slice(&mesh_data.as_slice());
            }
        };
    }

    pub fn update_entity_transform_buffer(&mut self, entity_id: &String, entity_transform: &Transform, next_swapchain_image_index: usize) {
        self.copy_transform_data_to_buffer(entity_id, entity_transform, next_swapchain_image_index);
    }
 
    fn copy_transform_data_to_buffer(& self, entity_id: &String, entity_transform: &Transform, next_swapchain_image_index: usize) {
        let entity_transform_index = self.entity_transform_buffer_index_map.get_transform_buffer_index(&entity_id);
        println!("transform index: {}", entity_transform_index);
        match self.transform_buffers[next_swapchain_image_index].write() {
            Err(_) => println!("Error writing onto transform buffer"),
            Ok(mut write_lock) => { 
                write_lock[entity_transform_index] = entity_transform.model_matrix();
                println!("Wrote this transform {:?} to the transform buffer with index [{}] ", entity_transform.model_matrix(), next_swapchain_image_index);
            }
        };
    }

    pub fn copy_vp_camera_data(& self, next_swapchain_image_index: usize, camera: &Camera) {
        match self.vp_camera_buffers[next_swapchain_image_index].write() {
            Err(_) => println!("Error writing onto the vp camera buffer"),
            Ok(mut write_lock) => { 
                println!("Wrote into vp camera buffer: {:?}", camera.projection_view_matrix);
                *write_lock = camera.projection_view_matrix.into();
            }
        };
    }

    pub fn get_vp_matrix_buffer_descriptor_set(& self, pipeline: Arc<GraphicsPipeline>, next_swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, Arc::new(self.vp_camera_buffers[next_swapchain_image_index].clone()))], // 0 is the binding
        )
        .unwrap()
    }

    pub fn get_transform_buffer_descriptor_set(& self, pipeline: Arc<GraphicsPipeline>, next_swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let layout = pipeline.layout().set_layouts().get(1).unwrap();
        PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, Arc::new(self.transform_buffers[next_swapchain_image_index].clone()))],
        )
        .unwrap()
    }
}
use std::{collections::{HashMap}, sync::Arc, cell::RefCell};
use cgmath::{Matrix4, SquareMatrix};
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, descriptor_set::{allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, CopyDescriptorSet, PersistentDescriptorSet, WriteDescriptorSet}, device::Device, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{GraphicsPipeline, Pipeline}};
use crate::{engine::camera::Camera, physics::physics_traits::Transform};
use super::{mesh_accessor::{MeshAccessor, MeshAccessorAddEntityResult}, primitives::{Mesh, Vertex}, rendering_traits::RenderableEntity, transform_buffers::TransformBuffers, vertex_buffers::VertexBuffers};
use std::error::Error;
use core::fmt::Error as ErrorVal;



pub struct BufferManager {
    pub mesh_accessor: MeshAccessor,
    pub descriptor_set_allocator: StandardDescriptorSetAllocator,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    /*     renderer_device:  Arc<Device>, */
   /*  entity_transform_buffer_entries: HashMap<u64, Vec<EntityAccessor>>, */
    pub vertex_buffers: VertexBuffers,
    transform_buffers: TransformBuffers,
    vp_camera_buffers:  Vec<Subbuffer<[[f32; 4]; 4]>>, // needs to be a push constant sooner or later
    pub entities_transform_ids: Vec<String>,
    pub entites_to_update: HashMap<String, Transform>
}

impl BufferManager {
    pub fn new(device: Arc<Device>, swapchain_images_length: usize) -> Self {
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(
            device.clone(), 
            StandardDescriptorSetAllocatorCreateInfo::default());
        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        );
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        
        let vertex_buffers = VertexBuffers::new(memory_allocator.clone(), swapchain_images_length);
        let transform_buffers = TransformBuffers::new(memory_allocator.clone(), swapchain_images_length);
        let vp_camera_buffers = Self::initialize_vp_camera_buffers(memory_allocator.clone(), swapchain_images_length);

        let mesh_accessor = MeshAccessor::default();
        let entities_transform_ids = Vec::new();
        let entites_to_update = HashMap::new();
        Self {
            mesh_accessor,
            vertex_buffers,
            transform_buffers,
            vp_camera_buffers,
            entities_transform_ids,
            descriptor_set_allocator,
            command_buffer_allocator,
            memory_allocator,
            entites_to_update,
        }
    }

    fn initialize_vp_camera_buffers(memory_allocator: Arc<StandardMemoryAllocator>, swapchain_images_length: usize) -> Vec<Subbuffer<[[f32; 4]; 4]>> {
        let mut vp_matrix_buffers = Vec::new();
        let projection_view_matrix: Matrix4<f32> = Matrix4::identity();
        for _ in 0..swapchain_images_length {
            let uniform_buffer = Buffer::from_data(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                projection_view_matrix.into(),
            )
            .unwrap();
            vp_matrix_buffers.push(uniform_buffer);
        }
        vp_matrix_buffers
    }

    pub fn sync_mesh_and_transform_buffers(&mut self, entity: Arc<RefCell<dyn RenderableEntity>>, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let binding = entity.borrow();
        let entity_mesh = binding.get_mesh();
        let entity_id = binding.get_id();
        let entity_transform = binding.get_transform();

        match self.mesh_accessors.iter().find(|accessor| accessor.name == entity_mesh.name) {
            Some(existing_mesh_accessor) => {
                self.copy_blueprint_mesh_data_to_vertex_buffer(&existing_mesh_accessor, &entity_mesh.data, next_swapchain_image_index)?;
                self.update_entity_transform_buffer(entity_id, entity_transform, next_swapchain_image_index)?;
            }
            _ => (),
        };
        Ok(())
    }

    pub fn register_entity(&mut self, entity: Arc<RefCell<dyn RenderableEntity>>, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let binding = entity.borrow();
        let entity_mesh = binding.get_mesh();
        let entity_id = binding.get_id();
        let entity_transform = binding.get_transform();

        self.add_entity_to_mesh_accessor(entity_mesh, next_swapchain_image_index)?;

        let entity_transform_index = self.entities_transform_ids.len();
        self.copy_transform_data_to_buffer(entity_transform_index, entity_transform, next_swapchain_image_index)?;
        self.entities_transform_ids.push(entity_id.to_string());

        Ok(())
    }
    
    fn add_entity_to_mesh_accessor(&mut self, entity_mesh: Mesh, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let first_index = self.mesh_accessor.get_last_vertex_index();
        let entity_add_result: MeshAccessorAddEntityResult = self.mesh_accessor.add_entity(entity_mesh);
        match entity_add_result {
            MeshAccessorAddEntityResult::AppendedToExistingMesh => {},
            MeshAccessorAddEntityResult::CreatedNewMesh(mesh) => {
                self.copy_blueprint_mesh_data_to_vertex_buffer(first_index, &mesh.data, next_swapchain_image_index)?;
            }
        }
        Ok(())
    }

    pub fn update_buffers(&mut self, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut entity_model_matrices = Vec::new();
        let mut last_index = 0;
        for (i, (id, transform)) in self.entites_to_update.iter().enumerate() {
            if i - last_index > 1 { 
                self.copy_transform_data_slice_to_buffer(0, entity_model_matrices.len(), &entity_model_matrices, next_swapchain_image_index)?;
                self.entites_to_update.clear();
            }
            entity_model_matrices.push(transform.model_matrix());
            last_index = i;
        }
        Ok(())
    }

    pub fn copy_transform_data_slice_to_buffer(& self,entity_transforms_first_index: usize, entity_transforms_last_index: usize, entity_model_matrices: &[[[f32; 4]; 4]], next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock =  self.transform_buffers[next_swapchain_image_index].write()?;
        write_lock[entity_transforms_first_index..entity_transforms_last_index].copy_from_slice(entity_model_matrices);
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn update_entity_transform_buffer(& self, entity_id: &String, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        match self.entities_transform_ids.iter().position(|existing_entity_id| existing_entity_id == entity_id) {
            Some(entity_transform_index) => {
                self.copy_transform_data_to_buffer(entity_transform_index, entity_transform, next_swapchain_image_index)?;
                Ok(())
            }
            None => Err(Box::new(ErrorVal))
        }
    }

    fn copy_blueprint_mesh_data_to_vertex_buffer(& self, first_index: usize, mesh_data: &Vec<Vertex>, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let offline_vertex_buffer = self.vertex_buffers[next_swapchain_image_index].clone();
        let mut write_lock =  offline_vertex_buffer.write()?;
        write_lock[first_index..mesh_data.iter().len()].copy_from_slice(mesh_data.as_slice());
        //println!("Successfully copied mesh data: {:?} to vertex buffer with index: {}", mesh_data.as_slice(), next_swapchain_image_index);
        Ok(())
    }
 
    fn copy_transform_data_to_buffer(& self, entity_transform_index: usize, entity_transform: &Transform, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock =  self.transform_buffers[next_swapchain_image_index].write()?;
        write_lock[entity_transform_index] = entity_transform.model_matrix();
        //println!("Successfully copied entity transform: {:?} to transform buffer with index: {}", entity_transform.model_matrix(), next_swapchain_image_index);
        Ok(())
    }

    pub fn copy_vp_camera_data(& self, camera: &Camera, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        let mut write_lock = self.vp_camera_buffers[next_swapchain_image_index].write()?;
        *write_lock = camera.projection_view_matrix.into();
        println!("Successfully copied camera vp_matrix: {:?} to vp buffer with index: {}", camera.projection_view_matrix, next_swapchain_image_index);
        Ok(())
    }

    pub fn get_vp_matrix_buffer_descriptor_set(& self, pipeline: Arc<GraphicsPipeline>, next_swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, self.vp_camera_buffers[next_swapchain_image_index].clone())], // 0 is the binding
            []
        )
        .unwrap()
    }

    pub fn get_transform_buffer_descriptor_set(& self, pipeline: Arc<GraphicsPipeline>, next_swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let layout = pipeline.layout().set_layouts().get(1).unwrap();
        PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, self.transform_buffers[next_swapchain_image_index].clone())],
            []
        )
        .unwrap()
    }
}
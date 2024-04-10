use std::{cell::RefCell, error::Error, sync::Arc};

use vulkano::{command_buffer::CommandBufferExecFuture, device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceCreateInfo, 
DeviceExtensions, Queue, QueueCreateInfo, QueueFlags}, image::{Image, ImageUsage}, instance::Instance, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, 
    ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{RenderPass, Subpass}, shader::ShaderModule, single_pass_renderpass, swapchain::{PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{future::{FenceSignalFuture, JoinFuture}, GpuFuture}, Validated, ValidationError, VulkanError};
use winit::{event_loop::{EventLoop}, window::{Window, WindowBuilder}};

use crate::{engine::scene::Scene, initialize::vulkan_instancing::get_vulkan_instance, physics::physics_traits::Transform};

use super::{buffer_manager::BufferManager, primitives::{self, Mesh}, rendering_traits::{RenderableEntity, Visibility}, shaders::Shaders};

pub enum EntityUpdateInfo {
    HasMoved(HasMovedInfo),
    ChangedVisibility(Visibility)
}

pub struct HasMovedInfo {
    pub entity_id: usize,
    pub new_transform: Transform
}

pub enum EngineEvent {
    EntityAdded(Transform, Mesh, usize),
    EntitiesUpdated(Vec<EntityUpdateInfo>),
    ChangedActiveScene(Arc<Scene>),
}



pub struct Renderer {
    vulkan_instance: Arc<Instance>,
    window: Arc<Window>, 
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    queue_family_index: u32,
    queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,

    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,

    pub buffer_manager: BufferManager,
    active_scene: Arc<Scene>,
    pub currenty_not_displayed_swapchain_image_index: usize,
}


impl Renderer {
    pub fn new(event_loop: & EventLoop<()>) -> Renderer {
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let vulkan_instance = get_vulkan_instance(event_loop);
        let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
        let surface = Surface::from_window(vulkan_instance.clone(), window.clone()).unwrap();
        let (physical_device, queue_family_index) = Renderer::build_physical_device_and_queue_family_index(vulkan_instance.clone(), surface.clone(), &device_extensions);
        let (queue, device) = Renderer::build_device_and_queues(physical_device.clone(), queue_family_index, device_extensions);
        let (swapchain, swapchain_images) = Renderer::build_swapchain_and_swapchain_images(physical_device.clone(), surface.clone(), window.clone(), device.clone());
        let render_pass = Renderer::build_render_pass(device.clone(), swapchain.clone());
        let (vertex_shader, fragment_shader) = Renderer::build_shaders(device.clone());
        let pipeline = Renderer::build_pipeline(vertex_shader.clone(), fragment_shader.clone(), device.clone(), render_pass.clone(), None);
        let buffer_manager = BufferManager::new(device.clone(), pipeline, swapchain_images, render_pass, queue_family_index);
        let active_scene = Arc::new(Scene::new());
        let currenty_not_displayed_swapchain_image_index = 0;

        Renderer {
            vulkan_instance,
            window,
            physical_device,
            queue_family_index,
            queue,
            device,
            surface,
            swapchain,
            buffer_manager,
            vertex_shader,
            fragment_shader,
            active_scene,
            currenty_not_displayed_swapchain_image_index
        }

    }

    pub fn build_physical_device_and_queue_family_index(instance: Arc<Instance>, surface: Arc<Surface>, device_extensions: &DeviceExtensions) -> (Arc<PhysicalDevice>, u32) {
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    // Find the first first queue family that is suitable.
                    // If none is found, `None` is returned to `filter_map`,
                    // which disqualifies this physical device.
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
        
                // Note that there exists `PhysicalDeviceType::Other`, however,
                // `PhysicalDeviceType` is a non-exhaustive enum. Thus, one should
                // match wildcard `_` to catch all unknown device types.
                _ => 4,
            })
            .expect("no device available");

        (physical_device, queue_family_index)
    }

    pub fn build_device_and_queues(physical_device: Arc<PhysicalDevice>, queue_family_index: u32, device_extensions: DeviceExtensions,) -> (Arc<Queue>, Arc<Device>) {
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                // here we pass the desired queue family to use by index
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create device");
        let queue: Arc<Queue> = queues.next().unwrap();
        (queue, device)
    }

    pub fn build_swapchain_and_swapchain_images(physical_device: Arc<PhysicalDevice>, surface: Arc<Surface>, window: Arc<Window>, device: Arc<Device>) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");
    
        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = physical_device.surface_formats(&surface, Default::default()).unwrap()[0].0;

        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap();
    
        (swapchain, swapchain_images)
    }

    pub fn build_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
        let render_pass = single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `foo` is a custom name we give to the first and only attachment.
                foo: {
                    format: swapchain.image_format(),  // set the format the same as the swapchain
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [foo],       // Repeat the attachment name here.
                depth_stencil: {},
            },
        )
        .unwrap();

        render_pass
    }
    
    pub fn build_shaders(device: Arc<Device>) -> (Arc<ShaderModule>, Arc<ShaderModule>) {
        let shaders = Shaders::load(device.clone()).unwrap();

        (shaders.vertex_shader, shaders.fragment_shader)
    }

    pub fn build_pipeline(vertex_shader: Arc<ShaderModule>, fragment_shader: Arc<ShaderModule>, device: Arc<Device>, render_pass: Arc<RenderPass>, viewport: Option<Viewport>) -> Arc<GraphicsPipeline> {
        let viewport = match viewport {
            Some(viewport) => viewport,
            None => {
                Viewport {
                    offset: [0.0, 0.0],
                    extent: [1024.0, 1024.0],
                    depth_range: 0.0..=1.0,
                }
            }
        };
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        let vs = vertex_shader.entry_point("main").unwrap();
        let fs = fragment_shader.entry_point("main").unwrap();
    
        let vertex_input_state = <primitives::Vertex as Vertex>::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();
    
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
    
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
    
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
    
        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                // The stages of our pipeline, we have vertex and fragment stages.
                stages: stages.into_iter().collect(),
                // Describes the layout of the vertex input and how should it behave.
                vertex_input_state: Some(vertex_input_state),
                // Indicate the type of the primitives (the default is a list of triangles).
                input_assembly_state: Some(InputAssemblyState::default()),
                // Set the fixed viewport.
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                // Ignore these for now.
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                // This graphics pipeline object concerns the first pass of the render pass.
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap();
        
        pipeline
    }

    pub fn get_future(& mut self, previous_future: Box<dyn GpuFuture>, acquire_future: SwapchainAcquireFuture, acquired_swapchain_index: usize) -> Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>, Validated<VulkanError>>  {
        //let after_future = gui.draw_on_image(previous_future, self.frames[acquired_swapchain_index].swapchain_image_view.clone());
        let command_buffer = self.buffer_manager.build_command_buffer(acquired_swapchain_index);
        previous_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), acquired_swapchain_index.try_into().unwrap())
            )
            .then_signal_fence_and_flush()
    }

    pub fn entities_updated_handler(&mut self, updated_entities_infos: Vec<EntityUpdateInfo>) -> ()  {
        for (i, entity_update_info) in updated_entities_infos.iter().enumerate() {
            match entity_update_info {
                EntityUpdateInfo::HasMoved(has_moved_info) => {
                    let mut entity_model_matrices = Vec::new();
                    let mut last_index = 0;
                    if has_moved_info.entity_id - last_index > 1 { 
                        self.buffer_manager.copy_transform_data_slice_to_buffer(0, entity_model_matrices.len(), &entity_model_matrices, self.currenty_not_displayed_swapchain_image_index);
                        entity_model_matrices.clear();
                    }
                    entity_model_matrices.push(has_moved_info.new_transform.model_matrix());
                    last_index = has_moved_info.entity_id;
                    
                },
                EntityUpdateInfo::ChangedVisibility(changed_visibility_info) => todo!(),
            }
        }
    }

    //todo: make it so that when multiple entities get added in one frame, they will get collected and not as many events get fired
    pub fn entity_added_handler(&mut self, entity_transform: Transform, entity_mesh: Mesh, entity_index: usize) -> ()  {
        //println!("Entity added in frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.register_entity(entity_transform, entity_mesh, self.currenty_not_displayed_swapchain_image_index, entity_index) {
            Ok(()) => {
                println!("Successfully handled EntityAdded event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }
    }

    pub fn changed_active_scene_handler(&mut self, active_scene: Arc<Scene>) -> ()  {
        println!("Active scene changed in frame index: {}", self.currenty_not_displayed_swapchain_image_index);
        match self.buffer_manager.copy_vp_camera_data(&active_scene.camera, self.currenty_not_displayed_swapchain_image_index) {
            Ok(()) => {
                println!("Successfully handled Changed Active Scene event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }

        //here the buffer_manager would have to do way more after setting the camera matrix, we would have to overwrite the whole state basically.
        //maybe an idea would be to have 1 buffer manager for each scene
    }

    //pub fn recreate_swapchain(&mut self) {
    //    let new_dimensions = self.window.inner_size();
    //    let (new_swapchain, new_images) = self.swapchain
    //        .recreate(SwapchainCreateInfo {
    //            // Here, `image_extend` will correspond to the window dimensions.
    //            image_extent: new_dimensions.into(),
    //            ..self.swapchain.create_info()
    //        })
    //        .expect("failed to recreate swapchain: {e}");
    //   
    //    // since framebuffers are dependant on swapchain (images) we need to recreate them aswell
    //    let frames = Renderer::build_frames(self.device.clone(), self.pipeline.clone(), new_images.clone(), 
    //                                                    self.render_pass.clone(), self.queue_family_index, self.buffer_manager);
//
    //    self.swapchain = new_swapchain;
    //    self.swapchain_images = new_images;
    //    self.frames = frames;
    //}

    pub fn recreate_pipeline(&mut self) {
        let new_dimensions = self.window.inner_size();
        let mut viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [1024.0, 1024.0],
            depth_range: 0.0..=1.0,
        };

        viewport.extent = new_dimensions.into();
        
    }

    //fn synch_buffers_handler(&mut self, most_up_to_date_buffer_index: usize, entity: Arc<dyn RenderableEntity>) -> () {
    //    if most_up_to_date_buffer_index == self.currenty_not_displayed_swapchain_image_index { //if this is equal, synching needs to be done
    //        self.receive_event(EventResolveTiming::Immediate(RendererEvent::BuffersSynched));
    //        return;
    //    } 
    //    println!("Attempting Vertex and transform buffer sync for frame index: {}", self.currenty_not_displayed_swapchain_image_index);
    //    match self.buffer_manager.unwrap().sync_mesh_and_transform_buffers(self.currenty_not_displayed_swapchain_image_index) {
    //        Ok(()) => {
    //            self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)));
    //        }
    //        Err(err) => println!("something went wrong while handling the SynchBuffers Event"),
    //    }
    //}
}
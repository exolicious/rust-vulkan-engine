use std::{cell::RefCell, error::Error, sync::Arc};

use egui_winit_vulkano::Gui;

use vulkano::{command_buffer::CommandBufferExecFuture, device::{physical::{PhysicalDevice, PhysicalDeviceType}, Device, DeviceCreateInfo, 
DeviceExtensions, Queue, QueueCreateInfo, QueueFlags}, image::{Image, ImageUsage}, instance::Instance, pipeline::{graphics::{color_blend::{ColorBlendAttachmentState, 
    ColorBlendState}, input_assembly::InputAssemblyState, multisample::MultisampleState, rasterization::RasterizationState, vertex_input::{BuffersDefinition, Vertex, VertexDefinition}, viewport::{Viewport, ViewportState}, GraphicsPipelineCreateInfo}, layout::PipelineDescriptorSetLayoutCreateInfo, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{RenderPass, Subpass}, shader::ShaderModule, single_pass_renderpass, swapchain::{PresentFuture, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{future::{FenceSignalFuture, JoinFuture}, GpuFuture}, Validated, ValidationError, VulkanError};

use vulkano_win::{create_surface_from_handle, required_extensions, VkSurfaceBuild};
use winit::{event_loop::{EventLoop}, window::{Window, WindowBuilder}};

use crate::{engine::scene::Scene, initialize::vulkan_instancing::get_vulkan_instance, physics::physics_traits::Transform};
use crate::rendering::frame::Frame;

use super::{buffer_manager::BufferManager, primitives, rendering_traits::{RenderableEntity, Visibility}, shaders::Shaders};

pub enum EntityUpdateInfo {
    HasMoved(HasMovedInfo),
    ChangedVisibility(Visibility)
}

pub struct HasMovedInfo {
    pub entity_id: usize,
    pub new_transform: Transform
}

pub enum EventResolveTiming {
    Immediate(RendererEvent),
    NextImage(RendererEvent)
}

pub enum RendererEvent {
    WindowResized,
    RecreateSwapchain,
    BuffersSynched,
    EntityAdded(Arc<RefCell<dyn RenderableEntity>>),
    EntitiesUpdated(Vec<EntityUpdateInfo>),
    SynchBuffers(Arc<RefCell<dyn RenderableEntity>>, usize),
    ChangedActiveScene(Arc<Scene>),
    SynchCameraBuffers(Arc<Scene>, usize)
}

pub struct RendererBuilder {
    renderer: Renderer
}

impl RendererBuilder {
    pub fn new() -> Self {
        let renderer = Renderer::default();
        Self {
            renderer
        }
    }

    pub fn get_renderer(&self) -> Renderer {
        self.renderer
    }


    pub fn build_device_extensions(&mut self) -> &mut Self {
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_shader_non_semantic_info: true,
            ..DeviceExtensions::empty()
        };
        self.renderer.device_extensions = Some(device_extensions);
        self
    }

    pub fn build_physical_device_and_queue_family_index(&mut self) -> &mut Self {
        let (physical_device, queue_family_index) = get_vulkan_instance()
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(&self.renderer.device_extensions.expect("Device Extensions not set in Renderer")))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(_queue_family_index, queue_family_properties)| {
                        queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                _ => 4,
            })
        .expect("no device available");

        self.renderer.physical_device = Some(physical_device);
        self.renderer.queue_family_index = queue_family_index;
        self
    }

    pub fn build_device_and_queues(&mut self) -> &mut Self {
        let queue_family_index = self.renderer.queue_family_index;
        let physical_device = self.renderer.physical_device.as_ref().unwrap();
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                // here we pass the desired queue family to use by index
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("failed to create device");
        let queue: Arc<Queue> = queues.next().unwrap();
        self.renderer.device = Some(device);
        self.renderer.queue = Some(queue);
        self
    }

    pub fn build_swapchain_and_swapchain_images(&mut self) -> &mut Self {
        let caps = self.renderer.physical_device.unwrap()
        .surface_capabilities(&self.renderer.surface.unwrap(), Default::default())
        .expect("failed to get surface capabilities");
    
        let dimensions = self.renderer.window.unwrap().inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = self.renderer.physical_device.unwrap().surface_formats(&self.renderer.surface.unwrap(), Default::default()).unwrap()[0].0;

        let (swapchain, swapchain_images) = Swapchain::new(
            self.renderer.device.unwrap().clone(),
            self.renderer.surface.unwrap().clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage {
                    ..Default::default()
                },
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap();
       
        self.renderer.swapchain = Some(swapchain);
        self.renderer.swapchain_images = Some(swapchain_images);
        self
    }

    pub fn build_render_pass(&mut self) ->&mut Self {
        let render_pass = single_pass_renderpass!(
            self.renderer.device.unwrap().clone(),
            attachments: {
                // `foo` is a custom name we give to the first and only attachment.
                foo: {
                    format: self.renderer.swapchain.unwrap().image_format(),  // set the format the same as the swapchain
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
        self.renderer.render_pass = Some(render_pass);
        self
    }

    pub fn build_buffer_manager(&mut self) -> &mut Self {
        self.renderer.buffer_manager = Some(BufferManager::new(self.renderer.device.unwrap().clone(), self.renderer.swapchain_images.unwrap().len()));
        self
    }

    
    pub fn build_shaders(&mut self) -> &mut Self {
        let shaders = Shaders::load(self.renderer.device.unwrap().clone()).unwrap();
        self.renderer.vertex_shader = Some(shaders.vertex_shader);
        self.renderer.fragment_shader = Some(shaders.fragment_shader);
        self
    }

    pub fn build_pipeline(&mut self) -> &mut Self {
        // More on this latter.
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [1024.0, 1024.0],
            depth_range: 0.0..=1.0,
        };

        
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        let vs = self.renderer.vertex_shader.unwrap().entry_point("main").unwrap();
        let fs = self.renderer.fragment_shader.unwrap().entry_point("main").unwrap();
    
        let vertex_input_state = <primitives::Vertex as Vertex>::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();
    
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
    
        let layout = PipelineLayout::new(
            self.renderer.device.unwrap().clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.renderer.device.unwrap().clone())
                .unwrap(),
        )
        .unwrap();
    
        let subpass = Subpass::from(self.renderer.render_pass.unwrap().clone(), 0).unwrap();
    
        let pipeline = GraphicsPipeline::new(
            self.renderer.device.unwrap().clone(),
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
        
        self.renderer.pipeline = Some(pipeline);
        self
    }

    //has to be called again, when its buffers are out of date (re-allocated due to being too small), or when the swapchain gets updated (window gets resized, or old swapchain was suboptimal )
    pub fn build_frames(&mut self) -> &mut Self {
        let mut temp_frames = Vec::new();
        for (swapchain_image_index, swapchain_image) in self.renderer.swapchain_images.unwrap().iter().enumerate() {
            let mut temp_frame = Frame::new(
                swapchain_image.clone(), 
                self.renderer.device.unwrap().clone(), 
                self.renderer.pipeline.as_ref().unwrap().clone(), 
                swapchain_image_index
            );
            temp_frame.init(self.renderer.render_pass.unwrap().clone(), self.renderer.active_queue.unwrap().queue_family_index(), &self.renderer.buffer_manager.unwrap());
            temp_frames.push(temp_frame);
        }
        self.renderer.frames = Some(temp_frames);
        self
    }

}

#[derive(Default)]
pub struct Renderer {
    builder: Option<Box<RendererBuilder>>,
    //vulkan_instance: Arc<Instance>,
    device_extensions: Option<DeviceExtensions>,
    viewport: Option<Viewport>,
    surface: Option<Arc<Surface>>,
    window: Option<Arc<Window>>, 
    device: Option<Arc<Device>>,
    physical_device: Option<Arc<PhysicalDevice>>,
    queue_family_index: u32,
    active_queue: Option<Arc<Queue>>,
    swapchain: Option<Arc<Swapchain>>,
    swapchain_images: Option<Vec<Arc<Image>>>,
    queue: Option<Arc<Queue>>,
    render_pass: Option<Arc<RenderPass>>,
    event_queue: Option<Vec<RendererEvent>>,
    event_queue_next_frame: Option<Vec<RendererEvent>>,
    frames: Option<Vec<Frame>>,
    buffer_manager: Option<BufferManager>,
    active_scene: Option<Arc<Scene>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_shader: Option<Arc<ShaderModule>>,
    fragment_shader: Option<Arc<ShaderModule>>,
    next_swapchain_image_index: usize,
}


impl Renderer {
    pub fn set_builder(&mut self, builder: Box<RendererBuilder>) {
        self.builder = Some(builder);
    }

    pub fn set_active_scene(&mut self, active_scene: Arc<Scene>) -> () {
        self.active_scene = Some(active_scene.clone());
        self.receive_event(EventResolveTiming::NextImage(RendererEvent::ChangedActiveScene(active_scene.clone())));
    }
    
    pub fn receive_event(&mut self, event_timing: EventResolveTiming) -> () {
        match event_timing {
            EventResolveTiming::Immediate(event) => {
                match event {
                    RendererEvent::WindowResized => self.window_resized_event_handler(),
                    RendererEvent::RecreateSwapchain => self.recreate_swapchain_event_handler(),
                    RendererEvent::BuffersSynched => self.init_command_buffers(),
                    RendererEvent::EntityAdded(_) => todo!(),
                    RendererEvent::SynchBuffers(_, _) => todo!(),
                    RendererEvent::ChangedActiveScene(_) => todo!(),
                    RendererEvent::SynchCameraBuffers(_, _) => todo!(),
                    RendererEvent::EntitiesUpdated(_) => todo!(),
                }  
            },
            EventResolveTiming::NextImage(event) => self.event_queue_next_frame.unwrap().push(event)
        }
    }

    fn entity_moved_event_handler(&mut self) -> () {
        
    }

    fn recreate_swapchain_event_handler(&mut self) -> () {
        self.recreate_swapchain_and_framebuffers();
        self.builder.unwrap().build_frames();
    }

    fn window_resized_event_handler(&mut self) -> ()  {
        self.recreate_swapchain_and_framebuffers();
        self.builder.unwrap().build_pipeline();
        self.builder.unwrap().build_frames();
    }

    fn init_command_buffers(&mut self) {
        for frame in self.frames.unwrap().iter_mut() {
            frame.init_command_buffer(self.active_queue.unwrap().queue_family_index(), &self.buffer_manager.unwrap());
        }
    }

    pub fn update_buffers(&mut self, next_swapchain_image_index: usize) -> Result<(), Box<dyn Error>> {
        self.buffer_manager.unwrap().update_buffers(next_swapchain_image_index)
    }

    pub fn recreate_swapchain_and_framebuffers(&mut self) -> () {
        let new_dimensions = self.window.unwrap().inner_size();
        let (new_swapchain, new_swapchain_images) = match self.swapchain.unwrap().recreate(SwapchainCreateInfo {
            image_extent: new_dimensions.into(),
            ..self.swapchain.unwrap().create_info()
        }) {
            Ok(r) => r,
            Err(ValidationError) => return,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.swapchain = Some(new_swapchain);
        self.swapchain_images =  Some(new_swapchain_images);
        
        //dependent on self.swapchain
        self.recreate_render_pass();
    }

    fn recreate_render_pass(&mut self) -> () {
        self.builder.unwrap().build_render_pass();
    }

    pub fn get_future(&mut self, previous_future: Box<dyn GpuFuture>, acquire_future: SwapchainAcquireFuture, acquired_swapchain_index: usize) -> Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>, Validated<VulkanError>>  {
        //let after_future = gui.draw_on_image(previous_future, self.frames[acquired_swapchain_index].swapchain_image_view.clone());

        previous_future
            .join(acquire_future)
            .then_execute(self.active_queue.unwrap().clone(), self.frames.unwrap()[acquired_swapchain_index].draw_command_buffer.as_ref().unwrap().clone())
            .unwrap()
            .then_swapchain_present(
                self.active_queue.unwrap().clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.unwrap().clone(), acquired_swapchain_index.try_into().unwrap())
            )
            .then_signal_fence_and_flush()
    }

            
    pub fn work_off_queue(&mut self, acquired_swapchain_index: usize) {
        //set up the event queue from the next_frame event queue
        let len = self.event_queue_next_frame.unwrap().len();
        for _ in 0..len {
            match self.event_queue_next_frame.unwrap().pop() {
                Some(event) => self.event_queue.unwrap().push(event),
                None => todo!(),
            }
        }

        let len = self.event_queue.unwrap().len();
        //work off the events
        for _ in 0..len {
            match self.event_queue.unwrap().pop() { // ToDo: decide if fifo or lifo is the right way, for now lifo seems to work
                Some(RendererEvent::EntityAdded(entity)) => self.entity_added_handler(acquired_swapchain_index, entity),
                Some(RendererEvent::ChangedActiveScene(active_scene)) => self.changed_active_scene_handler(acquired_swapchain_index, active_scene),
                Some(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)) => self.synch_buffers_handler(most_up_to_date_buffer_index, acquired_swapchain_index, entity),
                Some(RendererEvent::SynchCameraBuffers(scene, most_up_to_date_buffer_index)) => self.synch_camera_buffers_handler(most_up_to_date_buffer_index, acquired_swapchain_index, scene),
                Some(RendererEvent::EntitiesUpdated(updated_entities_infos)) => self.entities_updated_handler(updated_entities_infos),
                _ => ()
            }
        }
    }

    fn entities_updated_handler(&mut self, updated_entities_infos: Vec<EntityUpdateInfo>) -> ()  {
        for (i, entity_update_info) in updated_entities_infos.iter().enumerate() {
            match entity_update_info {
                EntityUpdateInfo::HasMoved(has_moved_info) => {
                    let mut entity_model_matrices = Vec::new();
                    let mut last_index = 0;
                    if has_moved_info.entity_id - last_index > 1 { 
                        self.buffer_manager.unwrap().copy_transform_data_slice_to_buffer(0, entity_model_matrices.len(), &entity_model_matrices, self.next_swapchain_image_index)?;
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
    fn entity_added_handler(&mut self, acquired_swapchain_index: usize, entity: Arc<RefCell<dyn RenderableEntity>>) -> ()  {
        //println!("Entity added in frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.unwrap().register_entity(entity.clone(), acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchBuffers(entity, acquired_swapchain_index))); //set the synch event with the index that is now the most up to date (regarding buffer data)
                //println!("Successfully handled EntityAdded event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }
    }

    fn synch_buffers_handler(&mut self, most_up_to_date_buffer_index: usize, acquired_swapchain_index: usize, entity: Arc<RefCell<dyn RenderableEntity>>) -> () {
        if most_up_to_date_buffer_index == acquired_swapchain_index { //if this is equal, synching needs to be done
            self.receive_event(EventResolveTiming::Immediate(RendererEvent::BuffersSynched));
            return;
        } 
        println!("Attempting Vertex and transform buffer sync for frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.unwrap().sync_mesh_and_transform_buffers(entity.clone(), acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)));
            }
            Err(err) => println!("something went wrong while handling the SynchBuffers Event"),
        }
    }

    fn changed_active_scene_handler(&mut self, acquired_swapchain_index: usize, active_scene: Arc<Scene>) -> ()  {
        println!("Active scene changed in frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.unwrap().copy_vp_camera_data(&active_scene.camera, acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchCameraBuffers(active_scene.clone(), acquired_swapchain_index)));
                println!("Successfully handled Changed Active Scene event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }

        //here the buffer_manager would have to do way more after setting the camera matrix, we would have to overwrite the whole state basically.
        //maybe an idea would be to have 1 buffer manager for each scene
    }

    fn synch_camera_buffers_handler(&mut self, most_up_to_date_buffer_index: usize, acquired_swapchain_index: usize, active_scene: Arc<Scene>) -> () {
        if most_up_to_date_buffer_index == acquired_swapchain_index { 
            self.receive_event(EventResolveTiming::Immediate(RendererEvent::BuffersSynched));
            println!("all buffers are up to date"); 
            return; 
        } //if this is not equal, there is still synching to be done, until they are equal
        println!("Attempting camera vp buffer sync for frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.unwrap().copy_vp_camera_data(&active_scene.camera, acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchCameraBuffers(active_scene, most_up_to_date_buffer_index))); //set the synch event with the index that is now the most up to date (regarding buffer data)
                println!("Successfully handled Synch Camera Buffers event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }
    }
}
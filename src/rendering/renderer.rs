use std::{sync::Arc, cell::RefCell};

use egui_winit_vulkano::Gui;

use vulkano::{swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError, SwapchainAcquireFuture, PresentFuture, SwapchainPresentInfo}, 
    device::{Device, Queue, physical::{PhysicalDevice, PhysicalDeviceType}, DeviceCreateInfo, QueueCreateInfo, DeviceExtensions}, instance::Instance, 
    image::{SwapchainImage, ImageUsage}, render_pass::{RenderPass, Subpass}, 
    pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState}, GraphicsPipeline}, 
    command_buffer::{CommandBufferExecFuture}, shader::ShaderModule, sync::{GpuFuture, FenceSignalFuture, JoinFuture, FlushError}};

use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::{EventLoop}, window::{Window, WindowBuilder}};

use crate::{initialize::vulkan_instancing::get_vulkan_instance, engine::{scene::Scene}};
use crate::rendering::primitives::Vertex;
use crate::rendering::frame::Frame;

use super::{rendering_traits::{RenderableEntity}, buffer_manager::BufferManager};

pub enum EventResolveTiming {
    Immediate(RendererEvent),
    NextImage(RendererEvent)
}

pub enum RendererEvent {
    WindowResized,
    RecreateSwapchain,
    BuffersSynched,
    EntityAdded(Arc<RefCell<dyn RenderableEntity>>),
    SynchBuffers(Arc<RefCell<dyn RenderableEntity>>, usize),
    ChangedActiveScene(Arc<Scene>),
    SynchCameraBuffers(Arc<Scene>, usize)
}

pub struct Renderer {
    //vulkan_instance: Arc<Instance>,
    viewport: Viewport,
    pub surface: Arc<Surface>,
    window: Arc<Window>, 
    pub device: Arc<Device>,
    /* physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
    queues: Box<(dyn ExactSizeIterator<Item = Arc<Queue>> + 'static)>, */
    pub active_queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub swapchain_images: Vec<Arc<SwapchainImage>>,
    render_pass: Arc<RenderPass>,
    event_queue: Vec<RendererEvent>,
    event_queue_next_frame: Vec<RendererEvent>,
    pub frames: Vec<Frame>,
    pub buffer_manager: Box<BufferManager>,
    pub active_scene: Option<Arc<Scene>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_shader: Option<Arc<ShaderModule>>,
    fragment_shader: Option<Arc<ShaderModule>>,
    pub next_swapchain_image_index: usize,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let vulkan_instance = get_vulkan_instance();
        let surface = WindowBuilder::new().build_vk_surface(&event_loop, vulkan_instance.clone()).unwrap();
        let window = surface.object().unwrap().to_owned().downcast::<Window>().unwrap();
        let viewport= Viewport {
            origin: [0.0, 0.0],
            dimensions: window.inner_size().into(),
            depth_range: 0.0..1.0,
        };

        Self::init(vulkan_instance, viewport, surface, window, event_loop)
    }

    pub fn init(vulkan_instance: Arc<Instance>, viewport: Viewport, surface : Arc<Surface>, window: Arc<Window>, event_loop: &EventLoop<()>) -> Self {
        //this is just hard coded since we want this to only work with devices that support swapchains
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_shader_non_semantic_info: true,
            ..DeviceExtensions::empty()
        };
       
        let (physical_device, queue_family_index) = Self::init_physical_device_and_queue_family_index(device_extensions.clone(), vulkan_instance.clone(), surface.clone());
        let (device, queues, active_queue) = Self::init_device_and_queues(device_extensions.clone(), queue_family_index.clone(), physical_device.clone());
        let (swapchain, swapchain_images) = Self::init_swapchain_and_swapchain_images(physical_device.clone(), surface.clone(), device.clone(), window.clone());
        
        let render_pass = Self::create_render_pass(device.clone(), swapchain.clone());

        let buffer_manager = Box::new(BufferManager::new(device.clone(), swapchain_images.len()));
        
        let event_queue = Vec::new();
        let event_queue_next_frame = Vec::new();
        let frames = Vec::new();

        Self {
            //vulkan_instance,
            viewport,
            surface,
            //physical_device,
            //queue_family_index,
            device,
            //queues: Box::new(queues),
            active_queue,
            swapchain,
            swapchain_images,
            render_pass,
            event_queue,
            event_queue_next_frame,
            frames,
            buffer_manager,
            window, 
            active_scene: None,
            pipeline: None,
            vertex_shader: None,
            fragment_shader: None,
            next_swapchain_image_index: 0
        }
    }

    pub fn set_active_scene(&mut self, active_scene: Arc<Scene>) -> () {
        self.active_scene = Some(active_scene.clone());
        self.receive_event(EventResolveTiming::NextImage(RendererEvent::ChangedActiveScene(active_scene.clone())));
    }

    pub fn build(&mut self, vertex_shader: Arc<ShaderModule>, fragment_shader: Arc<ShaderModule>) -> () {
        self.vertex_shader = Some(vertex_shader);
        self.fragment_shader = Some(fragment_shader);
        self.init_pipeline();
        self.init_frames();
    }

    //initializes the physical device and gets the queue family index of the queue family that supports the needed properties of the surface (.surface_support)
    fn init_physical_device_and_queue_family_index(device_extensions: DeviceExtensions, vulkan_instance: Arc<Instance>, surface: Arc<Surface>) -> (Arc<PhysicalDevice>, u32) {
        vulkan_instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.graphics && p.surface_support(i as u32, &surface).unwrap_or(false)
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
        .expect("no device available")
    }

    //initializes the logical device and gets all the available queues, then gets the first one and sets it as the active queue
    fn init_device_and_queues(device_extensions: DeviceExtensions, queue_family_index: u32, physical_device: Arc<PhysicalDevice>) -> (Arc<vulkano::device::Device>, impl ExactSizeIterator + Iterator<Item = Arc<Queue>>, Arc<Queue>) {
        let queue_create_info = QueueCreateInfo {
            queue_family_index: queue_family_index,
            ..Default::default()
        };
        let device_create_info = DeviceCreateInfo {
            queue_create_infos: vec![queue_create_info],
            enabled_extensions: device_extensions, // new
            
            ..Default::default()
        };
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            device_create_info
        )
        .expect("failed to create device");
        let queue = queues.next().unwrap();
        (device, queues, queue)
    }

    fn init_swapchain_and_swapchain_images(physical_device: Arc<PhysicalDevice>, surface: Arc<Surface>, device: Arc<Device> , window: Arc<Window>) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>) {
        let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    
        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let image_format = Some(
            physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );
    
        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage {
                    color_attachment: true,
                    ..Default::default()
                },
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap();
        (swapchain, swapchain_images)
    }

    fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),  // set the format the same as the swapchain
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();
        render_pass
    }

    pub fn init_pipeline(&mut self) -> () {
        let new_dimensions = self.window.inner_size();
        self.viewport.dimensions = new_dimensions.into();
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(self.vertex_shader.as_ref().unwrap().entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([self.viewport.clone()]))
            .fragment_shader(self.fragment_shader.as_ref().unwrap().entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(self.render_pass.clone(), 0).unwrap())
            .build(self.device.clone())
            .unwrap();
        self.pipeline = Some(pipeline);
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
                }  
            },
            EventResolveTiming::NextImage(event) => self.event_queue_next_frame.push(event)
        }
    }

    fn recreate_swapchain_event_handler(&mut self) -> () {
        self.recreate_swapchain_and_framebuffers();
        self.init_frames();
    }

    fn window_resized_event_handler(&mut self) -> ()  {
        self.recreate_swapchain_and_framebuffers();
        self.init_pipeline();
        self.init_frames();
    }

    //has to be called again, when its buffers are out of date (re-allocated due to being too small), or when the swapchain gets updated (window gets resized, or old swapchain was suboptimal )
    fn init_frames(&mut self) {
        let mut temp_frames = Vec::new();
        for (swapchain_image_index, swapchain_image) in self.swapchain_images.iter().enumerate() {
            let mut temp_frame = Frame::new(
                swapchain_image.clone(), 
                self.device.clone(), 
                self.pipeline.as_ref().unwrap().clone(), 
                swapchain_image_index
            );
            temp_frame.init(self.render_pass.clone(), self.active_queue.queue_family_index(), &self.buffer_manager);
            temp_frames.push(temp_frame);
        }
        self.frames = temp_frames;
    }

    fn init_command_buffers(&mut self) {
        for frame in self.frames.iter_mut() {
            frame.init_command_buffer(self.active_queue.queue_family_index(), &self.buffer_manager);
        }
    }

    pub fn recreate_swapchain_and_framebuffers(&mut self) -> () {
        let new_dimensions = self.window.inner_size();
        let (new_swapchain, new_swapchain_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: new_dimensions.into(),
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.swapchain = new_swapchain;
        self.swapchain_images = new_swapchain_images;
        
        //dependent on self.swapchain
        self.recreate_render_pass();
    }

    fn recreate_render_pass(&mut self) -> () {
        self.render_pass = Self::create_render_pass(self.device.clone(), self.swapchain.clone());

    }

    pub fn get_future(&mut self, previous_future: Box<dyn GpuFuture>, acquire_future: SwapchainAcquireFuture, acquired_swapchain_index: usize, gui: &mut Gui) -> Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>>, FlushError>  {
        //let after_future = gui.draw_on_image(previous_future, self.frames[acquired_swapchain_index].swapchain_image_view.clone());

        previous_future
            .join(acquire_future)
            .then_execute(self.active_queue.clone(), self.frames[acquired_swapchain_index].draw_command_buffer.as_ref().unwrap().clone())
            .unwrap()
            .then_swapchain_present(
                self.active_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), acquired_swapchain_index.try_into().unwrap())
            )
            .then_signal_fence_and_flush()
    }

            

    pub fn work_off_queue(&mut self, acquired_swapchain_index: usize) {
        //set up the event queue from the next_frame event queue
        let len = self.event_queue_next_frame.len();
        for _ in 0..len {
            match self.event_queue_next_frame.pop() {
                Some(event) => self.event_queue.push(event),
                None => todo!(),
            }
        }

        let len = self.event_queue.len();
        //work off the events
        for _ in 0..len {
            match self.event_queue.pop() { // ToDo: decide if fifo or lifo is the right way, for now lifo seems to work
                Some(RendererEvent::EntityAdded(entity)) => self.entity_added_handler(acquired_swapchain_index, entity),
                Some(RendererEvent::ChangedActiveScene(active_scene)) => self.changed_active_scene_handler(acquired_swapchain_index, active_scene),
                Some(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)) => self.synch_buffers_handler(most_up_to_date_buffer_index, acquired_swapchain_index, entity),
                Some(RendererEvent::SynchCameraBuffers(scene, most_up_to_date_buffer_index)) => self.synch_camera_buffers_handler(most_up_to_date_buffer_index, acquired_swapchain_index, scene),
                _ => ()
            }
        }
    }

    //todo: make it so that when multiple entities get added in one frame, they will get collected and not as many events get fired
    fn entity_added_handler(&mut self, acquired_swapchain_index: usize, entity: Arc<RefCell<dyn RenderableEntity>>) -> ()  {
        //println!("Entity added in frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.register_entity(entity.clone(), acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchBuffers(entity, acquired_swapchain_index))); //set the synch event with the index that is now the most up to date (regarding buffer data)
                //println!("Successfully handled EntityAdded event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }
    }

    fn synch_buffers_handler(&mut self, most_up_to_date_buffer_index: usize, acquired_swapchain_index: usize, entity: Arc<RefCell<dyn RenderableEntity>>) -> () {
        if most_up_to_date_buffer_index == acquired_swapchain_index { //if this is not equal, there is still synching to be done, until they are equal
            self.receive_event(EventResolveTiming::Immediate(RendererEvent::BuffersSynched));
            return; 
        } 
        println!("Attempting Vertex and transform buffer sync for frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.sync_mesh_and_transform_buffers(entity.clone(), acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchBuffers(entity, most_up_to_date_buffer_index)));
            }
            Err(err) => println!("something went wrong while handling the SynchBuffers Event"),
        }
    }

    fn changed_active_scene_handler(&mut self, acquired_swapchain_index: usize, active_scene: Arc<Scene>) -> ()  {
        println!("Active scene changed in frame index: {}", acquired_swapchain_index);
        match self.buffer_manager.copy_vp_camera_data(&active_scene.camera, acquired_swapchain_index) {
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
        match self.buffer_manager.copy_vp_camera_data(&active_scene.camera, acquired_swapchain_index) {
            Ok(()) => {
                self.receive_event(EventResolveTiming::NextImage(RendererEvent::SynchCameraBuffers(active_scene, most_up_to_date_buffer_index))); //set the synch event with the index that is now the most up to date (regarding buffer data)
                println!("Successfully handled Synch Camera Buffers event");
            }
            Err(err) => println!("something went wrong while handling the EntityAdded Event"),
        }
    }
}
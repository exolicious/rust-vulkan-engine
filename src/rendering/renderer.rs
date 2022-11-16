use std::{sync::Arc};

use vulkano::{swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError, SwapchainAcquireFuture, PresentFuture, PresentInfo}, 
    device::{Device, Queue, physical::{PhysicalDevice, PhysicalDeviceType}, DeviceCreateInfo, QueueCreateInfo, DeviceExtensions}, instance::Instance, 
    image::{SwapchainImage, ImageUsage}, render_pass::{RenderPass, Subpass}, 
    pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState}, GraphicsPipeline, Pipeline}, 
    command_buffer::{PrimaryAutoCommandBuffer, CommandBufferExecFuture}, shader::ShaderModule, buffer::{CpuAccessibleBuffer}, descriptor_set::{WriteDescriptorSet, PersistentDescriptorSet}, 
    sync::{GpuFuture, FenceSignalFuture, JoinFuture, FlushError}};

use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::{EventLoop}, window::{Window, WindowBuilder}};

use crate::{initialize::vulkan_instancing::get_vulkan_instance, camera::camera::Camera};
use crate::rendering::primitives::Vertex;
use crate::rendering::frame::Frame;

use super::rendering_traits::UniformBufferOwner;

pub enum RendererEvent {
    WindowResized,
    RecreateSwapchain,
    EntityAdded(Arc<CpuAccessibleBuffer<[Vertex]>>)
}

pub struct Renderer<T> {
    //vulkan_instance: Arc<Instance>,
    viewport: Viewport,
    surface: Arc<T>,
    pub device: Arc<Device>,
    /* physical_device: Arc<PhysicalDevice>,
    queue_family_index: u32,
    queues: Box<(dyn ExactSizeIterator<Item = Arc<Queue>> + 'static)>, */
    pub active_queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain<Window>>,
    pub swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    render_pass: Arc<RenderPass>,
    event_queue: Vec<RendererEvent>,
    frames: Vec<Frame>,
    pub camera: Option<Camera>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    vertex_shader: Option<Arc<ShaderModule>>,
    fragment_shader: Option<Arc<ShaderModule>>,
    vertex_buffers: Vec<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    uniform_buffers: Option<Vec<Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>>>>,
}

impl Renderer<Surface<Window>> {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let vulkan_instance = get_vulkan_instance();
        let surface = WindowBuilder::new().build_vk_surface(&event_loop, vulkan_instance.clone()).unwrap();
        let viewport= Viewport {
            origin: [0.0, 0.0],
            dimensions: surface.window().inner_size().into(),
            depth_range: 0.0..1.0,
        };
        Self::init(vulkan_instance, viewport, surface)
    }

    pub fn init(vulkan_instance: Arc<Instance>, viewport: Viewport, surface : Arc<Surface<Window>>) -> Self {
        //this is just hard coded since we want this to only work with devices that support swapchains
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
       
        let (physical_device, queue_family_index) = Self::init_physical_device_and_queue_family_index(device_extensions.clone(), vulkan_instance.clone(), surface.clone());
        let (device, queues, active_queue) = Self::init_device_and_queues(device_extensions.clone(), queue_family_index.clone(), physical_device.clone());
        let (swapchain, swapchain_images) = Self::init_swapchain_and_swapchain_images(physical_device.clone(), surface.clone(), device.clone());
        
        let render_pass = Self::create_render_pass(device.clone(), swapchain.clone());
        let event_queue = Vec::new();
        let vertex_buffers = Vec::new();
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
            frames,
            camera: None,
            pipeline: None,
            vertex_shader: None,
            fragment_shader: None,
            vertex_buffers,
            uniform_buffers: None
        }
    }

    pub fn use_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    pub fn build(&mut self, vertex_shader: Arc<ShaderModule>, fragment_shader: Arc<ShaderModule>, vertex_buffers: Option<Vec<Arc<CpuAccessibleBuffer<[Vertex]>>>>) -> () {
        self.vertex_shader = Some(vertex_shader);
        self.fragment_shader = Some(fragment_shader);
        self.uniform_buffers = Some(self.camera.as_ref().unwrap().get_uniform_buffers());
        for vertex_buffer in vertex_buffers.unwrap() {
            self.vertex_buffers.push(vertex_buffer);
        }
        self.init_pipeline();
        self.init_frames();
    }

    //initializes the physical device and gets the queue family index of the queue family that supports the needed properties of the surface (.surface_support)
    fn init_physical_device_and_queue_family_index(device_extensions: DeviceExtensions, vulkan_instance: Arc<Instance>, surface: Arc<Surface<Window>>) -> (Arc<PhysicalDevice>, u32) {
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

    fn init_swapchain_and_swapchain_images(physical_device: Arc<PhysicalDevice>, surface: Arc<Surface<Window>>, device: Arc<Device>) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    
        let dimensions = surface.window().inner_size();
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

    fn create_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Arc<RenderPass> {
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
        let new_dimensions = self.surface.window().inner_size();
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

    pub fn receive_event(&mut self, event: RendererEvent) {
        self.event_queue.push(event)
    }

    pub fn work_off_queue(&mut self) {
        match self.event_queue.pop() {
            Some(event) => {
                match event {
                    RendererEvent::WindowResized => {
                        self.recreate_swapchain_and_framebuffers();
                        self.init_pipeline();
                        self.init_frames()
                    }
                    RendererEvent::RecreateSwapchain => {
                        self.recreate_swapchain_and_framebuffers();
                        self.init_frames()
                    }
                    RendererEvent::EntityAdded(vertex_buffer) => {
                        for frame in &mut self.frames {
                            frame.add_vertex_buffer(vertex_buffer.clone());
                        }
                    }
                }
            }
            None => ()
        }
    }

    pub fn init_frames(&mut self) {
        let mut temp_frames = Vec::new();
        //if self.pipeline.is_none() { self.create_pipeline() }
        for (swapchain_image_index, swapchain_image) in self.swapchain_images.iter().enumerate() {
            let mut temp_frame = Frame::new(
                swapchain_image.clone(), 
                self.render_pass.clone(), 
                self.device.clone(), 
                self.active_queue.queue_family_index(), 
                self.pipeline.as_ref().unwrap().clone(), 
                self.vertex_buffers.clone(), 
                self.get_uniform_buffer_descriptor_set(swapchain_image_index)
            );
            temp_frame.init();
            temp_frames.push(temp_frame);
        }
        self.frames = temp_frames;
    }

    pub fn get_uniform_buffer_descriptor_set(& self, swapchain_image_index: usize) -> Arc<PersistentDescriptorSet> {
        let binding = self.pipeline.as_ref().unwrap().clone();
        let layout = binding.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, Arc::new(self.uniform_buffers.as_ref().unwrap()[swapchain_image_index].clone()))], // 0 is the binding
        )
        .unwrap()
    }

    pub fn recreate_swapchain_and_framebuffers(&mut self) -> () {
        let new_dimensions = self.surface.window().inner_size();
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

    pub fn get_future(& self, previous_future: Box<dyn GpuFuture>, acquire_future: SwapchainAcquireFuture<Window>, image_i: usize) -> Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>, Arc<PrimaryAutoCommandBuffer>>, Window>>, FlushError> {
        previous_future
            .join(acquire_future)
            .then_execute(self.active_queue.clone(), self.frames[image_i].command_buffer.as_ref().unwrap().clone())
            .unwrap()
            .then_swapchain_present(
                self.active_queue.clone(),
                PresentInfo {
                    index: image_i,
                    ..PresentInfo::swapchain(self.swapchain.clone())
                },
            )
            .then_signal_fence_and_flush()
    }
}
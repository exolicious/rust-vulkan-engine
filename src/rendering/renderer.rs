use std::{sync::Arc};
use vulkano::{swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError}, 
    device::{Device, Queue, physical::{PhysicalDevice, PhysicalDeviceType}, DeviceCreateInfo, QueueCreateInfo, DeviceExtensions}, instance::Instance, 
    image::{SwapchainImage, ImageUsage, view::ImageView}, render_pass::{RenderPass, Framebuffer, FramebufferCreateInfo, Subpass}, 
    pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState}, GraphicsPipeline, Pipeline}, 
    command_buffer::{PrimaryAutoCommandBuffer, pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, 
    SubpassContents}, shader::ShaderModule, buffer::{CpuAccessibleBuffer, TypedBufferAccess}, descriptor_set::WriteDescriptorSet};
use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::{EventLoop}, window::{Window, WindowBuilder}};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::PipelineBindPoint;


use crate::{initialize::vulkan_instancing::get_vulkan_instance, engine::engine::Engine};
use crate::rendering::primitives::Vertex;


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
    framebuffers: Vec<Arc<Framebuffer>>,
    pipeline: Option<Arc<GraphicsPipeline>>,
    pub command_buffers: Option<Vec<Arc<PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>>>>,
    vertex_shader: Option<Arc<ShaderModule>>,
    fragment_shader: Option<Arc<ShaderModule>>,
    vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    uniform_buffer: Option<Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>>>,
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
        let framebuffers = Self::create_framebuffers(swapchain_images.clone(), render_pass.clone());
        
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
            framebuffers,
            pipeline: None,
            command_buffers: None,
            vertex_shader: None,
            fragment_shader: None,
            vertex_buffer: None,
            uniform_buffer: None
        }
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

    fn create_framebuffers(swapchain_images: Vec<Arc<SwapchainImage<Window>>>, render_pass: Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
        let framebuffers = swapchain_images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        framebuffers
    }

    pub fn init_shaders(&mut self, vertex_shader: Arc<ShaderModule>, fragment_shader: Arc<ShaderModule>) -> () {
        self.set_vertex_shader(vertex_shader);
        self.set_fragment_shader(fragment_shader);
    }

    pub fn set_vertex_shader(&mut self, vertex_shader: Arc<ShaderModule>) -> () {
        self.vertex_shader = Some(vertex_shader);
    }

    pub fn set_fragment_shader(&mut self, fragment_shader: Arc<ShaderModule>) -> () {
        self.fragment_shader = Some(fragment_shader);
    }

    pub fn create_pipeline(&mut self) -> () {
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

    pub fn init_vertex_buffers(&mut self, vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>) -> () {
        self.vertex_buffer = Some(vertex_buffer);
    }

    pub fn init_uniform_buffers(&mut self, uniform_buffer: Arc<CpuAccessibleBuffer<[[f32; 4]; 4]>>) -> () {
        self.uniform_buffer = Some(uniform_buffer);
    }

    pub fn get_uniform_buffer_descriptor_set(&mut self) ->  Arc<PersistentDescriptorSet> {
        if self.pipeline.is_none() { self.create_pipeline() }
        let binding = self.pipeline.as_ref().unwrap().clone();
        let layout = binding.layout().set_layouts().get(0).unwrap();
        PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, self.uniform_buffer.as_ref().unwrap().clone())], // 0 is the binding
        )
        .unwrap()
    }

    pub fn create_command_buffers(&mut self) -> () {
        if self.pipeline.is_none() { self.create_pipeline() }
        if !self.command_buffers.is_none() { return }
        let command_buffers = self.framebuffers
        .clone()
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                self.device.clone(),
                self.active_queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassContents::Inline,
                )
                .unwrap()
                .bind_pipeline_graphics(self.pipeline.as_ref().unwrap().clone())
                .bind_vertex_buffers(0, self.vertex_buffer.as_ref().unwrap().clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.pipeline.as_ref().unwrap().layout().clone(),
                    0,
                    self.get_uniform_buffer_descriptor_set(),
                )
                .draw(self.vertex_buffer.as_ref().unwrap().clone().len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();
            Arc::new(builder.build().unwrap())
        })
        .collect();
        self.command_buffers = Some(command_buffers);
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
        //dependent on self.swapchain.images
        self.recreate_framebuffers();
    }

    fn recreate_render_pass(&mut self) -> () {
        self.render_pass = Self::create_render_pass(self.device.clone(), self.swapchain.clone());
    }
    
    fn recreate_framebuffers(&mut self) -> () {
        self.framebuffers = Self::create_framebuffers(self.swapchain_images.clone(), self.render_pass.clone());
    }

    pub fn recreate_pipeline_and_commandbuffers(&mut self) -> () {
        let new_dimensions = self.surface.window().inner_size();
        self.viewport.dimensions = new_dimensions.into();
        self.create_pipeline();
        self.create_command_buffers();
    }

    /* pub fn get_future(&self, previous_future: Box<dyn GpuFuture>, acquire_future: SwapchainAcquireFuture<Window>, image_i: usize) -> Result<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>, Arc<PrimaryAutoCommandBuffer>>, Window>>, FlushError> {
        previous_future
            .join(acquire_future)
            .then_execute(self.active_queue.clone(), self.command_buffers.as_ref().unwrap()[image_i].clone())
            .unwrap()
            .then_swapchain_present(
                self.active_queue.clone(),
                PresentInfo {
                    index: image_i,
                    ..PresentInfo::swapchain(self.swapchain.clone())
                },
            )
            .then_signal_fence_and_flush()
    } */

    pub fn start_renderer(&mut self, mut event_loop: EventLoop<()>, engine: &Engine) -> () {
        
    }
}
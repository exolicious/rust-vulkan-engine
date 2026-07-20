use std::sync::Arc;

use glam::Mat4;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SecondaryAutoCommandBuffer, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::format::Format;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::image::{Image, ImageUsage};
use vulkano::ordered_passes_renderpass;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{
    self, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::engine::scene::Scene;
use crate::initialize::vulkan_instancing::get_vulkan_instance;
use crate::physics::physics_traits::Transform;

use super::buffer_manager::BufferManager;
use super::frame::{Frame, FrameInFlight, MAX_FRAMES_IN_FLIGHT};
use super::primitives::{self, Mesh};
use super::shaders::Shaders;

pub struct HasMovedInfo {
    pub entity_id: usize,
    pub new_transform: Transform,
}

pub enum EntityUpdateInfo {
    HasMoved(HasMovedInfo),
}

pub enum EngineEvent {
    EntityAdded(Transform, Mesh, usize),
    EntitiesUpdated(Vec<EntityUpdateInfo>),
    ChangedActiveScene(Arc<Scene>),
}

pub struct Renderer {
    window: Arc<Window>,
    pub surface: Arc<Surface>,
    pub device: Arc<Device>,
    queue_family_index: u32,
    pub queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub graphics_pipeline: Arc<GraphicsPipeline>,
    pub buffer_manager: BufferManager,
    active_scene: Option<Arc<Scene>>,
    /// Per-swapchain-image resources, rebuilt whenever the swapchain is.
    swapchain_frames: Vec<Frame>,
    /// Per-frame-in-flight resources; fixed size, independent of the swapchain.
    frames_in_flight: Vec<FrameInFlight>,
    current_frame: usize,
    swapchain_outdated: bool,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>) -> Renderer {
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let vulkan_instance = get_vulkan_instance(event_loop);
        let window = Arc::new(WindowBuilder::new().build(event_loop).unwrap());
        let surface = Surface::from_window(vulkan_instance.clone(), window.clone()).unwrap();
        let (physical_device, queue_family_index) =
            Self::pick_physical_device(vulkan_instance, surface.clone(), &device_extensions);
        let (queue, device) =
            Self::build_device_and_queue(physical_device.clone(), queue_family_index, device_extensions);
        let (swapchain, swapchain_images) =
            Self::build_swapchain(physical_device, surface.clone(), window.clone(), device.clone());
        let render_pass = Self::build_render_pass(device.clone(), swapchain.clone());
        let shaders = Shaders::load(device.clone()).unwrap();
        let graphics_pipeline = Self::build_pipeline(
            shaders.vertex_shader,
            shaders.fragment_shader,
            device.clone(),
            render_pass.clone(),
        );
        let buffer_manager = BufferManager::new(device.clone());

        let set_layouts = graphics_pipeline.layout().set_layouts();
        let frames_in_flight = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                FrameInFlight::new(
                    buffer_manager.memory_allocator.clone(),
                    &buffer_manager.descriptor_set_allocator,
                    set_layouts[0].clone(),
                    set_layouts[1].clone(),
                )
            })
            .collect();
        let swapchain_frames = Self::build_swapchain_frames(swapchain_images, render_pass.clone());

        Renderer {
            window,
            surface,
            device,
            queue_family_index,
            queue,
            swapchain,
            render_pass,
            graphics_pipeline,
            buffer_manager,
            active_scene: None,
            swapchain_frames,
            frames_in_flight,
            current_frame: 0,
            swapchain_outdated: false,
        }
    }

    fn build_swapchain_frames(
        swapchain_images: Vec<Arc<Image>>,
        render_pass: Arc<RenderPass>,
    ) -> Vec<Frame> {
        swapchain_images
            .into_iter()
            .map(|image| Frame::new(image, render_pass.clone()))
            .collect()
    }

    fn pick_physical_device(
        instance: Arc<vulkano::instance::Instance>,
        surface: Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> (Arc<PhysicalDevice>, u32) {
        instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            .filter(|p| p.supported_extensions().contains(device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
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
                _ => 4,
            })
            .expect("no suitable physical device available")
    }

    fn build_device_and_queue(
        physical_device: Arc<PhysicalDevice>,
        queue_family_index: u32,
        device_extensions: DeviceExtensions,
    ) -> (Arc<Queue>, Arc<Device>) {
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create device");
        (queues.next().unwrap(), device)
    }

    fn build_swapchain(
        physical_device: Arc<PhysicalDevice>,
        surface: Arc<Surface>,
        window: Arc<Window>,
        device: Arc<Device>,
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        // egui blends in linear space and expects a UNORM render target, so
        // prefer one over the sRGB formats most drivers list first.
        let surface_formats = physical_device
            .surface_formats(&surface, Default::default())
            .unwrap();
        let image_format = surface_formats
            .iter()
            .map(|(format, _)| *format)
            .find(|format| matches!(format, Format::B8G8R8A8_UNORM | Format::R8G8B8A8_UNORM))
            .unwrap_or(surface_formats[0].0);

        Swapchain::new(
            device,
            surface,
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )
        .unwrap()
    }

    fn build_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
        // Subpass 0 draws the scene, subpass 1 draws the egui overlay on top.
        ordered_passes_renderpass!(
            device,
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            passes: [
                { color: [color], depth_stencil: {}, input: [] },
                { color: [color], depth_stencil: {}, input: [] },
            ],
        )
        .unwrap()
    }

    fn build_pipeline(
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
    ) -> Arc<GraphicsPipeline> {
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

        let subpass = Subpass::from(render_pass, 0).unwrap();

        GraphicsPipeline::new(
            device,
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                // The viewport is dynamic state, so the pipeline survives
                // swapchain recreation unchanged.
                viewport_state: Some(ViewportState::default()),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn swapchain_extent(&self) -> [u32; 2] {
        self.swapchain.image_extent()
    }

    pub fn mark_swapchain_outdated(&mut self) {
        self.swapchain_outdated = true;
    }

    /// Waits for the frame that last used the current frame-in-flight slot, so
    /// its buffers are safe to write, then acquires the next swapchain image.
    /// Returns None when no image can be acquired this frame (swapchain
    /// outdated, minimized).
    pub fn begin_frame(&mut self) -> Option<(u32, SwapchainAcquireFuture)> {
        if self.swapchain_outdated && !self.recreate_swapchain() {
            return None;
        }

        if let Some(fence) = &self.frames_in_flight[self.current_frame].fence {
            fence.wait(None).unwrap();
        }

        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(result) => result,
                Err(VulkanError::OutOfDate) => {
                    self.swapchain_outdated = true;
                    return None;
                }
                Err(e) => panic!("failed to acquire next swapchain image: {e}"),
            };
        if suboptimal {
            self.swapchain_outdated = true;
        }

        Some((image_index, acquire_future))
    }

    pub fn end_frame(
        &mut self,
        image_index: u32,
        acquire_future: SwapchainAcquireFuture,
        gui_command_buffer: Arc<SecondaryAutoCommandBuffer>,
    ) {
        // The vertex buffer is shared by all frames in flight; only write to it
        // once every in-flight frame has finished.
        if self.buffer_manager.has_pending_vertex_uploads() {
            self.wait_for_all_frames();
        }

        let view_projection = self
            .active_scene
            .as_ref()
            .map(|scene| scene.camera.projection_view_matrix)
            .unwrap_or(Mat4::IDENTITY);
        if let Err(e) = self
            .buffer_manager
            .upload_frame_data(&self.frames_in_flight[self.current_frame], view_projection)
        {
            eprintln!("failed to upload frame data: {e}");
        }

        let command_buffer = self.build_command_buffer(image_index as usize, gui_command_buffer);

        let previous_frame = (self.current_frame + MAX_FRAMES_IN_FLIGHT - 1) % MAX_FRAMES_IN_FLIGHT;
        let previous_future = match self.frames_in_flight[previous_frame].fence.clone() {
            Some(fence) => fence.boxed(),
            None => {
                let mut now = sync::now(self.device.clone());
                now.cleanup_finished();
                now.boxed()
            }
        };

        let future = previous_future
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        self.frames_in_flight[self.current_frame].fence = match future.map_err(Validated::unwrap) {
            Ok(fence) => Some(Arc::new(fence)),
            Err(VulkanError::OutOfDate) => {
                self.swapchain_outdated = true;
                None
            }
            Err(e) => {
                eprintln!("failed to flush frame: {e}");
                None
            }
        };
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn build_command_buffer(
        &self,
        image_index: usize,
        gui_command_buffer: Arc<SecondaryAutoCommandBuffer>,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.buffer_manager.command_buffer_allocator,
            self.queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let frame = &self.frames_in_flight[self.current_frame];
        let extent = self.swapchain.image_extent();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        self.swapchain_frames[image_index].framebuffer.clone(),
                    )
                },
                SubpassBeginInfo::default(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.graphics_pipeline.clone())
            .unwrap()
            .set_viewport(0, [viewport].into_iter().collect())
            .unwrap()
            .bind_vertex_buffers(0, self.buffer_manager.vertex_buffer.buffer.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.graphics_pipeline.layout().clone(),
                0,
                vec![
                    frame.vp_descriptor_set.clone(),
                    frame.transform_descriptor_set.clone(),
                ],
            )
            .unwrap();

        // In Vulkan gl_InstanceIndex starts at first_instance, so each mesh
        // group indexes its own contiguous slice of the transform buffer.
        let mut first_instance = 0;
        for group in &self.buffer_manager.vertex_buffer.mesh_accessor.groups {
            let instance_count = group.entity_indices.len() as u32;
            builder
                .draw(
                    group.mesh.data.len() as u32,
                    instance_count,
                    group.first_vertex as u32,
                    first_instance,
                )
                .unwrap();
            first_instance += instance_count;
        }

        builder
            .next_subpass(
                SubpassEndInfo::default(),
                SubpassBeginInfo {
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                },
            )
            .unwrap()
            .execute_commands(gui_command_buffer)
            .unwrap()
            .end_render_pass(SubpassEndInfo::default())
            .unwrap();

        builder.build().unwrap()
    }

    fn recreate_swapchain(&mut self) -> bool {
        let dimensions = self.window.inner_size();
        if dimensions.width == 0 || dimensions.height == 0 {
            return false;
        }

        self.wait_for_all_frames();

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: dimensions.into(),
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        self.swapchain = new_swapchain;
        self.swapchain_frames = Self::build_swapchain_frames(new_images, self.render_pass.clone());
        // All frames have finished; dropping the fences releases their
        // references to the old swapchain.
        for frame in &mut self.frames_in_flight {
            frame.fence = None;
        }
        self.current_frame = 0;
        self.swapchain_outdated = false;
        true
    }

    fn wait_for_all_frames(&self) {
        for fence in self.frames_in_flight.iter().filter_map(|frame| frame.fence.as_ref()) {
            fence.wait(None).unwrap();
        }
    }

    pub fn entity_added_handler(&mut self, transform: Transform, mesh: Mesh, entity_index: usize) {
        if let Err(e) = self.buffer_manager.register_entity(transform, mesh, entity_index) {
            eprintln!("failed to register entity {entity_index}: {e}");
        }
    }

    pub fn entities_updated_handler(&mut self, updates: Vec<EntityUpdateInfo>) {
        for update in updates {
            match update {
                EntityUpdateInfo::HasMoved(info) => self
                    .buffer_manager
                    .set_entity_transform(info.entity_id, &info.new_transform),
            }
        }
    }

    pub fn changed_active_scene_handler(&mut self, scene: Arc<Scene>) {
        self.active_scene = Some(scene);
    }
}

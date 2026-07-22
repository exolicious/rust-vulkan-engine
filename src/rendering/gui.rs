use std::sync::Arc;

use egui_winit_vulkano::{egui, Gui, GuiConfig};
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use vulkano::render_pass::Subpass;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

use super::renderer::Renderer;

/// The render pass subpass reserved for the gui overlay, drawn on top of the
/// scene (see `Renderer::build_render_pass`).
const GUI_SUBPASS_INDEX: u32 = 1;

/// The egui overlay. Owns the egui integration and hides how the gui gets
/// into the renderer's second subpass; what the UI shows is up to the caller
/// of `draw`.
pub struct GuiOverlay {
    gui: Gui,
}

impl GuiOverlay {
    pub fn new(event_loop: &EventLoop<()>, renderer: &Renderer) -> Self {
        let subpass = Subpass::from(renderer.render_pass.clone(), GUI_SUBPASS_INDEX).unwrap();
        let gui = Gui::new_with_subpass(
            event_loop,
            renderer.surface.clone(),
            renderer.queue.clone(),
            subpass,
            renderer.swapchain.image_format(),
            GuiConfig {
                allow_srgb_render_target: true,
                ..GuiConfig::default()
            },
        );

        let context = gui.context();
        let mut style = (*context.style()).clone();
        style.visuals.window_shadow = egui::epaint::Shadow::NONE;
        style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
        context.set_style(style);

        Self { gui }
    }

    /// Feeds a window event to egui. Returns true if egui consumed it, in
    /// which case the engine should ignore it.
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.gui.update(event)
    }

    /// Runs `build_ui` for this frame and records the result into a secondary
    /// command buffer for the renderer's gui subpass.
    pub fn draw(
        &mut self,
        extent: [u32; 2],
        build_ui: impl FnOnce(&egui::Context),
    ) -> Arc<SecondaryAutoCommandBuffer> {
        self.gui.immediate_ui(|gui| build_ui(&gui.context()));
        self.gui.draw_on_subpass_image(extent)
    }
}

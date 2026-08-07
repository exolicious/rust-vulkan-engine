use std::sync::Arc;

use egui_winit_vulkano::{egui, Gui, GuiConfig};
use vulkano::command_buffer::SecondaryAutoCommandBuffer;
use vulkano::render_pass::Subpass;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

use super::renderer::Renderer;

const GUI_SUBPASS_INDEX: u32 = 1;

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

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.gui.update(event)
    }

    /// True while the pointer is over any egui area. Unlike the `consumed`
    /// flag from `handle_event`, this is also true for plain hovering, which
    /// is what the engine needs to decide whether to hide the cursor.
    pub fn pointer_over_gui(&self) -> bool {
        self.gui.context().is_pointer_over_area()
    }

    pub fn draw(
        &mut self,
        extent: [u32; 2],
        build_ui: impl FnOnce(&egui::Context),
    ) -> Arc<SecondaryAutoCommandBuffer> {
        self.gui.immediate_ui(|gui| build_ui(&gui.context()));
        self.gui.draw_on_subpass_image(extent)
    }
}

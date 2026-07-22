use std::sync::Arc;

use egui_winit_vulkano::egui::{self, Button, Context, DragValue, Slider, Widget};
use glam::Vec3;
use vulkano::command_buffer::SecondaryAutoCommandBuffer;

use crate::engine::Engine;

pub struct GuiState {
    pub x_scale: f32,
    pub y_scale: f32,
    pub z_scale: f32
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            x_scale: 1.,
            y_scale: 1.,
            z_scale: 1.
        }
    }

    pub fn build_ui(&mut self, ctx: &Context, engine: &mut Engine, frame_time_ms: f32, fps: f32) -> () {
        egui::Window::new("Engine").show(ctx, |ui| {
                ui.label(format!("frame time: {frame_time_ms:.2} ms"));
                ui.label(format!("fps: {fps:.0}"));
                ui.label(format!("entities: {}", engine.entity_count()));
                ui.label("space: spawn cubes");
                ui.vertical(|ui| {
                    ui.label("Spawn Cube");
                    ui.vertical(|ui| {
                        ui.add(Slider::new(&mut self.x_scale, 1. ..=10.).integer().prefix("X Scale: "));
                        ui.add(Slider::new(&mut self.y_scale, 1. ..=10.).integer().prefix("Y Scale: "));
                        ui.add(Slider::new(&mut self.z_scale, 1. ..=10.).integer().prefix("Z Scale: "));

                        if ui.button("Spawn Cube").clicked() {
                            engine.add_cube_to_scene(Some(Vec3::new(0.,0.,0.)), Some(Vec3::new(self.x_scale,  self.y_scale, self.z_scale)));
                        }
                    })
                })
            });
    }
}
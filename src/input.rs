use std::collections::HashSet;

use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta,
    VirtualKeyCode, WindowEvent,
};

#[derive(Default)]
pub struct InputManager {
    keys_down: HashSet<VirtualKeyCode>,
    keys_pressed: HashSet<VirtualKeyCode>,
    keys_released: HashSet<VirtualKeyCode>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,

    cursor_position: Option<PhysicalPosition<f64>>,
    mouse_delta: (f32, f32),
    scroll_delta: f32,
    modifiers: ModifiersState,

    keyboard_captured: bool,
    pointer_captured: bool,
}

impl InputManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_gui_capture(&mut self, keyboard: bool, pointer: bool) {
        self.keyboard_captured = keyboard;
        self.pointer_captured = pointer;
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(key),
                        ..
                    },
                ..
            } => self.set_key_state(*key, *state),

            WindowEvent::MouseInput { state, button, .. } => {
                self.set_mouse_button_state(*button, *state)
            }

            WindowEvent::CursorMoved { position, .. } => self.cursor_position = Some(*position),

            WindowEvent::MouseWheel { delta, .. } if !self.pointer_captured => {
                self.scroll_delta += match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(position) => position.y as f32 / 16.0,
                };
            }

            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = *modifiers,

            WindowEvent::Focused(false) => self.release_all(),
            WindowEvent::CursorLeft { .. } => self.cursor_position = None,

            _ => {}
        }
    }

    
    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
            self.mouse_delta.0 += *x as f32;
            self.mouse_delta.1 += *y as f32;
        }
    }

    pub fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_buttons_pressed.clear();
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn was_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn was_key_released(&self, key: VirtualKeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub fn was_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    pub fn cursor_position(&self) -> Option<PhysicalPosition<f64>> {
        self.cursor_position
    }

    pub fn modifiers(&self) -> ModifiersState {
        self.modifiers
    }

    pub fn axis(&self, positive: VirtualKeyCode, negative: VirtualKeyCode) -> f32 {
        let mut value = 0.0;
        if self.is_key_down(positive) {
            value += 1.0;
        }
        if self.is_key_down(negative) {
            value -= 1.0;
        }
        value
    }

    fn set_key_state(&mut self, key: VirtualKeyCode, state: ElementState) {
        match state {
            // Releases are never dropped, even while the gui has focus: a key
            // pressed before the gui took focus still has to come back up.
            ElementState::Released => {
                self.keys_down.remove(&key);
                self.keys_released.insert(key);
            }
            ElementState::Pressed if !self.keyboard_captured => {
                // Key repeat re-sends `Pressed` for a key that is already
                // down; only the first one counts as a press.
                if self.keys_down.insert(key) {
                    self.keys_pressed.insert(key);
                }
            }
            ElementState::Pressed => {}
        }
    }

    fn set_mouse_button_state(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Released => {
                self.mouse_buttons_down.remove(&button);
            }
            ElementState::Pressed if !self.pointer_captured => {
                if self.mouse_buttons_down.insert(button) {
                    self.mouse_buttons_pressed.insert(button);
                }
            }
            ElementState::Pressed => {}
        }
    }

    fn release_all(&mut self) {
        self.keys_released.extend(self.keys_down.drain());
        self.mouse_buttons_down.clear();
        self.mouse_delta = (0.0, 0.0);
    }
}

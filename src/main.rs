use rendering::window_manager::WindowManager;

pub mod initialize;
pub mod rendering;
pub mod physics;
pub mod camera;
pub mod engine;

fn main() {
    let window = WindowManager::new();
    window.start_engine();
}

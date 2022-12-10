use rendering::window_manager::WindowManager;

pub mod initialize;
pub mod rendering;
pub mod physics;
pub mod engine;

use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let window = WindowManager::new();
    window.start_engine();
}

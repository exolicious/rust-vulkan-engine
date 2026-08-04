use glam::Vec3;

pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: i8,
}

impl DirectionalLight {
    pub fn new(direction: Vec3, color: Vec3, intensity: i8) -> Self {
        Self {
            direction,
            color,
            intensity,
        }
    }
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0., -1., 0.),
            color: Vec3::ONE,
            intensity: 1,
        }
    }
}
use glam::Vec3;

pub struct PointLight {
    pub translation: Vec3,
    pub color: Vec3,
    pub intensity: i8,
}

impl PointLight {
    pub fn new(translation: Vec3, color: Vec3, intensity: i8) -> Self {
        Self {
            translation,
            color,
            intensity,
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            color: Vec3::ONE,
            intensity: 1,
        }
    }
}
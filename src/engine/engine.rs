use std::collections::VecDeque;
use std::sync::Arc;

use glam::Vec3;
use rand::Rng;

use crate::physics::Transform;
use crate::rendering::primitives::Cube;
use crate::rendering::renderer::{EngineEvent, EntityUpdateInfo, HasMovedInfo, Renderer};

use super::general_traits::{Entity, TickAction};
use super::scene::Scene;

pub struct Engine {
    entities: Vec<Box<dyn Entity>>,
    active_scene: Option<Arc<Scene>>,
    event_queue: VecDeque<EngineEvent>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            active_scene: None,
            event_queue: VecDeque::new(),
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn active_scene(&self) -> Option<&Arc<Scene>> {
        self.active_scene.as_ref()
    }

    pub fn set_active_scene(&mut self, scene: Arc<Scene>) {
        self.active_scene = Some(scene.clone());
        self.event_queue.push_back(EngineEvent::ChangedActiveScene(scene));
    }

    pub fn tick(&mut self) {
        let mut updates = Vec::new();
        for (entity_index, entity) in self.entities.iter_mut().enumerate() {
            match entity.tick() {
                Some(TickAction::HasMoved(transform)) => {
                    updates.push(EntityUpdateInfo::HasMoved(HasMovedInfo {
                        entity_id: entity_index,
                        new_transform: transform,
                    }));
                }
                None => {}
            }
        }
        if !updates.is_empty() {
            self.event_queue.push_back(EngineEvent::EntitiesUpdated(updates));
        }
    }

    pub fn add_cube_to_scene(&mut self, translation: Option<Vec3>, scale: Option<Vec3>) {
        let translation = translation.unwrap_or_else(|| {
            let mut rng = rand::thread_rng();
            Vec3::new(
                rng.gen_range(-0.5..0.5),
                rng.gen_range(-0.5..1.0),
                rng.gen_range(-2.0..-0.7),
            )
        });

        let scale = scale.unwrap_or_else(|| {
            Vec3::new(1.,1.,1.)
        });

        println!("Scale of the cube to add {}", scale);

        let cube: Cube = Cube::new(
            Transform {
                translation,
                scale,
                ..Default::default()
            },
        );
        let entity_index = self.entities.len();
        self.event_queue.push_back(EngineEvent::EntityAdded(
            cube.transform(),
            cube.mesh(),
            entity_index,
        ));
        self.entities.push(Box::new(cube));
    }

    pub fn add_multiple_cubes_to_scene(&mut self, amount: usize) {
        for _ in 0..amount {
            self.add_cube_to_scene(None, None);
        }
    }

    pub fn work_off_event_queue(&mut self, renderer: &mut Renderer) {
        while let Some(event) = self.event_queue.pop_front() {
            match event {
                EngineEvent::EntityAdded(transform, mesh, entity_index) => {
                    renderer.entity_added_handler(transform, mesh, entity_index)
                }
                EngineEvent::EntitiesUpdated(updates) => {
                    renderer.entities_updated_handler(updates)
                }
                EngineEvent::ChangedActiveScene(scene) => {
                    renderer.changed_active_scene_handler(scene)
                }
            }
        }
    }
}

use canvas::Color;
use flo_draw::*;
use physics::ecs::*;
use physics::bundles::*;

pub struct Start;

impl System for Start {
    fn update(&mut self, entitys: Vec<&mut Bundle>, api: &mut ECS) {
        for i in 0..1000 {
            api.add_entity(atom());
        }
        api.remove_system(self.name());
    }

    fn name(&self) -> String {
        "Start".to_string()
    }

    fn contains(&self) -> Vec<ComponentType> {
        vec![]
    }
}

pub fn main() {
    ECS::start().add(Box::new(Start)).run();
}

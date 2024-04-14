use std::f32::consts::PI;

use crate::render::Instance;

pub struct Animal{
    body_id: usize,
}

impl Animal{
    fn new()->Self{
        Self{
            body_id: 0,
        }
    }
}
pub struct Animals{
    animals: Vec<Animal>,
    bodies: Vec<Instance>,
}
impl Animals{
    pub fn genesis()->Self{
        let animals = (0..100).map(|_i| {
            Animal::new()
        }).collect::<Vec<Animal>>();

        let bodies = (0..100).map(|i| {
            let x = i as f32 % 10.;
            let y = (i as f32/10.).trunc();
            Instance::new([(x*0.15)-0.6, (y*0.15)-0.8], [1.0, 1.0, 1.0], PI,0.1)
        }).collect::<Vec<Instance>>();

        Self{
            animals,
            bodies,
        }
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }
    pub fn count(&self)->usize{
        self.animals.len()
    }
}
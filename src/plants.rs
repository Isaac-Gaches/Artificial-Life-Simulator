use std::f32::consts::PI;

use crate::render::Instance;

pub struct Plant{
    body_id: usize,
}
impl Plant {
    fn new()->Self{
        Self{
            body_id: 0,
        }
    }
}
pub struct Plants{
    plants: Vec<Plant>,
    bodies: Vec<Instance>,
}
impl Plants {
    pub fn genesis()->Self{
        let plants = (0..100).map(|_i| {
            Plant::new()
        }).collect::<Vec<Plant>>();

        let bodies = (0..100).map(|i| {
            let x = i as f32 % 10.;
            let y = (i as f32/10.).trunc();
            Instance::new([(x*0.15)+0.1, (y*0.15)+0.2], [0.0, 1.0, 0.0], -PI/2.0,0.05)
        }).collect::<Vec<Instance>>();

        Self{
            plants,
            bodies,
        }
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn count(&self)->usize{
        self.plants.len()
    }

    pub fn update(&mut self){
        self.bodies.iter_mut().for_each(|instance: &mut Instance| instance.rotation+=0.05)
    }
}
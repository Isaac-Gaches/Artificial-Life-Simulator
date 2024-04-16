use std::f32::consts::PI;
use rayon::prelude::*;
use crate::render::Instance;

pub struct PlantEntity{
    body_id: usize,
}
impl PlantEntity {
    fn new()->Self{
        Self{
            body_id: 0,
        }
    }
}
pub struct Plants{
    entities: Vec<PlantEntity>,
    bodies: Vec<Instance>,
}
impl Plants {
    pub fn genesis()->Self{
        let entities = (0..100).map(|_i| {
            PlantEntity::new()
        }).collect::<Vec<PlantEntity>>();

        let bodies = (0..100).map(|i| {
            let x = i as f32 % 10.;
            let y = (i as f32/10.).trunc();
            Instance::new([(x*0.15)+0.1, (y*0.15)+0.2], [1.0, 0.0, 0.0], -PI/2.0,0.05)
        }).collect::<Vec<Instance>>();

        Self{
            entities,
            bodies,
        }
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn count(&self)->usize{
        self.entities.len()
    }

    pub fn update(&mut self){
        self.bodies.par_iter_mut().for_each(|instance: &mut Instance| instance.rotation+=0.05)
    }
}
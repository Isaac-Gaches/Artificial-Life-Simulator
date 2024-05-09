use std::f32::consts::PI;
use std::ops::Index;
use crate::render::Instance;
use rand::Rng;
use crate::{WORLD_HEIGHT, WORLD_WIDTH};

pub struct Plant{
    pub eaten: bool,
}
pub struct Plants{
    pub plants: Vec<Plant>,
    pub bodies: Vec<Instance>,
}
impl Plants {
    pub fn genesis()->Self{
/*        let plants = (0..200).map(|_|{
            Plant{ eaten: false }
        }).collect();
        let bodies = (0..200).map(|_|{
            Instance::new([rand::thread_rng().gen_range((-8.)..8.), rand::thread_rng().gen_range((-8.)..8.)], [0.0, 1.0, 0.0], PI/4.0,0.04)
        }).collect();*/
        Self{
            plants: vec![],
            bodies: vec![],
        }
    }
    pub fn remove(&mut self, i: usize){
        self.bodies.remove(i);
        self.plants.remove(i);
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn count(&self)->usize{
        self.bodies.len()
    }

    pub fn kill(&mut self){
        (0..self.count()).rev().for_each(|i|{
            if self.plants.index(i).eaten{
                self.remove(i);
            }
        });
    }

    pub fn spawn(&mut self){
        self.bodies.push(Instance::new([rand::thread_rng().gen_range((-WORLD_WIDTH)..WORLD_WIDTH), rand::thread_rng().gen_range((-WORLD_HEIGHT)..WORLD_HEIGHT)], [0.0, 1.0, 0.0], PI/4.0,0.04));
        self.plants.push(Plant{ eaten: false });
    }
}
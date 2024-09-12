use std::f32::consts::PI;
use std::ops::{Index, IndexMut};
use crate::rendering::render::Instance;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::{WORLD_HEIGHT, WORLD_WIDTH};

#[derive(Clone,Serialize,Deserialize)]
pub struct Plant{
    pub eaten: bool,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Plants{
    pub plants: Vec<Plant>,
    pub bodies: Vec<Instance>,
}
impl Plants {
    pub fn genesis()->Self{
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

    pub fn handle_collision(&mut self,plant_id:usize)->(f32,f32){
        self.plants.index_mut(plant_id).eaten = true;
        (70.,1.0)
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
        self.bodies.push(Instance::new([rand::thread_rng().gen_range(0.0..WORLD_WIDTH), rand::thread_rng().gen_range(0.0..WORLD_HEIGHT)], [0.0, 1.0, 0.0], PI/4.0,0.04));
        self.plants.push(Plant{ eaten: false });
    }
}
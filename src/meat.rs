use std::f32::consts::PI;
use std::ops::{Index, IndexMut};
use rayon::prelude::*;
use crate::render::Instance;
use rand::Rng;

pub struct MeatChunk{
    pub eaten: bool,
    time: f32,
}
pub struct Meat{
    pub meat: Vec<MeatChunk>,
    pub bodies: Vec<Instance>,
}
impl Meat {
    pub fn genesis()->Self{
        let meat = (0..200).map(|_|{
            MeatChunk{ eaten: false, time: 0.0 }
        }).collect();
        let bodies = (0..200).map(|_|{
            Instance::new([rand::thread_rng().gen_range((-8.)..8.), rand::thread_rng().gen_range((-8.)..8.)], [1.0, 0.0, 0.0], 0.0,0.04)
        }).collect();
        Self{
            meat,
            bodies,
        }
    }
    pub fn remove(&mut self, i: usize){
        self.bodies.remove(i);
        self.meat.remove(i);
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn count(&self)->usize{
        self.bodies.len()
    }

    pub fn update(&mut self){
        (0..self.count()).rev().for_each(|i|{
            self.meat.index_mut(i).time += 1./60.;
            if self.meat.index(i).eaten || self.meat.index(i).time > 40.{
                self.remove(i);
            }
        });
    }

    pub fn spawn(&mut self,pos: [f32;2]){
        self.bodies.push(Instance::new(pos, [1.0, 0.0, 0.0], 0.0,0.04));
        self.meat.push(MeatChunk{ eaten: false, time: 0.0 });
    }
}
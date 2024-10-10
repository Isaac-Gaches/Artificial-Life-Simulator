use std::ops::{Index, IndexMut};
use serde::{Deserialize, Serialize};
use crate::rendering::render::Instance;
use crate::environment::animal::{Animal, Animals};

#[derive(Serialize,Deserialize,Clone)]
pub struct Egg{
    pub time: f32,
    pub animal: Animal,
}
#[derive(Default,Serialize,Deserialize,Clone)]
pub struct Eggs{
    pub eggs: Vec<Egg>,
    pub bodies: Vec<Instance>,
}
impl Eggs {
    pub fn remove(&mut self, i: usize){
        self.bodies.remove(i);
        self.eggs.remove(i);
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn count(&self)->usize{
        self.bodies.len()
    }

    pub fn update(&mut self,animals: &mut Animals){
        (0..self.count()).rev().for_each(|i|{
            self.eggs.index_mut(i).time += 1./60.;
            if self.eggs.index(i).time > 20.{
                let egg = self.eggs.index(i);
                animals.birth(egg.animal.clone());
                self.remove(i);
            }
        });

    }

    pub fn spawn(&mut self,pos: [f32;2],animal: Animal){
        self.bodies.push(Instance::new(pos, [0.3, 0.3, 0.3], 0.0,0.08));
        self.eggs.push(Egg { time: 0.0, animal});
    }
}
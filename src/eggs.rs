use std::ops::{Index, IndexMut};
use rayon::prelude::*;
use crate::render::Instance;
use rand::Rng;
use crate::animal::{Animal, Animals, SensoryInput};
use crate::neural_network::Network;

pub struct Egg{
    pub time: f32,
    pub body: Instance,
    pub brain: Network,
    pub animal: Animal,
    pub sense: SensoryInput,
}
#[derive(Default)]
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
            if self.eggs.index(i).time > 10.{
                let egg = self.eggs.index(i);
                animals.spawn(egg.body.clone(), egg.sense.clone(), egg.animal.clone(), egg.brain.clone());
                self.remove(i);
            }
        });

    }

    pub fn spawn(&mut self,pos: [f32;2],body: Instance,sense: SensoryInput,animal: Animal,brain:Network){
        self.bodies.push(Instance::new(pos, [0.3, 0.3, 0.3], 0.0,0.04));
        self.eggs.push(Egg { time: 0.0, body, brain, animal, sense });
    }
}
use std::collections::HashMap;
use std::f32::consts::PI;
use std::hash::BuildHasherDefault;
use rayon::prelude::*;
use crate::neural_network::Network;
use crate::render::Instance;
use nohash_hasher::{BuildNoHashHasher, NoHashHasher};

pub struct AnimalEntity {
    id: u16,
}

impl AnimalEntity {
    fn new()->Self{
        Self{
            id: 0,
        }
    }
}
struct SensoryInput{

}
impl SensoryInput{
    fn stimulus(&self) -> Vec<f32>{
        vec![]
    }
}
pub struct Animals{
    entities: Vec<AnimalEntity>,
    bodies: HashMap::<u16, Instance, BuildHasherDefault<NoHashHasher<u16>>>,
    brains: HashMap::<u16, Network, BuildHasherDefault<NoHashHasher<u16>>>,
    senses: HashMap::<u16, SensoryInput, BuildHasherDefault<NoHashHasher<u16>>>,
}

impl Animals{
    pub fn genesis()->Self{
        /*let entities = (0..100).map(|_i| {
            AnimalEntity::new()
        }).collect::<Vec<AnimalEntity>>();

        let bodies = (0..100).map(|i| {
            let x = i as f32 % 10.;
            let y = (i as f32/10.).trunc();
            Instance::new([(x*0.15)-0.6, (y*0.15)-0.8], [1.0, 1.0, 1.0], PI,0.1)
        }).collect::<Vec<Instance>>();*/

        Self{
            entities: vec![],
            bodies: Default::default(),
            brains: Default::default(),
            senses: Default::default(),
        }
    }

    pub fn update(&mut self){
        let responses: Vec<Vec<f32>> = self.brains.par_iter().zip(self.senses.par_iter()).map(|(network,sense)| network.propagate(sense.stimulus())).collect();
    }

    pub fn instances(&self) -> &Vec<Instance>{
        self.bodies.values().collect()
    }

    pub fn count(&self)->usize{
        self.entities.len()
    }
}
use std::ops::{Index, IndexMut};
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::environment::collisions::{CELLS_HEIGHT, CELLS_WIDTH, Collisions, DIV};
use crate::environment::rocks::RockMap;
use crate::rendering::instance::Instance;
use crate::utilities::simulation_parameters::SimParams;

#[derive(Clone,Serialize,Deserialize)]
pub struct Fruit{
    pub eaten: bool,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Fruits{
    pub fruit: Vec<Fruit>,
    pub bodies: Vec<Instance>,
}
impl Fruits {
    pub fn genesis()->Self{
        Self{
            fruit: vec![],
            bodies: vec![],
        }
    }
    pub fn remove(&mut self, i: usize){
        self.bodies.remove(i);
        self.fruit.remove(i);
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn handle_collision(&mut self,plant_id:usize,sim_params: &SimParams)->(f32,f32){
        self.fruit.index_mut(plant_id).eaten = true;
        (sim_params.plants.energy,sim_params.plants.protein)
    }

    pub fn count(&self)->usize{
        self.bodies.len()
    }

    pub fn kill(&mut self){
        (0..self.count()).rev().for_each(|i|{
            if self.fruit.index(i).eaten{
                self.remove(i);
            }
        });
    }

    pub fn spawn(&mut self,rock_map: &RockMap){
        for _trials in 0..100{
            let x = rand::thread_rng().gen_range(0.0..WORLD_WIDTH);
            let y = rand::thread_rng().gen_range(0.0..WORLD_HEIGHT);

            let mut spawn = true;

            'outer: for m in -1..=1{
                for n in -1..=1{
                    let i = (x * DIV + m as f32) as usize * CELLS_HEIGHT + (y * DIV + n as f32) as usize;
                    if rock_map.rocks[i] > 0 {
                        spawn = false;
                        break 'outer;
                    }
                }
            }

            if spawn {
                self.bodies.push(Instance::new([x, y], [0.3, 1.0, 0.0], 0.0, 0.1));
                self.fruit.push(Fruit { eaten: false });
                break;
            }
        }
    }

    pub fn remove_plants_in_walls(&mut self,rock_map: &RockMap){
        (0..self.count()).rev().for_each(|i|{
            let plant = &self.bodies[i];
            'outer: for m in -1..=1{
                for n in -1..=1{
                    let r = (plant.position[0] * DIV + m as f32) as usize * CELLS_HEIGHT + (plant.position[1] * DIV + n as f32) as usize;
                    if rock_map.rocks[r] > 0 {
                        self.remove(i);
                        break 'outer;
                    }
                }
            }
        });
    }
}
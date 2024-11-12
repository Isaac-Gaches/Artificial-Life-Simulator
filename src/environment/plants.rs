use std::ops::{Index, IndexMut};
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::environment::collisions::{CELLS_HEIGHT, CELLS_WIDTH, Collisions, DIV};
use crate::environment::rocks::RockMap;
use crate::rendering::instance::Instance;
use crate::utilities::simulation_parameters::SimParams;

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

    pub fn handle_collision(&mut self,plant_id:usize,sim_params: &SimParams)->(f32,f32){
        self.plants.index_mut(plant_id).eaten = true;
        (sim_params.plants.energy,sim_params.plants.protein)
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

    pub fn spawn(&mut self,rock_map: &RockMap,collisions: &Collisions,sim_params: &SimParams){
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

            if spawn == true {
                let mut plants = 0;

                for m in -2..=2 {
                    for n in -2..=2 {
                        let i = (x * DIV + m as f32) as usize * CELLS_HEIGHT + (y * DIV + n as f32) as usize;
                        plants += collisions.plants_grid[i].count();
                    }
                }

                if plants < (sim_params.plants.max_density * 16.0) as usize {
                    self.bodies.push(Instance::new([x, y], [0.0, 1.0, 0.0], 0.0, 0.06));
                    self.plants.push(Plant { eaten: false });
                    break;
                }
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
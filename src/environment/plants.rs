use std::f32::consts::PI;
use std::ops::{Index, IndexMut};
use cgmath::num_traits::Float;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::environment::collisions::{CELL_SIZE, Collisions, DIV};
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
#[derive(Clone,Serialize,Deserialize)]
pub struct PlantSpawners{
    pub bodies: Vec<Instance>,
}
impl PlantSpawners {
    pub fn spawn(&self,plants: &mut Plants,rock_map: &RockMap,collisions: &Collisions,sim_params: &SimParams){
        self.bodies.iter().for_each(|spawner|{
            plants.spawn_near(rock_map,collisions,sim_params,spawner.position[0],spawner.position[1]);
        })
    }
    pub fn random(&mut self,sim_params: &SimParams){
        if sim_params.world.plant_spawners == 0 {return;}
        for _ in 0..sim_params.world.plant_spawners{
            let x = rand::thread_rng().gen_range(0..(sim_params.world.width*DIV-1.) as u32);
            let y = rand::thread_rng().gen_range(0..(sim_params.world.height*DIV-1.) as u32);

            self.bodies.push(Instance::new([x as f32 * CELL_SIZE+CELL_SIZE*0.5,y as f32 * CELL_SIZE+CELL_SIZE*0.5],[0.0, 0.7, 0.0],PI/4.,CELL_SIZE * 0.9));
        }
    }
    pub fn place(&mut self,pos: [f32;2], sim_params: &SimParams){
        if pos[0] > CELL_SIZE && pos[1] > CELL_SIZE && pos[0] < sim_params.world.width - CELL_SIZE && pos[1] < sim_params.world.height - CELL_SIZE {
            self.bodies.push(Instance::new([(pos[0] * DIV - 0.5).round() * CELL_SIZE + CELL_SIZE * 0.5, (pos[1] * DIV - 0.5).round() * CELL_SIZE + CELL_SIZE * 0.5], [0.0, 0.7, 0.0], PI / 4., CELL_SIZE * 0.9));
        }
    }
    pub fn remove(&mut self,pos: [f32;2]){
        (0..self.count()).rev().for_each(|i|{
            if self.bodies[i].position == [(pos[0]*DIV-0.5).round()*CELL_SIZE+CELL_SIZE*0.5,(pos[1]*DIV-0.5).round()*CELL_SIZE+CELL_SIZE*0.5]{
                self.bodies.remove(i);
            }
        });
    }
    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }
    pub fn count(&self)->usize{
        self.bodies.len()
    }
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

    pub fn spawn_random(&mut self,rock_map: &RockMap, collisions: &Collisions, sim_params: &SimParams){
        for _trials in 0..100{
            let x = rand::thread_rng().gen_range(0.0..sim_params.world.width);
            let y = rand::thread_rng().gen_range(0.0..sim_params.world.height);

            let mut spawn = true;

            'outer: for m in -1..=1{
                for n in -1..=1{
                    let i = (x * DIV + m as f32) as usize * collisions.cells_height + (y * DIV + n as f32) as usize;
                    if rock_map.rocks[i] > 0 {
                        spawn = false;
                        break 'outer;
                    }
                }
            }

            if spawn && collisions.plants_grid[(x * DIV) as usize * collisions.cells_height + (y * DIV) as usize].count() < 2 {
                self.bodies.push(Instance::new([x, y], [0.0, 0.7, 0.0], 0.0, 0.06));
                self.plants.push(Plant { eaten: false });
                break;
            }
        }
    }

    pub fn spawn_near(&mut self,rock_map: &RockMap, collisions: &Collisions, sim_params: &SimParams,sx: f32, sy: f32){
        let mut rng = rand::thread_rng();
        for _trials in 0..10{
            let x = (sx + rand::thread_rng().gen_range(-sim_params.plants.spawn_radius..=sim_params.plants.spawn_radius)).clamp(0.,sim_params.world.width);
            let y = (sy + rand::thread_rng().gen_range(-sim_params.plants.spawn_radius..=sim_params.plants.spawn_radius)).clamp(0.,sim_params.world.height);

            let mut spawn = true;

            if !rng.gen_bool((((x-sx).powf(2.) + (y-sy).powf(2.))/(sim_params.plants.spawn_radius*sim_params.plants.spawn_radius)).min(1.0) as f64) {
                'outer: for m in -1..=1{
                    for n in -1..=1{
                        let i = (x * DIV + m as f32) as usize * collisions.cells_height + (y * DIV + n as f32) as usize;
                        if rock_map.rocks[i] > 0 {
                            spawn = false;
                            break 'outer;
                        }
                    }
                }
                if spawn && collisions.plants_grid[(x * DIV) as usize * collisions.cells_height + (y * DIV) as usize].count() < 2 {
                    self.bodies.push(Instance::new([x, y], [0.0, 0.7, 0.0], 0.0, 0.06));
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
                    let r = (plant.position[0] * DIV + m as f32) as usize * rock_map.height + (plant.position[1] * DIV + n as f32) as usize;
                    if rock_map.rocks[r] > 0 {
                        self.remove(i);
                        break 'outer;
                    }
                }
            }
        });
    }
}
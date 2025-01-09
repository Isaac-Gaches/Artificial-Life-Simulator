use std::f32::consts::PI;
use std::ops::{Index, IndexMut};
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::environment::collisions::{CELL_SIZE, Collisions, DIV};
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
#[derive(Clone,Serialize,Deserialize)]
pub struct FruitSpawners{
    pub bodies: Vec<Instance>,
}
impl FruitSpawners {
    pub fn spawn(&self,fruit: &mut Fruits,rock_map: &RockMap,collisions: &Collisions,sim_params: &SimParams){
        self.bodies.iter().for_each(|spawner|{
            fruit.spawn_near(rock_map,collisions,sim_params,spawner.position[0],spawner.position[1]);
        })
    }
    pub fn random(&mut self,sim_params: &SimParams){
        for _ in 0..5{
            let x = rand::thread_rng().gen_range(0..(sim_params.world.width*DIV) as u32);
            let y = rand::thread_rng().gen_range(0..(sim_params.world.height*DIV) as u32);

            self.bodies.push(Instance::new([x as f32 * CELL_SIZE+CELL_SIZE*0.5,y as f32 * CELL_SIZE+CELL_SIZE*0.5],[0.3, 1.0, 0.0],PI/4.,CELL_SIZE*1.1));
        }
    }
    pub fn place(&mut self,pos: [f32;2],sim_params: &SimParams){
        if pos[0] > CELL_SIZE && pos[1] > CELL_SIZE && pos[0] < sim_params.world.width - CELL_SIZE && pos[1] < sim_params.world.height - CELL_SIZE {
            self.bodies.push(Instance::new([(pos[0] * DIV - 0.5).round() * CELL_SIZE + CELL_SIZE * 0.5, (pos[1] * DIV - 0.5).round() * CELL_SIZE + CELL_SIZE * 0.5], [0.3, 1.0, 0.0], PI / 4., CELL_SIZE * 1.1));
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
        (sim_params.fruit.energy,sim_params.fruit.protein)
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

            if spawn && collisions.fruit_grid[(x * DIV) as usize * collisions.cells_height + (y * DIV) as usize].count() < 2 {
                self.bodies.push(Instance::new([x, y], [0.3, 1.0, 0.0], 0.0, 0.1));
                self.fruit.push(Fruit { eaten: false });
                break;
            }
        }
    }

    pub fn spawn_near(&mut self,rock_map: &RockMap, collisions: &Collisions, sim_params: &SimParams,sx: f32, sy: f32){
        let mut rng = rand::thread_rng();
        for _trials in 0..10{
            let x = (sx + rand::thread_rng().gen_range(-sim_params.fruit.spawn_radius..=sim_params.fruit.spawn_radius)).clamp(0.,sim_params.world.width);
            let y = (sy + rand::thread_rng().gen_range(-sim_params.fruit.spawn_radius..=sim_params.fruit.spawn_radius)).clamp(0.,sim_params.world.height);

            let mut spawn = true;

            if rng.gen_bool((((x-sx).powf(2.) + (y-sy).powf(2.))/(sim_params.fruit.spawn_radius*sim_params.fruit.spawn_radius)).min(1.0) as f64){
                spawn = false;
            }
            else {
                'outer: for m in -1..=1{
                    for n in -1..=1{
                        let i = (x * DIV + m as f32) as usize * collisions.cells_height + (y * DIV + n as f32) as usize;
                        if rock_map.rocks[i] > 0 {
                            spawn = false;
                            break 'outer;
                        }
                    }
                }
                if spawn && collisions.fruit_grid[(x * DIV) as usize * collisions.cells_height + (y * DIV) as usize].count() < 1 {
                    self.bodies.push(Instance::new([x, y], [0.3, 1., 0.0], 0.0, 0.1));
                    self.fruit.push(Fruit { eaten: false });
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
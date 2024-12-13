use serde::{Deserialize, Serialize};
use sysinfo::System;
use crate::environment::animal::Animal;
#[derive(Serialize,Deserialize,Clone)]
pub struct Stats{
    pub populations: Populations,
    pub distributions: Distributions,
    pub fps: usize,
    pub used_mem: u64,
    pub tot_mem: u64,
    pub cpu_usages: Vec<f32>,
    pub tot_cpu_usage: f32,
    step: usize,
    pub step_time: usize,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct Distributions{
    pub diet: Vec<f64>,
    pub speed: Vec<f64>,
    pub size: Vec<f64>,
    pub attack: Vec<f64>,
    pub fruit_vision: Vec<f64>,
    pub plant_vision: Vec<f64>,
    pub animal_vision: Vec<f64>,
    pub rock_vision: Vec<f64>,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct Populations{
    pub animals: Vec<[f64;2]>,
    pub plants: Vec<[f64;2]>,
    pub herbivores: Vec<[f64;2]>,
    pub omnivores: Vec<[f64;2]>,
    pub carnivores: Vec<[f64;2]>,
    pub slow: Vec<[f64;2]>,
    pub moderate_speed: Vec<[f64;2]>,
    pub fast: Vec<[f64;2]>,
    pub small: Vec<[f64;2]>,
    pub medium: Vec<[f64;2]>,
    pub large: Vec<[f64;2]>,
}

impl Default for Stats{
    fn default() -> Self {
        Self{
            populations: Populations{
                animals: vec![],
                plants: vec![],
                herbivores: vec![],
                omnivores: vec![],
                carnivores: vec![],
                slow: vec![],
                moderate_speed: vec![],
                fast: vec![],
                small: vec![],
                medium: vec![],
                large: vec![],
            },
            distributions: Distributions{
                diet: vec![0.;11],
                speed: vec![0.;11],
                size: vec![0.;11],
                attack: vec![0.;11],
                fruit_vision: vec![0.;13],
                plant_vision: vec![0.;13],
                animal_vision: vec![0.;13],
                rock_vision: vec![0.;13],
            },
            fps: 0,
            used_mem: 0,
            tot_mem: 0,
            cpu_usages: vec![],
            tot_cpu_usage: 0.0,
            step: 0,
            step_time: 1,
        }
    }
}
impl Stats{
    pub fn clear_graph_data(&mut self){
        self.populations.animals = vec![];
        self.populations.plants = vec![];
        self.populations.herbivores = vec![];
        self.populations.omnivores = vec![];
        self.populations.carnivores = vec![];
        self.populations.slow = vec![];
        self.populations.moderate_speed = vec![];
        self.populations.fast = vec![];
    }
    pub fn update_diagnostics(&mut self, frames: usize,system: &mut System){
        system.refresh_memory();
        system.refresh_cpu_usage();
        self.cpu_usages = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect::<Vec<f32>>();
        self.tot_cpu_usage = self.cpu_usages.iter().sum::<f32>()/self.cpu_usages.len() as f32;
        self.tot_mem = system.total_memory();
        self.used_mem = system.used_memory();
        self.fps = frames;
    }
    pub fn update_graphs(&mut self, animal_population: usize, plant_population: usize, animals: &[Animal]){
        if self.step % self.step_time == 0 {
            self.populations.animals.push([self.step as f64, animal_population as f64]);
            self.populations.plants.push([self.step as f64, plant_population as f64]);
            self.distributions.diet = vec![0.;11];
            self.distributions.speed = vec![0.;11];
            let (mut herb,mut omni,mut carn) = (0.,0.,0.);
            let (mut slow,mut moderate,mut fast) = (0.,0.,0.);
            animals.iter().for_each(|animal|{
                let diet = (animal.combat_stats.carnivore_factor * 10.).round() as usize;
                self.distributions.diet[diet] += 1.;
                if diet < 4{
                    herb += 1.0;
                }
                else if diet < 7{
                    omni += 1.0;
                }
                else{
                    carn += 1.0;
                }

                let speed = (animal.combat_stats.speed * 2.5).round() as usize;
                self.distributions.speed[speed] += 1.;
                if speed < 3{
                    slow += 1.0;
                }
                else if speed < 6{
                    moderate += 1.0;
                }
                else{
                    fast += 1.0;
                }
            });
            self.populations.herbivores.push([self.step as f64, herb]);
            self.populations.omnivores.push([self.step as f64, omni]);
            self.populations.carnivores.push([self.step as f64, carn]);

            self.populations.slow.push([self.step as f64, slow]);
            self.populations.moderate_speed.push([self.step as f64, moderate]);
            self.populations.fast.push([self.step as f64, fast]);
        }
        self.step+=1
    }
}

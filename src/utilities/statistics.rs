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
    pub fruit: Vec<[f64;2]>,
    pub herbivores: Vec<[f64;2]>,
    pub omnivores: Vec<[f64;2]>,
    pub carnivores: Vec<[f64;2]>,
    pub average_speed: Vec<[f64;2]>,
    pub average_size: Vec<[f64;2]>,
}

impl Default for Stats{
    fn default() -> Self {
        Self{
            populations: Populations{
                animals: vec![],
                plants: vec![],
                fruit: vec![],
                herbivores: vec![],
                omnivores: vec![],
                carnivores: vec![],
                average_speed: vec![],
                average_size: vec![],
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
        self.populations.fruit = vec![];
        self.populations.herbivores = vec![];
        self.populations.omnivores = vec![];
        self.populations.carnivores = vec![];
        self.populations.average_size = vec![];
        self.populations.average_speed = vec![];
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
    pub fn update_graphs(&mut self, animal_population: usize, fruit_population: usize, plant_population: usize, animals: &[Animal]){
        if self.step % self.step_time == 0 {
            self.populations.animals.push([self.step as f64, animal_population as f64]);
            self.populations.plants.push([self.step as f64, plant_population as f64]);
            self.populations.fruit.push([self.step as f64, fruit_population as f64]);

            self.distributions.diet = vec![0.;11];
            self.distributions.speed = vec![0.;11];
            self.distributions.size = vec![0.;11];

            let (mut herb,mut omni,mut carn) = (0.,0.,0.);
            let (mut slow,mut moderate,mut fast) = (0.,0.,0.);
            let (mut small,mut medium,mut large) = (0.,0.,0.);

            let mut avg_speed = 0.;
            let mut avg_size = 0.;

            animals.iter().for_each(|animal|{
                let diet = (animal.combat_stats.carnivore_factor * 10.).round() as usize;
                update_stats(diet,&mut herb,&mut omni ,&mut carn,&mut self.distributions.diet);

                let speed = (animal.combat_stats.speed - 0.5)/3.5;
                avg_speed += speed;
                update_stats((speed*10.).round() as usize,&mut slow,&mut moderate,&mut fast,&mut self.distributions.speed);

                let size = (animal.body.scale - 0.08)/0.42;
                avg_size += size;
                update_stats((size*10.).round() as usize,&mut small,&mut medium,&mut large,&mut self.distributions.size);
            });

            self.populations.herbivores.push([self.step as f64, herb]);
            self.populations.omnivores.push([self.step as f64, omni]);
            self.populations.carnivores.push([self.step as f64, carn]);

            if !animals.is_empty() {
                avg_speed /= animals.len() as f32;
                avg_size /= animals.len() as f32;
            }
            self.populations.average_speed.push([self.step as f64, avg_speed as f64]);
            self.populations.average_size.push([self.step as f64,avg_size as f64]);
        }
        self.step+=1
    }
}

fn update_stats(stat: usize,low: &mut f64, medium: &mut f64, high: &mut f64, distribution: &mut Vec<f64>){
    distribution[stat] += 1.;
    if stat < 4{
        *low += 1.0;
    }
    else if stat < 7{
        *medium += 1.0;
    }
    else{
        *high += 1.0;
    }
}

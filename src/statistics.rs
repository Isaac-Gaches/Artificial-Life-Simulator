use sysinfo::System;
use crate::animal::Animal;

pub struct Stats{
    pub fps: usize,
    pub animal_pop: Vec<[f64;2]>,
    pub plant_pop: Vec<[f64;2]>,
    pub herb_pop: Vec<[f64;2]>,
    pub omni_pop: Vec<[f64;2]>,
    pub carn_pop: Vec<[f64;2]>,
    system: System,
    pub used_mem: u64,
    pub tot_mem: u64,
    pub cpu_usages: Vec<f32>,
    pub tot_cpu_usage: f32,
    step: usize,
    pub step_time: usize,
    pub diet_dist: Vec<f64>,
}
impl Default for Stats{
    fn default() -> Self {
        Self{
            fps: 0,
            animal_pop: vec![],
            plant_pop: vec![],
            herb_pop: vec![],
            omni_pop: vec![],
            carn_pop: vec![],
            system: Default::default(),
            used_mem: 0,
            tot_mem: 0,
            cpu_usages: vec![],
            tot_cpu_usage: 0.0,
            step: 0,
            step_time: 2,
            diet_dist: vec![0.;11],
        }
    }
}
impl Stats{
    pub fn clear_graph_data(&mut self){
        self.animal_pop = vec![];
        self.plant_pop = vec![];
        self.herb_pop = vec![];
        self.omni_pop = vec![];
        self.carn_pop = vec![];
    }
    pub fn update_diagnostics(&mut self, frames: usize){
        self.system.refresh_memory();
        self.system.refresh_cpu_usage();
        self.cpu_usages = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect::<Vec<f32>>();
        self.tot_cpu_usage = self.cpu_usages.iter().sum::<f32>()/self.cpu_usages.len() as f32;
        self.tot_mem = self.system.total_memory();
        self.used_mem = self.system.used_memory();
        self.fps = frames;
    }
    pub fn update_graphs(&mut self, animal_population: usize, plant_population: usize, animals: &[Animal]){
        if self.step % self.step_time == 0 {
            self.animal_pop.push([self.step as f64, animal_population as f64]);
            self.plant_pop.push([self.step as f64, plant_population as f64]);
            self.diet_dist = vec![0.;11];
            let (mut herb,mut omni,mut carn) = (0.,0.,0.);
            animals.iter().for_each(|animal|{
                let diet = (animal.combat_stats.carnivore_factor * 10.).round() as usize;
                self.diet_dist[diet] += 1.;
                if diet < 4{
                    herb += 1.0;
                }
                else if diet < 7{
                    omni += 1.0;
                }
                else{
                    carn += 1.0;
                }
            });
            self.herb_pop.push([self.step as f64, herb]);
            self.omni_pop.push([self.step as f64, omni]);
            self.carn_pop.push([self.step as f64, carn]);
        }
        self.step+=1
    }
}

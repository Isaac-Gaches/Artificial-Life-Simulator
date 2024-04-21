use egui_winit::State;
use sysinfo::{
    System,
};

pub struct Stats{
    pub fps: usize,
    pub animal_pop: Vec<[f64;2]>,
    pub plant_pop: Vec<[f64;2]>,
    system: System,
    pub used_mem: u64,
    pub tot_mem: u64,
    pub cpu_usages: Vec<f32>,
    pub tot_cpu_usage: f32,
    step: usize,
    pub step_time: usize,
}
impl Default for Stats{
    fn default() -> Self {
        Self{
            fps: 0,
            animal_pop: vec![],
            plant_pop: vec![],
            system: Default::default(),
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
    pub fn update(&mut self,frames: usize,animal_population: usize, plant_population: usize){
        if self.step % self.step_time == 0 {
            self.system.refresh_cpu_usage();
            self.system.refresh_memory();
            self.cpu_usages = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect::<Vec<f32>>();
            self.tot_cpu_usage = self.cpu_usages.iter().sum();
            self.tot_mem = self.system.total_memory();
            self.used_mem = self.system.used_memory();
            self.fps = frames;
            self.animal_pop.push([self.step as f64, animal_population as f64]);
            self.plant_pop.push([self.step as f64, plant_population as f64]);
        }
        self.step+=1;
    }
}

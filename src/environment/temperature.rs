use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use crate::environment::collisions::{CELL_SIZE, DIV};
use crate::environment::fruit::FruitSpawners;
use crate::environment::plants::PlantSpawners;
use crate::rendering::instance::Instance;

#[derive(Serialize,Deserialize,Clone)]
pub struct TemperatureMap{
    pub instances: Vec<Instance>,
    cells: Vec<f32>,
    size: usize
}
impl  TemperatureMap{
    pub fn new(size: usize)-> Self{
        Self{
            instances: vec![],
            cells: vec![0.; size*size],
            size,
        }
    }
    pub fn set(&mut self,temp: f32 ,bodies: &Vec<Instance>){
        for body in bodies{
            let i = (body.position[0] * DIV) as usize * self.size + (body.position[1] * DIV) as usize;
            self.cells[i] = temp;
        }
    }
    pub fn smooth(&mut self,rocks: &Vec<u8>){
        let mut new_temp = vec![0.;self.size*self.size];
        for x in 1..self.size-1{
            for y in 1..self.size-1{
                let mut avg = 0.;
                for i in -1..=1{
                    for j in -1..=1{
                        let n = ((x as i32+i) * self.size as i32 + (y as i32+j)) as usize;
                        avg += self.cells[n];
                        if rocks[n] != 0 {
                            avg -= 0.8;
                        }
                    }
                }
                avg/=9.;
                new_temp[x * self.size + y] = avg;
            }
        }
        self.cells = new_temp;
    }
    pub fn diffuse(&mut self,decay:f32,rocks: &Vec<u8>){
         //up
         for x in 1..self.size-1{
             for y in 1..self.size-1{
                 let i = x * self.size + y;
                 if rocks[i] == 0{
                     let j = x * self.size + y-1;
                     let t = self.cells[j]* decay;
                     self.cells[i] = self.cells[i].max(t);
                 }
             }
         }
         //down
         for x in 1..self.size-1{
             for y in (1..self.size-1).rev(){
                 let i = x * self.size + y;
                 if rocks[i] == 0 {
                     let j = x * self.size + y + 1;
                     let t = self.cells[j] * decay;
                     self.cells[i] = self.cells[i].max(t);
                 }
             }
         }
         //left
         for y in 1..self.size-1{
             for x in (1..self.size-1).rev(){
                 let i = x * self.size + y;
                 if rocks[i] == 0 {
                     let j = (x + 1) * self.size + y;
                     let t = self.cells[j] * decay;
                     self.cells[i] = self.cells[i].max(t);
                 }
             }
         }
         //right
         for y in 1..self.size-1{
             for x in 1..self.size-1{
                 let i = x * self.size + y;
                 if rocks[i] == 0 {
                     let j = (x - 1) * self.size + y;
                     let t = self.cells[j] * decay;
                     self.cells[i] = self.cells[i].max(t);
                 }
             }
         }
    }
    pub fn update(&mut self,decay:f32,smooth: u8,rocks:&Vec<u8>){
        self.diffuse(decay,rocks);
        self.diffuse(decay,rocks);
        self.diffuse(decay,rocks);
        self.diffuse(decay,rocks);

        for _ in 0..smooth{
            self.smooth(rocks);
        }

        self.update_instances();
    }
    pub fn instances(&self)->&Vec<Instance>{
        &self.instances
    }
    fn update_instances(&mut self){
        let instances = self.cells.par_iter().enumerate().map(|(i,temp)|{
            let x = (i / self.size) as f32 * CELL_SIZE;
            let y = (i % self.size) as f32 * CELL_SIZE;
            Instance::new([x+CELL_SIZE*0.5 , y+CELL_SIZE*0.5], [0., *temp/200. + 0.02, *temp/80. + 0.18], 0.0, CELL_SIZE)

        }).collect();
        self.instances = instances;
    }
    pub fn clear(&mut self){
        self.cells = vec![0.;self.size*self.size];
    }
}


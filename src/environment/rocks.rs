use std::ops::Index;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use simdnoise::NoiseBuilder;
use crate::environment::collisions::{CELL_SIZE, CELLS_HEIGHT, CELLS_WIDTH};
use crate::rendering::render::Instance;

#[derive(Serialize, Deserialize,Clone)]
pub struct RockMap{
    rocks: Vec<u8>
}
impl RockMap{
    pub fn new()->Self{
        Self{
            rocks: vec![0;CELLS_WIDTH*CELLS_HEIGHT],
        }
    }
    pub fn randomise(&mut self){
        let seed = thread_rng().gen_range(-100000..100000);

        let noise = NoiseBuilder::fbm_2d_offset(seed as f32,CELLS_WIDTH,seed as f32, CELLS_HEIGHT).with_seed(seed).with_freq(0.04).generate_scaled(0.0, 1.0);

        for i in 0..self.count() as usize{
            let x = i % CELLS_WIDTH;
            let y = i / CELLS_HEIGHT;
            if x == 0 || x == CELLS_WIDTH - 1 || y == 0 || y == CELLS_HEIGHT - 1 || (*noise.index(i) > 0.5 && *noise.index(i) < 0.6){
                self.rocks[i] =1 ;
            }
        }
    }
    pub fn instances(&self)->Vec<Instance>{
        self.rocks.iter().enumerate().filter_map(|(i,rock)|{
            if *rock == 0 { return None }
            let x = (i % CELLS_WIDTH) as f32 * CELL_SIZE;
            let y = (i / CELLS_HEIGHT) as f32 * CELL_SIZE;
            Some(Instance::new([x, y], [0.3, 0.3, 0.3], 0.0, CELL_SIZE))
        }).collect()
    }
    pub fn count(&self) -> u32{
        self.rocks.len() as u32
    }
}
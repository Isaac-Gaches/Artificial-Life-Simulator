use std::ops::Index;
use rand::{Rng, thread_rng};
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use simdnoise::NoiseBuilder;
use crate::environment::collisions::{CELL_SIZE, CELLS_HEIGHT, CELLS_WIDTH, DIV};
use crate::rendering::instance::Instance;
use crate::{WORLD_HEIGHT, WORLD_WIDTH};

#[derive(Serialize, Deserialize,Clone)]
pub struct RockMap{
    pub rocks: Vec<u8>
}
impl RockMap{
    pub fn new()->Self{
        Self{
            rocks: vec![0;CELLS_WIDTH*CELLS_HEIGHT],
        }
    }
    pub fn randomise(&mut self){
        let seed = thread_rng().gen_range(-100000..100000);

        let noise = NoiseBuilder::fbm_2d_offset(seed as f32,CELLS_WIDTH,seed as f32, CELLS_HEIGHT).with_seed(seed).with_freq(0.1).generate_scaled(0.0, 1.0);
        let noise2 = NoiseBuilder::fbm_2d_offset(seed as f32,CELLS_WIDTH,seed as f32, CELLS_HEIGHT).with_seed(seed).with_freq(0.02).generate_scaled(0.0, 1.0);

        for i in 0..self.count() as usize{
            let x = i % CELLS_WIDTH;
            let y = i / CELLS_HEIGHT;
            if x == 0 || x == CELLS_WIDTH - 1 || y == 0 || y == CELLS_HEIGHT - 1 || (*noise.index(i) > 0.85 || (*noise2.index(i) > 0.51 && *noise2.index(i) < 0.56)){
                self.rocks[i] =1 ;
            }
        }
    }
    pub fn instances(&self)->Vec<Instance>{
        self.rocks.par_iter().enumerate().filter_map(|(i,rock)|{
            if *rock == 0 { return None }
            let x = (i / CELLS_HEIGHT) as f32 * CELL_SIZE;
            let y = (i % CELLS_HEIGHT) as f32 * CELL_SIZE;
            Some(Instance::new([x+CELL_SIZE*0.5 , y+CELL_SIZE*0.5], [0.3, 0.3, 0.3], 0.0, CELL_SIZE))
        }).collect()
    }
    pub fn count(&self) -> u32{
        self.rocks.len() as u32
    }

    pub fn set(&mut self,id:u8, pos: [f32;2],splat: i32) {
        if pos[0] > CELL_SIZE && pos[0] < WORLD_WIDTH-CELL_SIZE && pos[1] > CELL_SIZE && pos[1] < WORLD_HEIGHT-CELL_SIZE{
            if splat > 0 {
                for x in -splat..=splat {
                    for y in -splat..=splat {
                        if pos[0] + x as f32*CELL_SIZE > CELL_SIZE && pos[0] + (x as f32)*CELL_SIZE < WORLD_WIDTH-CELL_SIZE && pos[1] + y as f32*CELL_SIZE > CELL_SIZE && pos[1] + (y as f32)*CELL_SIZE < WORLD_HEIGHT-CELL_SIZE{
                            let i = ((pos[0] * DIV) as i32 + x) as usize * CELLS_HEIGHT + ((pos[1] * DIV) as i32 + y) as usize;
                            self.rocks[i] = id;
                        }
                    }
                }
            } else {
                let i = (pos[0] * DIV) as usize * CELLS_HEIGHT + (pos[1] * DIV) as usize;
                self.rocks[i] = id;
            }
        }
    }
}
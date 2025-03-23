use rand::{Rng, thread_rng};
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use simdnoise::NoiseBuilder;
use crate::environment::collisions::{CELL_SIZE, DIV};
use crate::rendering::instance::Instance;

#[derive(Serialize, Deserialize,Clone)]
pub struct RockMap{
    pub instances: Vec<Instance>,
    pub rocks: Vec<u8>,
    width: usize,
    pub height: usize,
}
impl RockMap{
    pub fn new(size: usize)->Self{
        let mut rocks = vec![0;size*size];
        let width = size;
        let height = size;

        for i in 0..rocks.len(){
            let x = i % width;
            let y = i / height;
            if x == 0 || x == width- 1 || y == 0 || y == height - 1{
                rocks[i] =1 ;
            }
        }

        Self{
            instances: vec![],
            rocks,
            width,
            height,
        }
    }
    pub fn randomise(&mut self){
        let seed = thread_rng().gen_range(-100000..100000);

        let high_freq_noise = NoiseBuilder::fbm_2d_offset(seed as f32, self.width, seed as f32, self.height)
            .with_seed(seed).with_freq(0.1)
            .generate_scaled(0.0, 1.0);

        let low_freq_noise = NoiseBuilder::fbm_2d_offset(seed as f32, self.width, seed as f32, self.height)
            .with_seed(seed).with_freq(0.02)
            .generate_scaled(0.0, 1.0);

        for i in 0..self.rocks.len(){
            if high_freq_noise[i] > 0.85 || (low_freq_noise[i] > 0.51 && low_freq_noise[i] < 0.56){
                self.rocks[i] =1 ;
            }
        }

        self.update_instances();
    }
    fn update_instances(&mut self){
        let instances = self.rocks.par_iter().enumerate().filter_map(|(i,rock)|{
            if *rock == 1{
                let x = (i / self.height) as f32 * CELL_SIZE;
                let y = (i % self.height) as f32 * CELL_SIZE;
                Some(Instance::new([x+CELL_SIZE*0.5 , y+CELL_SIZE*0.5], [0.3, 0.3, 0.3], 0.0, CELL_SIZE))
            }
            else {
                None
            }
        }).collect();
        self.instances = instances;
    }
    pub fn instances(&self)-> &Vec<Instance>{
        &self.instances
    }
    pub fn count(&self) -> u32{
        let mut count = 0;
        self.rocks.iter().for_each(|i|{
            if *i == 1{
                count += 1;
            }
        });
        count
    }

    pub fn set(&mut self,id:u8, pos: [f32;2],splat: i32) -> bool{
        if pos[0] > CELL_SIZE && pos[0] < (CELL_SIZE*self.width as f32)-CELL_SIZE && pos[1] > CELL_SIZE && pos[1] < (CELL_SIZE*self.height as f32)-CELL_SIZE && id != self.rocks[(pos[0] * DIV) as usize * self.height + (pos[1] * DIV) as usize]{
            if splat > 0 {
                for x in -splat..=splat {
                    for y in -splat..=splat {
                        if pos[0] + x as f32*CELL_SIZE > CELL_SIZE && pos[0] + (x as f32)*CELL_SIZE < (CELL_SIZE*self.height as f32)-CELL_SIZE && pos[1] + y as f32*CELL_SIZE > CELL_SIZE && pos[1] + (y as f32)*CELL_SIZE < (CELL_SIZE*self.height as f32)-CELL_SIZE{
                            let i = ((pos[0] * DIV) as i32 + x) as usize * self.height + ((pos[1] * DIV) as i32 + y) as usize;
                            self.rocks[i] = id;

                        }
                    }
                }
            } else {
                let i = (pos[0] * DIV) as usize * self.height + (pos[1] * DIV) as usize;
                self.rocks[i] = id;
            }
            self.update_instances();
            return true
        }
        false
    }
}
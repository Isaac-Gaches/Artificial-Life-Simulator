use std::cell::UnsafeCell;
use std::ops::{Index, IndexMut};
use rayon::prelude::*;
use crate::animal::Animals;
use crate::plants::Plants;
use crate::render::Instance;
use crate::{WORLD_HEIGHT, WORLD_WIDTH};

const CELLS_HEIGHT: usize = ((WORLD_HEIGHT*2.)/CELL_SIZE) as usize + 1;
const CELLS_WIDTH: usize = ((WORLD_WIDTH*2.)/CELL_SIZE) as usize + 1;
const CELL_SIZE: f32 = 0.2;
const DIV: f32 = 1.0/CELL_SIZE;

pub struct Collisions{
    animals_grid: UnsafeCell<Vec<Cell>>,
    plants_grid: UnsafeCell<Vec<Cell>>,
}
#[derive(Default,Clone)]
struct Cell{
    object_ids: Vec<usize>,
}
impl Cell{
    fn index(&self,i: usize) -> usize{
        self.object_ids[i]
    }
    fn clear(&mut self){
        self.object_ids.clear();
    }
    fn add(&mut self, id: usize){
        self.object_ids.push(id);
    }
    fn count(&self)-> usize{
        self.object_ids.len()
    }
}
impl Collisions{
    pub fn new()-> Self{
        Self{
            animals_grid: UnsafeCell::new(vec![Cell::default();CELLS_HEIGHT*CELLS_WIDTH]),
            plants_grid: UnsafeCell::new(vec![Cell::default();CELLS_HEIGHT*CELLS_WIDTH]),
        }
    }
    fn get_grid(&mut self,id: usize) -> &mut Vec<Cell>{
        match id{
            0 =>{
                self.animals_grid.get_mut()
            }
            _ => {
                self.plants_grid.get_mut()
            }
        }
    }
    pub fn update_grid(&mut self,objects: &[Instance],grid_id: usize){
        let grid = self.get_grid(grid_id);

        grid.par_iter_mut().for_each(|cell| cell.clear());

        objects.iter().enumerate().for_each(|(id,instance)|{
            let i = (instance.position[0] * DIV ) as usize * CELLS_HEIGHT + (instance.position[1] * DIV ) as usize;
            grid[i].add(id);
        });
    }

    pub fn collisions(&mut self,animals: &mut Animals, plants: &mut Plants){
        for x in 0..CELLS_WIDTH{
            for y in 0..CELLS_HEIGHT{
                for z in 0..self.get_grid(0)[x * CELLS_HEIGHT + y].count(){
                    let animal_id = self.get_grid(0)[x * CELLS_HEIGHT + y].index(z);
                    let animal_body = animals.bodies[animal_id];

                    for i in 0..3{
                        for j in 0..3{
                            let grid_index = (x+i).saturating_sub(1) * CELLS_HEIGHT + (y+j).saturating_sub(1);

                            for k in 0..self.get_grid(0)[grid_index].count(){
                                if k == z {continue}
                                let other_animal_id = self.get_grid(0)[grid_index].index(k);
                                let other_animal_body = animals.bodies[other_animal_id];

                                let collision_axis_x = other_animal_body.position[0] - animal_body.position[0];
                                let collision_axis_y = other_animal_body.position[1] - animal_body.position[1];

                                let distance = (collision_axis_x*collision_axis_x + collision_axis_y*collision_axis_y).sqrt();

                                if distance < (other_animal_body.scale + animal_body.scale) * 0.5{
                                    let attack_i = animals.animals.index(animal_id).aggression * animal_body.scale;
                                    let attack_j = animals.animals.index(other_animal_id).aggression * other_animal_body.scale;
                                    if attack_i > attack_j && animals.animals.index(animal_id).energy > 0.{
                                        animals.animals.index_mut(animal_id).energy += animals.animals.index(other_animal_id).energy * (1.3*animals.animals.index(animal_id).carnivore_factor).min(1.0);
                                        animals.animals.index_mut(other_animal_id).energy = 0.;
                                    }
                                    else if attack_j > attack_i && animals.animals.index(other_animal_id).energy > 0.{
                                        animals.animals.index_mut(other_animal_id).energy += animals.animals.index(animal_id).energy * (1.3*animals.animals.index(other_animal_id).carnivore_factor).min(1.0);
                                        animals.animals.index_mut(animal_id).energy = 0.;
                                    }
                                }
                            }
                        }
                    }

                    for i in 0..3{
                        for j in 0..3{
                            let grid_index = (x+i).saturating_sub(1) * CELLS_HEIGHT + (y+j).saturating_sub(1);

                            for k in 0..self.get_grid(1)[grid_index].count(){
                                let plant_id = self.get_grid(1)[grid_index].index(k);
                                let plant_body = plants.bodies[plant_id];

                                let relative_pos_x = plant_body.position[0] - animal_body.position[0];
                                let relative_pos_y = plant_body.position[1] - animal_body.position[1];

                                if (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y) < 0.05 * animal_body.scale && !plants.plants[plant_id].eaten{
                                    plants.plants[plant_id].eaten = true;
                                    animals.animals[animal_id].energy += 45. * (1.-animals.animals[animal_id].carnivore_factor);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
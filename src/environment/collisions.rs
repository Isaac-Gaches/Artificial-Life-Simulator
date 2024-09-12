use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animals;
use crate::environment::plants::Plants;
use crate::rendering::render::Instance;
use crate::{WORLD_HEIGHT, WORLD_WIDTH};

pub const CELLS_HEIGHT: usize = (WORLD_HEIGHT/CELL_SIZE) as usize + 2;
pub const CELLS_WIDTH: usize = (WORLD_WIDTH/CELL_SIZE) as usize + 2;
pub const CELL_SIZE: f32 = 0.4;
pub const DIV: f32 = 1.0/CELL_SIZE;

#[derive(Serialize,Deserialize,Clone)]
pub struct Collisions{
    pub animals_grid: Vec<Cell>,
    pub plants_grid: Vec<Cell>,
}
#[derive(Default,Clone,Serialize,Deserialize)]
pub struct Cell{
    pub object_ids: Vec<usize>,
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
            animals_grid: vec![Cell::default();CELLS_HEIGHT*CELLS_WIDTH],
            plants_grid: vec![Cell::default();CELLS_HEIGHT*CELLS_WIDTH],
        }
    }
    pub fn update_animal_grid(&mut self,objects: &[Instance]){
        self.animals_grid.par_iter_mut().for_each(|cell| cell.clear());

        objects.iter().enumerate().for_each(|(id,instance)|{
            let i = (instance.position[0] * DIV ) as usize * CELLS_HEIGHT + (instance.position[1] * DIV ) as usize;
            self.animals_grid[i].add(id);
        });
    }
    pub fn update_plant_grid(&mut self,objects: &[Instance]){
        self.plants_grid.par_iter_mut().for_each(|cell| cell.clear());

        objects.iter().enumerate().for_each(|(id,instance)|{
            let i = (instance.position[0] * DIV ) as usize * CELLS_HEIGHT + (instance.position[1] * DIV ) as usize;
            self.plants_grid[i].add(id);
        });
    }
    pub fn handle_collisions(&mut self, animals: &mut Animals, plants: &mut Plants){
        for x in 0..CELLS_WIDTH{
            for y in 0..CELLS_HEIGHT{
                for z in 0..self.animals_grid[x * CELLS_HEIGHT + y].count(){
                    let animal_id = self.animals_grid[x * CELLS_HEIGHT + y].index(z);
                    let animal_body = animals.animals[animal_id].body;

                    for i in 0..3{
                        for j in 0..3{
                            let grid_index = (x+i).saturating_sub(1) * CELLS_HEIGHT + (y+j).saturating_sub(1);

                            for k in 0..self.animals_grid[grid_index].count(){
                                if k == z {continue}
                                let other_animal_id = self.animals_grid[grid_index].index(k);

                                let collision_axis_x = animals.animals[other_animal_id].body.position[0] - animal_body.position[0];
                                let collision_axis_y = animals.animals[other_animal_id].body.position[1] - animal_body.position[1];

                                let distance = (collision_axis_x*collision_axis_x + collision_axis_y*collision_axis_y).sqrt();

                                if distance < (animals.animals[other_animal_id].body.scale + animal_body.scale) * 0.5{
                                    animals.handle_animal_collision(animal_id,other_animal_id);
                                }
                            }
                        }
                    }

                    for i in 0..3{
                        for j in 0..3{
                            let grid_index = (x+i).saturating_sub(1) * CELLS_HEIGHT + (y+j).saturating_sub(1);

                            for k in 0..self.plants_grid[grid_index].count(){
                                let plant_id = self.plants_grid[grid_index].index(k);
                                let plant_body = plants.bodies[plant_id];

                                let relative_pos_x = plant_body.position[0] - animal_body.position[0];
                                let relative_pos_y = plant_body.position[1] - animal_body.position[1];

                                if (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y) < 0.05 * animal_body.scale && !plants.plants[plant_id].eaten{
                                    let energy = plants.handle_collision(plant_id);
                                    animals.handle_plant_collision(animal_id,energy);
                                }

                            }
                        }
                    }
                }
            }
        }
    }
}
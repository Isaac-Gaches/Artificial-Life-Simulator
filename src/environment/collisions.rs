use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animals;
use crate::environment::plants::Plants;
//use crate::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::environment::fruit::Fruits;
use crate::rendering::instance::Instance;
use crate::utilities::simulation_parameters::SimParams;

//pub const CELLS_HEIGHT: usize = (WORLD_HEIGHT/CELL_SIZE) as usize;
//pub const CELLS_WIDTH: usize = (WORLD_WIDTH/CELL_SIZE) as usize;
pub const CELL_SIZE: f32 = 0.4;
pub const DIV: f32 = 1.0/CELL_SIZE;

#[derive(Serialize,Deserialize,Clone)]
pub struct Collisions{
    pub animals_grid: Vec<Cell>,
    pub plants_grid: Vec<Cell>,
    pub fruit_grid: Vec<Cell>,
    pub cells_height: usize,
    pub cells_width: usize,
}
#[derive(Default,Clone,Serialize,Deserialize)]
pub struct Cell{
    pub object_ids: Vec<usize>,
}
impl Cell{
    pub(crate) fn index(&self, i: usize) -> usize{
        self.object_ids[i]
    }
    fn clear(&mut self){
        self.object_ids.clear();
    }
    fn add(&mut self, id: usize){
        self.object_ids.push(id);
    }
    pub fn count(&self)-> usize{
        self.object_ids.len()
    }
}
impl Collisions{
    pub fn new(sim_params: &SimParams)-> Self{
        let cells_height= (sim_params.world.height/CELL_SIZE) as usize;
        let cells_width= (sim_params.world.width/CELL_SIZE) as usize;
        Self{
            animals_grid: vec![Cell::default();cells_height*cells_width],
            plants_grid: vec![Cell::default();cells_height*cells_width],
            fruit_grid: vec![Cell::default();cells_height*cells_width],
            cells_width,
            cells_height,
        }
    }
    pub fn update_animal_grid(&mut self,objects: &[Instance]){
        self.animals_grid.par_iter_mut().for_each(|cell| cell.clear());

        objects.iter().enumerate().for_each(|(id,instance)|{
            let i = (instance.position[0] * DIV ) as usize * self.cells_height + (instance.position[1] * DIV ) as usize;
            self.animals_grid[i].add(id);
        });
    }
    pub fn update_plant_grid(&mut self,objects: &[Instance]){
        self.plants_grid.par_iter_mut().for_each(|cell| cell.clear());

        objects.iter().enumerate().for_each(|(id,instance)|{
            let i = (instance.position[0] * DIV ) as usize * self.cells_height + (instance.position[1] * DIV ) as usize;
            self.plants_grid[i].add(id);
        });
    }
    pub fn update_fruit_grid(&mut self,objects: &[Instance]){
        self.fruit_grid.par_iter_mut().for_each(|cell| cell.clear());

        objects.iter().enumerate().for_each(|(id,instance)|{
            let i = (instance.position[0] * DIV ) as usize * self.cells_height + (instance.position[1] * DIV ) as usize;
            self.fruit_grid[i].add(id);
        });
    }
    pub fn handle_collisions(&mut self, animals: &mut Animals, plants: &mut Plants,fruit: &mut Fruits,sim_params: &SimParams){
        for x in 0..self.cells_width{
            for y in 0..self.cells_height{
                for z in 0..self.animals_grid[x * self.cells_height + y].count(){
                    let animal_id = self.animals_grid[x * self.cells_height + y].index(z);
                    let animal_body = animals.animals[animal_id].body;

                    //plants
                    for i in 0..3{
                        for j in 0..3{
                            let grid_index = (x+i).saturating_sub(1) * self.cells_height + (y+j).saturating_sub(1);
                            //animal
                            for k in 0..self.animals_grid[grid_index].count(){
                                if k == z {continue}
                                let other_animal_id = self.animals_grid[grid_index].index(k);

                                let collision_axis_x = animals.animals[other_animal_id].body.position[0] - animal_body.position[0];
                                let collision_axis_y = animals.animals[other_animal_id].body.position[1] - animal_body.position[1];

                                let distance = (collision_axis_x*collision_axis_x + collision_axis_y*collision_axis_y).sqrt();

                                if distance < (animals.animals[other_animal_id].body.scale + animal_body.scale) * 0.5{
                                    animals.handle_animal_collision(animal_id,other_animal_id,sim_params.animals.carnivory_efficiency);
                                }
                            }
                            //plant
                            for k in 0..self.plants_grid[grid_index].count(){
                                let plant_id = self.plants_grid[grid_index].index(k);
                                if plants.plants[plant_id].eaten{
                                    continue;
                                }
                                let plant_body = plants.bodies[plant_id];

                                let relative_pos_x = plant_body.position[0] - animal_body.position[0];
                                let relative_pos_y = plant_body.position[1] - animal_body.position[1];

                                if (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y) < 0.05 * animal_body.scale && !plants.plants[plant_id].eaten{
                                    let resources = plants.handle_collision(plant_id,sim_params);
                                    animals.handle_plant_collision(animal_id,resources,sim_params.animals.herbivory_efficiency);
                                }

                            }
                            //fruit
                            for k in 0..self.fruit_grid[grid_index].count(){
                                let fruit_id = self.fruit_grid[grid_index].index(k);
                                if fruit.fruit[fruit_id].eaten{
                                    continue;
                                }
                                let fruit_body = fruit.bodies[fruit_id];

                                let relative_pos_x = fruit_body.position[0] - animal_body.position[0];
                                let relative_pos_y = fruit_body.position[1] - animal_body.position[1];

                                if (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y) < 0.05 * animal_body.scale && !fruit.fruit[fruit_id].eaten{
                                    let resources = fruit.handle_collision(fruit_id,sim_params);
                                    animals.handle_fruit_collision(animal_id,resources,sim_params.animals.herbivory_efficiency);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
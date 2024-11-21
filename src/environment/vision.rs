use std::f32::consts::{PI, TAU};
use std::ops::Index;
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animal;
use crate::environment::collisions::{CELL_SIZE, Collisions, DIV};
use crate::environment::rocks::RockMap;
use crate::rendering::instance::Instance;

#[derive(Clone,Serialize,Deserialize)]
pub struct SensoryInput{
    pub animal_vision: f32,
    pub plant_vision: f32,
    pub fruit_vision: f32,
    pub rock_vision: f32,
}

impl SensoryInput{
    pub(crate) fn stimulus(&self, plants: &[Instance], fruit: &[Instance], body: &Instance, animals: &[Animal], animal: &Animal, collisions: &Collisions, rock_map: &RockMap) -> Vec<f32>{
        let mut input = Vec::new();

        let mut closest = f32::MAX;
        let mut angle = 0.;

        let x = (body.position[0] * DIV) as usize;
        let y = (body.position[1] * DIV) as usize;

        if self.plant_vision > 0.1 {
            for i in 0..self.plant_vision as usize {
                for j in 0..self.plant_vision as usize {
                    let cell = collisions.plants_grid.index((x + i).saturating_sub(3).min(collisions.cells_width - 1) * collisions.cells_height + (y + j).saturating_sub(3).min(collisions.cells_height - 1));
                    for id in &cell.object_ids {
                        let plant = plants.index(*id);

                        let relative_pos_x = plant.position[0] - body.position[0];
                        let relative_pos_y = plant.position[1] - body.position[1];
                        let dist = relative_pos_x.abs() + relative_pos_y.abs();

                        if dist < closest {
                            closest = dist;
                            angle = relative_pos_y.atan2(relative_pos_x) - body.rotation;
                        }
                    }
                }
            }
            //always finds angle on rhs, this converts it into the acute angle if it's not already
            angle = if angle < -PI { angle + TAU } else if angle > PI { TAU - angle } else { angle };
            input.push(angle / PI);
            input.push(((self.plant_vision * CELL_SIZE) - closest).max(0.0) / (self.plant_vision * CELL_SIZE));
        }
        else{
            input.push(0.);
            input.push(0.);
        }

        //fruit
        if self.fruit_vision > 0.1 {
            for i in 0..self.fruit_vision as usize {
                for j in 0..self.fruit_vision as usize {
                    let cell = collisions.fruit_grid.index((x + i).saturating_sub(3).min(collisions.cells_width - 1) * collisions.cells_height + (y + j).saturating_sub(3).min(collisions.cells_height - 1));
                    for id in &cell.object_ids {
                        let fruit = fruit.index(*id);

                        let relative_pos_x = fruit.position[0] - body.position[0];
                        let relative_pos_y = fruit.position[1] - body.position[1];
                        let dist = relative_pos_x.abs() + relative_pos_y.abs();

                        if dist < closest {
                            closest = dist;
                            angle = relative_pos_y.atan2(relative_pos_x) - body.rotation;
                        }
                    }
                }
            }
            //always finds angle on rhs, this converts it into the acute angle if it's not already
            angle = if angle < -PI { angle + TAU } else if angle > PI { TAU - angle } else { angle };
            input.push(angle / PI);
            input.push(((self.fruit_vision * CELL_SIZE) - closest).max(0.0) / (self.fruit_vision * CELL_SIZE));
        }
        else{
            input.push(0.);
            input.push(0.);
        }

        if self.animal_vision > 0.1 {
            let mut carn: f32 = 0.;
            let mut same_species = 0.;

            closest = f32::MAX;
            angle = 0.;

            for i in 0..self.animal_vision as usize {
                for j in 0..self.animal_vision as usize {
                    let cell = collisions.animals_grid.index((x + i).saturating_sub(3).min(collisions.cells_width - 1) * collisions.cells_height + (y + j).saturating_sub(3).min(collisions.cells_height - 1));
                    for id in &cell.object_ids {
                        let other = animals.index(*id);

                        let relative_pos_x = other.body.position[0] - body.position[0];
                        let relative_pos_y = other.body.position[1] - body.position[1];
                        let dist = relative_pos_x.abs() + relative_pos_y.abs();

                        if dist < closest && dist > 0. {
                            closest = dist;
                            angle = relative_pos_y.atan2(relative_pos_x) - body.rotation;
                            carn = other.combat_stats.carnivore_factor;
                            same_species = if animal.species_id == other.species_id { -1. } else { 1.0 };
                        }
                    }
                }
            }

            angle = if angle < -PI { angle + TAU } else if angle > PI { TAU - angle } else { angle };
            input.push(angle / PI);
            input.push(((self.animal_vision * CELL_SIZE) - closest).max(0.0) / (self.animal_vision * CELL_SIZE));
            input.push(carn);
            //     input.push(same_species);
        }
        else{
            input.push(0.);
            input.push(0.);
            input.push(0.);
            // input.push(0.);
        }

        if self.rock_vision > 0.1 {
            let mut ray1 = animal.body.position;
            let mut dist1 = 0.;

            let mut ray2 = animal.body.position;
            let mut dist2 = 0.;

            for i in 0..(self.rock_vision * 5.) as usize {
                if rock_map.rocks[(ray1[0] * DIV) as usize * collisions.cells_height + (ray1[1] * DIV) as usize] > 0 {
                    dist1 = i as f32 * CELL_SIZE * 0.2;
                    break
                }

                ray1[0] += (animal.body.rotation - PI * 0.125).cos() * CELL_SIZE * 0.2;
                ray1[1] += (animal.body.rotation - PI * 0.125).sin() * CELL_SIZE * 0.2;
            }
            for i in 0..(self.rock_vision * 5.) as usize {
                if rock_map.rocks[(ray2[0] * DIV) as usize * collisions.cells_height + (ray2[1] * DIV) as usize] > 0 {
                    dist2 = i as f32 * CELL_SIZE * 0.2;
                    break
                }

                ray2[0] += (animal.body.rotation + PI * 0.125).cos() * CELL_SIZE * 0.2;
                ray2[1] += (animal.body.rotation + PI * 0.125).sin() * CELL_SIZE * 0.2;
            }

            input.push((dist1) / (self.rock_vision * CELL_SIZE));
            input.push((dist2) / (self.rock_vision * CELL_SIZE));
        }
        else {
            input.push(0.);
            input.push(0.);
        }

        input
    }
}
use std::f32::consts::{PI, TAU};
use std::ops::{Index, IndexMut};
use rand::Rng;
use rayon::prelude::*;
use crate::neural_network::Network;
use crate::plants::Plants;
use crate::render::Instance;

#[derive(Clone)]
struct Animal{
    energy: f32,
}
#[derive(Clone)]
struct SensoryInput{

}
impl SensoryInput{
    fn stimulus(&self,plants: &Vec<Instance>, body: &Instance) -> Vec<f32>{
        let mut closest = f32::MAX;
        let mut angle = 0.;
        plants.iter().for_each(|plant|{
            let relative_pos_x = plant.position[0] - body.position[0];
            let relative_pos_y = plant.position[1] - body.position[1];
            let dist = (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y).sqrt();
            if dist < closest{
                closest = dist;
                angle = relative_pos_y.atan2(relative_pos_x) - body.rotation;
            }
        });
        //always finds angle on rhs, this converts it into the acute angle if it's not already
        angle = if angle < -PI { angle + TAU } else if angle > PI { TAU - angle } else { angle };
        vec![closest,angle]
    }
}
pub struct Animals{
    animals: Vec<Animal>,
    bodies: Vec<Instance>,
    brains: Vec<Network>,
    senses: Vec<SensoryInput>,
}

impl Animals{
    pub fn genesis()->Self{
        let bodies = (0..500).map(|i| {
            Instance::new([rand::thread_rng().gen_range((-8.)..8.), rand::thread_rng().gen_range((-8.)..8.)], [1.0, 1.0, 1.0], rand::thread_rng().gen_range(-PI..PI),0.1)
        }).collect::<Vec<Instance>>();

        let brains = (0..500).map(|_| {
            Network::random(&[3,6,4])
        }).collect::<Vec<Network>>();

        let senses = (0..500).map(|_| {
            SensoryInput{ }
        }).collect::<Vec<SensoryInput>>();

        let animals = (0..500).map(|_| {
            Animal{energy: 100.}
        }).collect::<Vec<Animal>>();

        Self{
            animals,
            bodies,
            brains,
            senses,
        }
    }

    pub fn update(&mut self, plants: &mut Plants){
        let mut reproducing = Vec::new();
        self.brains.iter().zip(self.senses.iter()).zip(self.animals.iter()).enumerate().for_each(|(i,((network,senses),animal))|{
            let mut input = senses.stimulus(&plants.bodies,self.bodies.index(i));
            input.push(animal.energy/100.);
            let response = network.propagate(input);
            if *response.index(3) > 0. && animal.energy > 180.{
                reproducing.push(i);
            }
            let mut body = self.bodies.index_mut(i);
            body.position[0] += response.index(0).min(1.0) * 0.005 * body.rotation.cos();
            body.position[1] += response.index(0).min(1.0) * 0.005 * body.rotation.sin();
            body.rotation += (response.index(1) - response.index(2)).min(1.0) * 0.05;

        });

        reproducing.iter().for_each(|i|{
            self.asexual_offspring(*i);
            self.asexual_offspring(*i);
        });

        self.bodies.par_iter_mut().for_each(|body|{
            if body.position[0] > 8.0{
                body.position[0] = -8.0;
            }
            else if body.position[0] < -8.0 {
                body.position[0] = 8.0;
            }
            if body.position[1] > 8.0{
                body.position[1] = -8.0;
            }
            else if body.position[1] < -8.0 {
                body.position[1] = 8.0;
            }

            if body.rotation > PI{
                body.rotation = -PI;
            }
            else if body.rotation < -PI{
                body.rotation = PI;
            }
        });

        self.animals.par_iter_mut().for_each(|animal|{
            animal.energy -= 0.05;
        });

        (0..self.count()).rev().for_each(|i|{
            if self.animals.index(i).energy <= 0.{
                self.remove(i);
            }
        });

        self.bodies.iter().zip(self.animals.iter_mut()).for_each(|animal|{
            plants.bodies.iter().zip(plants.plants.iter_mut()).enumerate().for_each(|(i,plant)|{
                let relative_pos_x = plant.0.position[0] - animal.0.position[0];
                let relative_pos_y = plant.0.position[1] - animal.0.position[1];
                if (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y) < 0.003{
                    plant.1.eaten = true;
                    animal.1.energy += 40.;
                }
            });
        });
    }

    pub fn remove(&mut self, i: usize){
        self.animals.remove(i);
        self.bodies.remove(i);
        self.brains.remove(i);
        self.senses.remove(i);
    }

    pub fn instances(&self) -> &Vec<Instance>{
        &self.bodies
    }

    pub fn count(&self)->usize{
        self.bodies.len()
    }

    fn asexual_offspring(&mut self,i: usize){
        let mut animal = self.animals.index_mut(i);
        animal.energy -= 90.;

        let mut new_amimal = animal.clone();
        new_amimal.energy = 80.;
        self.animals.push(new_amimal);
        let mut new_brain = self.brains.index(i).clone();
        new_brain.mutate();
        self.brains.push(new_brain);
        let mut new_senses = self.senses.index(i).clone();
        self.senses.push(new_senses);
        let mut new_body = self.bodies.index(i).clone();
        self.bodies.push(new_body);
    }
}
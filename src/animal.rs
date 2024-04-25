use std::f32::consts::{PI, TAU};
use std::ops::{Index, IndexMut};
use rand::Rng;
use rayon::prelude::*;
use crate::eggs::Eggs;
use crate::neural_network::Network;
use crate::plants::Plants;
use crate::render::Instance;

#[derive(Clone)]
pub struct Animal{
    energy: f32,
    aggression: f32,
    carnivore_factor: f32,
}
#[derive(Clone)]
pub struct SensoryInput{

}
impl SensoryInput{
    fn stimulus(&self,plants: &Vec<Instance>, body: &Instance,animals: &Vec<Instance>) -> Vec<f32>{
        let mut input = Vec::new();

        let mut closest = f32::MAX;
        let mut angle = 0.;

        //plants
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
        input.push(angle);
        input.push((1.0-closest).max(0.));

        let mut bigger:f32 = -1.;

        //animals
        animals.iter().for_each(|animal|{
            let relative_pos_x = animal.position[0] - body.position[0];
            let relative_pos_y = animal.position[1] - body.position[1];
            let dist = (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y).sqrt();
            if dist < closest && dist > 0.{
                closest = dist;
                angle = relative_pos_y.atan2(relative_pos_x) - body.rotation;
                bigger = if animal.scale >= body.scale { -1. } else {1.0}
            }
        });
        angle = if angle < -PI { angle + TAU } else if angle > PI { TAU - angle } else { angle };
        input.push(angle);
        input.push((1.0-closest).max(0.));
        input.push(bigger);

        input
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
        let mut rng = rand::thread_rng();
        let bodies = (0..300).map(|i| {
            Instance::new([rng.gen_range((-8.)..8.), rng.gen_range((-8.)..8.)], [1.0, 1.0, 1.0], rng.gen_range(-PI..PI),rng.gen_range(0.05..0.2))
        }).collect::<Vec<Instance>>();

        let brains = (0..300).map(|_| {
            Network::random(&[6,12,5])
        }).collect::<Vec<Network>>();

        let senses = (0..300).map(|_| {
            SensoryInput{ }
        }).collect::<Vec<SensoryInput>>();

        let animals = (0..300).map(|_| {
            Animal{energy: 100., aggression: 0.0, carnivore_factor: rng.gen_range(0.0..1.0) }
        }).collect::<Vec<Animal>>();

        Self{
            animals,
            bodies,
            brains,
            senses,
        }
    }

    pub fn update(&mut self, plants: &mut Plants, eggs: &mut Eggs){
        let mut reproducing = Vec::new();
        self.brains.iter().zip(self.senses.iter()).zip(self.animals.iter_mut()).enumerate().for_each(|(i,((network,senses),animal))|{
            let mut input = senses.stimulus(&plants.bodies,self.bodies.index(i),&self.bodies);
            input.push(animal.energy/100.);
            let response = network.propagate(input);
            if *response.index(3) > 0. && animal.energy > 200.{
                reproducing.push(i);
            }
            let mut body = self.bodies.index_mut(i);
            body.position[0] += response.index(0).min(1.0) * 0.005 * body.rotation.cos();
            body.position[1] += response.index(0).min(1.0) * 0.005 * body.rotation.sin();
            body.rotation += (response.index(1) - response.index(2)).min(1.0) * 0.05;
            animal.energy -= body.scale * 0.4;
            animal.aggression = response.index(4).clamp(0.,1.);
        });

        reproducing.iter().for_each(|i|{
            let mut animal = self.animals.index_mut(*i);
            animal.energy -= 90.;

            let mut new_amimal = animal.clone();
            new_amimal.energy = 80.;
            let mut new_brain = self.brains.index(*i).clone();
            new_brain.mutate();
            let new_senses = self.senses.index(*i).clone();
            let mut new_body = self.bodies.index(*i).clone();
            new_body.scale = (new_body.scale + rand::thread_rng().gen_range(-0.015..0.015)).clamp(0.05,0.2);
            new_body.color = [new_amimal.carnivore_factor,1.-new_amimal.carnivore_factor,new_body.scale*5.];

            eggs.spawn(self.bodies.index(*i).position,new_body,new_senses,new_amimal,new_brain);
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

        (0..self.count()).rev().for_each(|i|{
            if self.animals.index(i).energy <= 0.{
               // meat.spawn(self.bodies.index(i).position);
                self.remove(i);
            }
        });

        self.bodies.iter().zip(self.animals.iter_mut()).for_each(|animal|{
            plants.bodies.iter().zip(plants.plants.iter_mut()).enumerate().for_each(|(i,plant)|{
                let relative_pos_x = plant.0.position[0] - animal.0.position[0];
                let relative_pos_y = plant.0.position[1] - animal.0.position[1];
                if (relative_pos_x * relative_pos_x + relative_pos_y * relative_pos_y) < 0.003 && !plant.1.eaten{
                    plant.1.eaten = true;
                    animal.1.energy += 50. * (1.0-animal.1.carnivore_factor);
                }
            });
        });

        for i in 0..self.count(){
            for j in 0..self.count(){
                if i == j { continue }
                let collision_axis_x = self.bodies.index(j).position[0] - self.bodies.index(i).position[0];
                let collision_axis_y = self.bodies.index(j).position[1] - self.bodies.index(i).position[1];

                let distance = (collision_axis_x*collision_axis_x + collision_axis_y*collision_axis_y).sqrt();

                if distance < 0.05{
                    let attack_i = self.animals.index(i).aggression * self.bodies.index(i).scale;
                    let attack_j = self.animals.index(j).aggression * self.bodies.index(j).scale;
                    if attack_i > attack_j && self.animals.index(i).energy > 0.{
                        self.animals.index_mut(i).energy += self.animals.index(j).energy * self.animals.index(i).carnivore_factor;
                        self.animals.index_mut(j).energy = 0.;
                    }
                    else if attack_j > attack_i && self.animals.index(j).energy > 0.{
                      //  self.bodies.index_mut(i).position[0] -= (collision_axis_x/distance)*0.02;
                       // self.bodies.index_mut(i).position[1] -= (collision_axis_y/distance)*0.02;
                        self.animals.index_mut(j).energy += self.animals.index(i).energy * self.animals.index(j).carnivore_factor;
                        self.animals.index_mut(i).energy = 0.;
                    }
                }
            }
        }
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

    pub fn spawn(&mut self,body: Instance,sense: SensoryInput,animal: Animal,brain:Network){
        self.animals.push(animal);
        self.senses.push(sense);
        self.brains.push(brain);
        self.bodies.push(body);
    }
}
use std::f32::consts::{PI, TAU};
use std::ops::{Index, IndexMut};
use rand::Rng;
use rayon::prelude::*;
use crate::eggs::Eggs;
use crate::neural_network::Network;
use crate::plants::Plants;
use crate::render::Instance;
use crate::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::simulation_parameters::SimParams;
use crate::species::SpeciesList;

#[derive(Clone)]
pub struct Animal{
    pub energy: f32,
    pub aggression: f32,
    pub carnivore_factor: f32,
    speed: f32,
    pub species_id: usize,
}
#[derive(Clone)]
pub struct SensoryInput{

}
impl SensoryInput{
    fn stimulus(&self, plants: &[Instance], body: &Instance, animals_bodies: &[Instance], animals: &[Animal],animal: &Animal) -> Vec<f32>{
        let mut input = Vec::new();

        let mut closest = f32::MAX;
        let mut angle = 0.;

        //plants
        plants.iter().for_each(|plant|{
            let relative_pos_x = plant.position[0] - body.position[0];
            let relative_pos_y = plant.position[1] - body.position[1];
            let dist = relative_pos_x.abs() + relative_pos_y.abs();
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
        let mut same_species = -1.;

        //animals
        animals_bodies.iter().zip(animals.iter()).for_each(|other|{
            let relative_pos_x = other.0.position[0] - body.position[0];
            let relative_pos_y = other.0.position[1] - body.position[1];
            let dist = relative_pos_x.abs() + relative_pos_y.abs();
            if dist < closest && dist > 0.{
                closest = dist;
                angle = relative_pos_y.atan2(relative_pos_x) - body.rotation;
                bigger = if other.0.scale >= body.scale { -1. } else {1.0};
                same_species = if animal.species_id == other.1.species_id { -1. } else {1.0};
            }
        });
        angle = if angle < -PI { angle + TAU } else if angle > PI { TAU - angle } else { angle };
        input.push(angle);
        input.push((1.0-closest).max(0.));
        input.push(bigger);
        input.push(same_species);

        input
    }
}
pub struct Animals{
    pub animals: Vec<Animal>,
    pub bodies: Vec<Instance>,
    pub brains: Vec<Network>,
    pub senses: Vec<SensoryInput>,
}

impl Animals{
    pub fn genesis()->Self{
        Self{
            animals: vec![],
            bodies: vec![],
            brains: vec![],
            senses: vec![],
        }
    }
    pub fn genesis_continued(&mut self){
        let mut rng = rand::thread_rng();

        self.senses.push(SensoryInput{ });
        self.brains.push(Network::random(&[7,14,5]));
        self.bodies.push(Instance::new([rng.gen_range((-WORLD_WIDTH)..WORLD_WIDTH), rng.gen_range((-WORLD_HEIGHT)..WORLD_HEIGHT)], [1.,1.,1.], rng.gen_range(-PI..PI),rng.gen_range(0.05..0.2)));
        self.animals.push(Animal{energy: 100., aggression: 0.0, carnivore_factor: rng.gen_range(0.0..=1.0), speed: rng.gen_range(1.0..3.0), species_id: 0 });
    }
    pub fn kill(&mut self){
        (0..self.count()).rev().for_each(|i|{
            if self.animals.index(i).energy <= 0.{
                self.remove(i);
            }
        });
    }

    pub fn update(&mut self, plants: &mut Plants, eggs: &mut Eggs,sim_params: &mut SimParams){
        let mut reproducing = Vec::new();
        self.brains.iter().zip(self.senses.iter()).enumerate().for_each(|(i,(network,senses))|{
            let mut input = senses.stimulus(&plants.bodies,self.bodies.index(i),&self.bodies,&self.animals,self.animals.index(i));
            let animal = self.animals.index_mut(i);
            input.push(animal.energy/100.);
            let response = network.propagate(input);
            if *response.index(3) > 0. && animal.energy > 200.{
                reproducing.push(i);
            }
            let body = self.bodies.index_mut(i);
            body.position[0] += response.index(0).min(1.0) * 0.002 * body.rotation.cos() * animal.speed;
            body.position[1] += response.index(0).min(1.0) * 0.002 * body.rotation.sin() * animal.speed;
            body.rotation += (response.index(1) - response.index(2)).clamp(-1.0,1.0) * 0.03 * animal.speed;
            animal.energy -= body.scale * 0.5 + response.index(0).min(1.0) * animal.speed * 0.02;
            animal.aggression = response.index(4).clamp(0.,1.);
        });

        reproducing.iter().for_each(|i|{
            let animal = self.animals.index_mut(*i);
            animal.energy -= 90.;
            let mut rng = rand::thread_rng();
            let mut new_animal = animal.clone();
            new_animal.speed = (new_animal.speed + rng.gen_range(-0.2..0.2)).clamp(1., 3.);
            new_animal.carnivore_factor = (new_animal.carnivore_factor + rng.gen_range(-0.1..0.1)).clamp(0., 1.);
            new_animal.energy = 80.;
            let mut new_brain = self.brains.index(*i).clone();
            new_brain.mutate();
            let new_senses = self.senses.index(*i).clone();
            let mut new_body = *self.bodies.index(*i);
            new_body.color = [1.,1.,1.];
            new_body.scale = (new_body.scale + rng.gen_range(-0.015..0.015)).clamp(0.05,0.2);
            eggs.spawn(self.bodies.index(*i).position, new_body, new_senses, new_animal, new_brain);
        });



        (&mut self.bodies,&self.animals).into_par_iter().for_each(|(body,animal)|{
            if body.position[0] > WORLD_WIDTH{
                body.position[0] = -WORLD_WIDTH;
            }
            else if body.position[0] < -WORLD_WIDTH {
                body.position[0] = WORLD_WIDTH;
            }
            if body.position[1] > WORLD_HEIGHT{
                body.position[1] = -WORLD_HEIGHT;
            }
            else if body.position[1] < -WORLD_HEIGHT {
                body.position[1] = WORLD_HEIGHT;
            }

            if body.rotation > PI{
                body.rotation = -PI;
            }
            else if body.rotation < -PI{
                body.rotation = PI;
            }

            if  sim_params.highlighted_species > 0 && animal.species_id == sim_params.highlighted_species as usize{
                body.color = [1.,1.,1.];
            }
            else if body.color == [1.,1.,1.]{
                body.color = [animal.carnivore_factor,1.- animal.carnivore_factor,(animal.speed -1.0)/3.];
            }

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

    pub fn spawn(&mut self,body: Instance,sense: SensoryInput,mut animal: Animal,brain:Network, species_list: &mut SpeciesList){
        animal.species_id = species_list.speciate(&animal,&brain);
        self.animals.push(animal);
        self.senses.push(sense);
        self.brains.push(brain);
        self.bodies.push(body);
    }
}
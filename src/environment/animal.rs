use std::f32::consts::PI;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use crate::environment::eggs::Eggs;
use crate::environment::neural_network::Network;
use crate::environment::plants::Plants;
use crate::environment::collisions::{CELL_SIZE, Collisions, DIV};
use crate::environment::fruit::Fruits;
use crate::environment::rocks::RockMap;
use crate::utilities::simulation_parameters::SimParams;
use crate::environment::species::SpeciesList;
use crate::environment::temperature::TemperatureMap;
use crate::environment::vision::SensoryInput;
use crate::rendering::instance::Instance;

#[derive(Clone,Serialize,Deserialize)]
pub struct Animal{
    pub id: usize,
    pub generation: usize,
    pub species_id: usize,
    pub maturity: f32,
    pub lean_mass: f32,
    pub hue: f32,
    pub resources: Resources,
    pub body: Instance,
    pub brain: Brain,
    pub senses: SensoryInput,
    max_stats: MaxStats,
    pub reproduction_stats: ReproductionStats,
    pub combat_stats: CombatStats,
    pub age: f32,
    pub temperature: f32,
    pub temp_tolerance: f32,
    pub ideal_temp: f32
}

impl Animal{
    fn offspring(&self, sim_params: &SimParams,species_list: &mut SpeciesList) ->Self{
        let mut new_animal = self.clone();

        let mut rng = rand::thread_rng();
        let mutation_strength = sim_params.animals.physical_mutation_strength/100.;
        let mutation_rate = (sim_params.animals.physical_mutation_rate/100.) as f64;

        new_animal.maturity = 0.;
        new_animal.generation = self.generation+1;
        new_animal.age = 0.;
        new_animal.resources.protein = new_animal.lean_mass*0.2*(new_animal.reproduction_stats.offspring_investment);
        new_animal.resources.energy = (new_animal.reproduction_stats.offspring_investment+0.5)*new_animal.lean_mass*5.0;

        new_animal.brain.network.mutate(sim_params.animals.brain_mutation_strength/100.,sim_params.animals.brain_mutation_rate/100.);

        if rng.gen_bool(mutation_rate){ new_animal.max_stats.speed = (new_animal.max_stats.speed + 3.0 * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(1., 4.);}
        if rng.gen_bool(mutation_rate) { new_animal.max_stats.attack = (new_animal.max_stats.attack + 10. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0., 10.); }
        if rng.gen_bool(mutation_rate) { new_animal.max_stats.size = (new_animal.max_stats.size + 0.4 * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0.16, 0.5); }
        if rng.gen_bool(mutation_rate) { new_animal.reproduction_stats.offspring_investment = (new_animal.reproduction_stats.offspring_investment + 10. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0., 10.); }
        if rng.gen_bool(mutation_rate) { new_animal.combat_stats.carnivore_factor = (new_animal.combat_stats.carnivore_factor + 1.0 * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0., 1.0); }
        if rng.gen_bool(mutation_rate) { new_animal.senses.animal_vision = (new_animal.senses.animal_vision + 12. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0.0, 12.); }
        if rng.gen_bool(mutation_rate) { new_animal.senses.plant_vision = (new_animal.senses.plant_vision + 12. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0.0, 12.); }
        if rng.gen_bool(mutation_rate) { new_animal.senses.rock_vision = (new_animal.senses.rock_vision + 12. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0.0, 12.); }
        if rng.gen_bool(mutation_rate) { new_animal.senses.fruit_vision = (new_animal.senses.fruit_vision + 12. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0.0, 12.); }
        if rng.gen_bool(mutation_rate) { new_animal.temp_tolerance = (new_animal.temp_tolerance + 14. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(1.0, 15.); }
        if rng.gen_bool(mutation_rate) { new_animal.ideal_temp = (new_animal.ideal_temp + 35. * rng.gen_range(-mutation_strength..=mutation_strength)).clamp(0.0, 35.); }

        if rng.gen_bool(mutation_rate) {
            new_animal.hue = (new_animal.hue + rng.gen_range(-mutation_strength..=mutation_strength)).rem_euclid(1.);
            new_animal.body.set_hsl(new_animal.hue, 1.0);
        }

        new_animal.combat_stats.speed = new_animal.max_stats.speed * 0.5;
        new_animal.combat_stats.attack = new_animal.max_stats.attack * 0.5;
        new_animal.body.scale = new_animal.max_stats.size * 0.5;

        new_animal.lean_mass = new_animal.combat_stats.attack * 5.0 + new_animal.combat_stats.speed * 8.0 + new_animal.body.scale * 30.;
        new_animal.species_id = species_list.speciate(&new_animal,self.species_id,sim_params);

        new_animal
    }
    fn internal_inputs(&self, mut input: Vec<f32>) -> Vec<f32>{
        input.push(self.resources.energy/self.resources.max_energy);
        input.push(self.combat_stats.aggression);
      //  input.push(self.reproduction_stats.birth_desire);

        input
    }
}
#[derive(Clone,Serialize,Deserialize)]
pub struct CombatStats{
    pub carnivore_factor: f32,
    pub aggression: f32,
    pub attack: f32,
    pub speed: f32,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Brain{
    pub network: Network,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct ReproductionStats{
    pub offspring_investment: f32,
    pub birth_timer: f32,
    //pub birth_desire: f32,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct MaxStats{
    speed: f32,
    size: f32,
    attack: f32,
}
#[derive(Clone,Serialize,Deserialize)]
pub struct Resources{
    pub energy: f32,
    pub protein: f32,
    max_energy: f32,
    max_protein: f32,
}

impl Resources{
    fn add(&mut self,resources:(f32,f32)){
        self.energy+=resources.0;
        self.protein+=resources.1;

        self.energy = self.max_energy.min(self.energy);
        self.protein = self.max_protein.min(self.protein);
    }
}

#[derive(Clone,Serialize,Deserialize)]
pub struct Animals{
    pub animals: Vec<Animal>,
    next_free_id: usize,
}

impl Animals{
    pub fn genesis()->Self{
       Self{
           animals: vec![],
           next_free_id: 0,
       }
    }
    pub fn spawn(&mut self,sim_params: &SimParams){
        let mut rng = rand::thread_rng();

        let senses = SensoryInput{
            animal_vision: rng.gen_range(0.0..12.0),
            plant_vision: rng.gen_range(0.0..12.0),
            fruit_vision: rng.gen_range(0.0..12.0),
            rock_vision: rng.gen_range(0.0..12.0),
        };

        let mut brain = Brain{ network: Network::zero(&[11,11,4])};
        brain.network.mutate(0.5,0.4);

        let max_stats = MaxStats{ speed: rng.gen_range(1.0..4.0), size: rng.gen_range(0.16..0.5), attack: rng.gen_range(0.0..10.)};
        let mut body = Instance::new([rng.gen_range(CELL_SIZE*2.0..sim_params.world.width -CELL_SIZE*2.0), rng.gen_range(CELL_SIZE*2.0..sim_params.world.height -CELL_SIZE*2.0)],[0.0,0.0,0.0], rng.gen_range(-PI..PI),max_stats.size * 0.5);
        let hue = rng.gen_range(0.0..=1.0);
        body.set_hsl(hue,1.0);
        let resources = Resources{ energy: 2000.0, protein: 0.0, max_energy: body.scale * 20000., max_protein: body.scale * 400. };
        let reproduction_stats = ReproductionStats{ offspring_investment: rng.gen_range(0.0..10.0), birth_timer: 0.0};
        let combat_stats = CombatStats{ carnivore_factor: rng.gen_range(0.0..=1.0), aggression: 0.0, attack: max_stats.attack * 0.5, speed: max_stats.speed * 0.5, };
        let ideal_temp = rng.gen_range(0.0..=35.0);
        let temp_tolerance = rng.gen_range(1.0..=15.0);

        let animal = Animal{
            id: self.next_free_id,
            generation: 0,
            species_id: 0,
            maturity: 0.0,
            lean_mass: body.scale * 30. + combat_stats.speed * 8.0 + combat_stats.attack * 5.0,
            hue,
            resources,
            body,
            brain,
            senses,
            max_stats,
            reproduction_stats,
            combat_stats,
            age: 0.0,
            temperature: ideal_temp,
            temp_tolerance,
            ideal_temp,
        };

        self.next_free_id += 1;

        self.animals.push(animal);
    }
    pub fn kill(&mut self){
        (0..self.count()).rev().for_each(|i|{
            if self.animals.index(i).resources.energy <= 0.{
                self.remove(i);
            }
        });
    }

    pub fn update(&mut self, plants: &mut Plants,fruit: &mut Fruits, eggs: &mut Eggs,sim_params: &mut SimParams,collisions: &Collisions, species_list: &mut SpeciesList,rock_map: &RockMap, temp_map: &TemperatureMap){
        for i in 0..self.count(){
            let input = self.animals.index(i).senses.stimulus(&plants.bodies,&fruit.bodies,&self.animals.index(i).body,&self.animals,self.animals.index(i),collisions,rock_map);

            let animal = self.animals.index_mut(i);

            let input = animal.internal_inputs(input);

            animal.brain.network.input(input);

            if animal.maturity == 10. && animal.reproduction_stats.birth_timer <= 0. && animal.resources.energy > ((animal.reproduction_stats.offspring_investment/10.)+0.5)*animal.lean_mass*5.0 * sim_params.animals.reproduction_energy_cost + animal.resources.max_energy * 0.3 && animal.resources.protein > animal.lean_mass*0.2*((animal.reproduction_stats.offspring_investment/10.)+0.5) * sim_params.animals.reproduction_protein_cost{
                animal.reproduction_stats.birth_timer = (10. + animal.reproduction_stats.offspring_investment * 4.) * sim_params.animals.reproduction_time;
                animal.resources.energy -= ((animal.reproduction_stats.offspring_investment/10.)+0.5)*animal.lean_mass*5.0 * sim_params.animals.reproduction_energy_cost;
                animal.resources.protein -= animal.lean_mass*0.4*((animal.reproduction_stats.offspring_investment/10.)+0.5) * sim_params.animals.reproduction_protein_cost;

                let mut offspring = animal.offspring(sim_params,species_list);
                offspring.id = self.next_free_id;

                self.next_free_id += 1;

                eggs.spawn(animal.body.position,offspring);
            }
        }

        let arc_rocks = Arc::new(rock_map);
        let arc_temperature = Arc::new(temp_map);

        self.animals.par_iter_mut().for_each(|animal|{
            let response = animal.brain.network.propagate();

            /*let start = animal.body.position[0];
            animal.body.position[0] += response.index(0).max(0.0) * 0.01 * animal.body.rotation.cos() * animal.combat_stats.speed;

            'outer: for x in -1..=1{
                for y in -1..=1{
                    if x == 0 && y == 0 { continue;}
                    let i = (animal.body.position[0] * DIV + x as f32) as usize * CELLS_HEIGHT + (animal.body.position[1] * DIV + y as f32) as usize;
                    if arc.rocks[i] > 0{
                        animal.body.position[0] = start;
                        break 'outer;
                    }
                }
            }

            let start = animal.body.position[1];
            animal.body.position[1] += response.index(0).max(0.0) * 0.01 * animal.body.rotation.sin() * animal.combat_stats.speed;

            'outer: for x in -1..=1{
                for y in -1..=1{
                    if x == 0 && y == 0 { continue;}
                    let i = (animal.body.position[0] * DIV + x as f32) as usize * CELLS_HEIGHT + (animal.body.position[1] * DIV + y as f32) as usize;
                    if arc.rocks[i] > 0{
                        animal.body.position[1] = start;
                        break 'outer;
                    }
                }
            }*/

            let start = animal.body.position[0];
            animal.body.position[0] += response.index(0).min(1.0) * 0.008 * animal.body.rotation.cos() * animal.combat_stats.speed * sim_params.animals.movement_speed;

            let i = (animal.body.position[0] * DIV) as usize * collisions.cells_height + (animal.body.position[1] * DIV) as usize;
            if arc_rocks.rocks[i] > 0{
                animal.body.position[0] = start;
            }

            let start = animal.body.position[1];
            animal.body.position[1] += response.index(0).min(1.0) * 0.008 * animal.body.rotation.sin() * animal.combat_stats.speed * sim_params.animals.movement_speed;

            let i = (animal.body.position[0] * DIV) as usize * collisions.cells_height + (animal.body.position[1] * DIV) as usize;
            if arc_rocks.rocks[i] > 0{
                animal.body.position[1] = start;
            }

            let temperature = arc_temperature.cells[i];
            let temp_diff = temperature - animal.temperature;
            animal.temperature += (temp_diff/sim_params.animals.temperature_sensitivity)/60.;

            if (animal.temperature - animal.ideal_temp).abs() > animal.temp_tolerance{
                animal.resources.energy -= 1.0;
            }

            animal.body.rotation += response.index(1).min(1.0) * 0.04 * animal.combat_stats.speed * sim_params.animals.turning_speed;
            animal.body.rotation -= response.index(2).min(1.0) * 0.04 * animal.combat_stats.speed * sim_params.animals.turning_speed;
            animal.combat_stats.aggression = response.index(3).min(1.0);
           // animal.reproduction_stats.birth_desire = response.index(4).min(1.0);

            animal.resources.energy -= //1 energy per sec at min ish
                animal.body.scale * 0.042 * sim_params.animals.size_energy_cost + // 0.08 -> 0.5
                response.index(0) * animal.combat_stats.speed * 0.0067 * sim_params.animals.speed_energy_cost + // 0.5 -> 4
                (response.index(1)+response.index(2)) * animal.combat_stats.speed * 0.0067 * sim_params.animals.turning_energy_cost + // 0.5 -> 4
                0.001 * animal.combat_stats.aggression * animal.combat_stats.attack * sim_params.animals.attack_energy_cost + // 0 -> 10
                (animal.senses.animal_vision + animal.senses.rock_vision + animal.senses.plant_vision + animal.senses.fruit_vision) * 0.0001 * sim_params.animals.vision_energy_cost; // 0 -> 48

            if animal.reproduction_stats.birth_timer > 0. { animal.reproduction_stats.birth_timer -= 1./60.; }

            if animal.maturity < 10. && animal.resources.protein > animal.lean_mass*0.3  {
                animal.maturity += 1.;
                animal.resources.protein -= animal.lean_mass*0.3;
                animal.combat_stats.attack = animal.max_stats.attack * (0.5 + animal.maturity * 0.05);
                animal.combat_stats.speed = animal.max_stats.speed * (0.5 + animal.maturity * 0.05);
                animal.body.scale = animal.max_stats.size * (0.5 + animal.maturity * 0.05);
                animal.lean_mass = sim_params.animals.attack_protein_cost * animal.combat_stats.attack * 2.0 + sim_params.animals.speed_protein_cost * animal.combat_stats.speed * 4.0 + sim_params.animals.size_protein_cost * animal.body.scale * 32.;
                animal.resources.max_protein = animal.body.scale * 400.;
                animal.resources.max_energy = animal.body.scale *10000.;
            }

            if animal.body.position[0] > sim_params.world.width{
                animal.body.position[0] = 0.0;
            }
            else if animal.body.position[0] < 0.0 {
                animal.body.position[0] = sim_params.world.width;
            }
            if animal.body.position[1] > sim_params.world.height{
                animal.body.position[1] = 0.0;
            }
            else if animal.body.position[1] < 0.0 {
                animal.body.position[1] = sim_params.world.height;
            }

            if animal.body.rotation > PI{
                animal.body.rotation = -PI;
            }
            else if animal.body.rotation < -PI{
                animal.body.rotation = PI;
            }

            animal.age+=1./60.;
            if animal.age > animal.max_stats.size * 5760. * sim_params.animals.lifespan{
                animal.resources.energy = 0.;
            }
        });
    }

    pub fn handle_animal_collision(&mut self, animal_id: usize, other_animal_id: usize,efficiency: f32){
        if self.animals.index(animal_id).resources.energy > 0. && self.animals.index(other_animal_id).resources.energy > 0. {
            let damage_i = self.animals.index(animal_id).combat_stats.aggression * self.animals.index(animal_id).combat_stats.attack * self.animals.index(animal_id).body.scale;
            let damage_j = self.animals.index(other_animal_id).combat_stats.aggression * self.animals.index(other_animal_id).combat_stats.attack * self.animals.index(animal_id).body.scale;

            if damage_i > damage_j {
                self.animal_collision(animal_id,other_animal_id,efficiency);
            } else if damage_j > damage_i{
                self.animal_collision(other_animal_id,animal_id,efficiency);
            }
        }
    }

    fn animal_collision(&mut self,animal_id: usize,other_animal_id: usize,efficiency: f32,){
        let efficiency = (1.0/(-3.* self.animals[animal_id].combat_stats.carnivore_factor -1.2) + 1.235) * efficiency;

        let (energy,protein) =
            ((self.animals.index(other_animal_id).resources.energy + self.animals.index(other_animal_id).lean_mass * 5.) * efficiency,
            (self.animals.index(other_animal_id).resources.protein + self.animals.index(other_animal_id).lean_mass ) * efficiency);

        self.animals.index_mut(other_animal_id).resources.energy = 0.;

        let animal = self.animals.index_mut(animal_id);

        animal.resources.add((energy,protein));
    }

    pub fn handle_plant_collision(&mut self, animal_id: usize, resources: (f32,f32),efficiency: f32){
        let efficiency = (1.0 - 0.7 * self.animals[animal_id].combat_stats.carnivore_factor) * efficiency;

        self.animals[animal_id].resources.add((resources.0 * efficiency,resources.1 * efficiency));
    }

    pub fn handle_fruit_collision(&mut self, animal_id: usize, resources: (f32,f32)){
        let efficiency = 1.0/(2.* self.animals[animal_id].combat_stats.carnivore_factor -3.5) + 1.285;
        self.animals[animal_id].resources.add((resources.0 * efficiency,resources.1 * efficiency));
    }

    pub fn remove(&mut self, i: usize){
        self.animals.remove(i);
    }

    pub fn instances(&self) -> Vec<Instance>{
        self.animals.par_iter().map(|animal: &Animal|{
            animal.body
        }).collect()
    }

    pub fn count(&self)->usize{
        self.animals.len()
    }

    pub fn birth(&mut self, animal: Animal){
        self.animals.push(animal);
    }
}
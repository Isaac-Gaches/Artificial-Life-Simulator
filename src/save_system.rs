use std::fs;
use std::fs::File;
use serde::{Deserialize, Serialize};
use crate::animal::Animals;
use crate::collisions::Collisions;
use crate::eggs::Eggs;
use crate::plants::Plants;
use crate::simulation_parameters::SimParams;
use crate::species::SpeciesList;
use crate::statistics::Stats;

pub struct SaveSystem;

impl SaveSystem{
    pub fn load() -> SimulationSave{
        let data = fs::read_to_string("save").expect("Unable to read file");

        serde_json::from_str(&data).unwrap()
    }

    pub fn save(step: i32, animals: Animals,plants: Plants,eggs: Eggs, collisions: Collisions,species_list: SpeciesList,stats: Stats,sim_params: SimParams){
        let save = SimulationSave{
            step,
            animals,
            plants,
            eggs,
            collisions,
            species_list,
            stats,
            sim_params,
        };

        let serialized = serde_json::to_string(&save).unwrap();

        File::create("save").unwrap();
        fs::write("save", &serialized).expect("Unable to write file");
    }
}
#[derive(Serialize, Deserialize)]
pub struct SimulationSave{
    step: i32,
    animals: Animals,
    plants: Plants,
    eggs: Eggs,
    collisions: Collisions,
    species_list: SpeciesList,
    stats: Stats,
    sim_params: SimParams,
}

impl SimulationSave{
    pub fn open(self) -> (i32,Animals,Plants,Eggs,Collisions,SpeciesList,Stats,SimParams){
        (self.step,self.animals,self.plants,self.eggs,self.collisions,self.species_list,self.stats,self.sim_params)
    }
}
use std::fs;
use std::fs::File;
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animals;
use crate::environment::collisions::Collisions;
use crate::environment::eggs::Eggs;
use crate::environment::plants::Plants;
use crate::environment::rocks::RockMap;
use crate::utilities::simulation_parameters::SimParams;
use crate::environment::species::SpeciesList;
use crate::utilities::statistics::Stats;

pub struct SaveSystem;

impl SaveSystem{
    pub fn load() -> SimulationSave{
        let data = fs::read_to_string("saves/save").expect("Unable to read file");

        serde_json::from_str(&data).unwrap()
    }

    pub fn save(step: i32, animals: Animals,plants: Plants,eggs: Eggs, collisions: Collisions,species_list: SpeciesList,stats: Stats,sim_params: SimParams,rock_map: RockMap){
        let save = SimulationSave{
            step,
            animals,
            plants,
            eggs,
            collisions,
            species_list,
            stats,
            sim_params,
            rock_map,
        };

        let serialized = serde_json::to_string(&save).unwrap();

        File::create("saves/save").unwrap();
        fs::write("saves/save", serialized).expect("Unable to write file");
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
    rock_map: RockMap,
}

impl SimulationSave{
    pub fn open(self) -> (i32,Animals,Plants,Eggs,Collisions,SpeciesList,Stats,SimParams,RockMap){
        (self.step,self.animals,self.plants,self.eggs,self.collisions,self.species_list,self.stats,self.sim_params,self.rock_map)
    }
}
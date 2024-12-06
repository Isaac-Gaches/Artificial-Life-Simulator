use std::fs;
use std::fs::File;
use std::ops::Index;
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animals;
use crate::environment::collisions::Collisions;
use crate::environment::eggs::Eggs;
use crate::environment::fruit::Fruits;
use crate::environment::plants::Plants;
use crate::environment::rocks::RockMap;
use crate::utilities::simulation_parameters::SimParams;
use crate::environment::species::SpeciesList;
use crate::utilities::statistics::Stats;

#[derive(Default)]
pub struct SaveSystem{
    saves: Vec<String>,
    save_number: usize,
}

impl SaveSystem{
    pub fn load(&self,save_id: usize) -> SimulationSave{
        let data = fs::read_to_string(self.saves.index(save_id)).expect("Unable to read file");

        serde_json::from_str(&data).unwrap()
    }

    pub fn save(&mut self,step: i32, animals: Animals,plants: Plants, fruits: Fruits,eggs: Eggs,species_list: SpeciesList,stats: Stats,sim_params: SimParams,rock_map: RockMap){
        let save = SimulationSave{
            step,
            animals,
            plants,
            fruits,
            eggs,
            species_list,
            stats,
            sim_params,
            rock_map,
        };

        let serialized = serde_json::to_string(&save).unwrap();

        let path = ["saves/save_", &self.save_number.to_string()].join("");

        File::create(&path).unwrap();
        fs::write(&path, serialized).expect("Unable to write file");

        self.save_number+=1;
        self.saves.push(path);
    }
}
#[derive(Serialize, Deserialize)]
pub struct SimulationSave{
    step: i32,
    animals: Animals,
    plants: Plants,
    fruits: Fruits,
    eggs: Eggs,
    species_list: SpeciesList,
    stats: Stats,
    sim_params: SimParams,
    rock_map: RockMap,
}

impl SimulationSave{
    pub fn open(self) -> (i32,Animals,Plants,Fruits,Eggs,SpeciesList,Stats,SimParams,RockMap){
        (self.step,self.animals,self.plants,self.fruits,self.eggs,self.species_list,self.stats,self.sim_params,self.rock_map)
    }
}
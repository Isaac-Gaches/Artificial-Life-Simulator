use std::fs;
use std::fs::File;
use std::ops::Index;
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animals;
use crate::environment::eggs::Eggs;
use crate::environment::fruit::{Fruits, FruitSpawners};
use crate::environment::plants::{Plants, PlantSpawners};
use crate::environment::rocks::RockMap;
use crate::utilities::simulation_parameters::SimParams;
use crate::environment::species::SpeciesList;
use crate::utilities::statistics::Stats;

pub struct SaveSystem{
    pub saves: Vec<String>,
    save_number: usize,
}

impl Default for SaveSystem{
    fn default() -> Self {
        let saves: Vec<String> = fs::read_dir("saves").unwrap().map(|file|{
            file.unwrap().file_name().into_string().unwrap()
        }).collect();
        Self{
            save_number: saves.len(),
            saves,
        }
    }
}

impl SaveSystem{
    pub fn load(&self,save_id: usize) -> SimulationSave{
        let data = fs::read_to_string(["saves/",self.saves.index(save_id)].join("")).expect("Unable to read file");

        serde_json::from_str(&data).unwrap()
    }

    pub fn save(&mut self,step: i32, animals: Animals,plants: Plants, fruits: Fruits,eggs: Eggs,species_list: SpeciesList,stats: Stats,sim_params: SimParams,rock_map: RockMap, fruit_spawners: FruitSpawners,plant_spawners: PlantSpawners){
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
            fruit_spawners,
            plant_spawners,
        };

        let serialized = serde_json::to_string(&save).unwrap();

        let mut path = ["save_",&self.save_number.to_string()].join("");
        loop {
            if self.saves.contains(&path) {
                self.save_number += 1;
                path = ["save_",&self.save_number.to_string()].join("")
            }
            else {
                break
            }
        }

        File::create(["saves/",&path].join("")).unwrap();
        fs::write(["saves/",&path].join(""), serialized).expect("Unable to write file");

        self.save_number+=1;
        self.saves.push(path);
    }

    pub fn delete(&mut self, i:usize){
        fs::remove_file(["saves/",self.saves.index(i)].join("")).unwrap();
        self.saves.remove(i);
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
    fruit_spawners: FruitSpawners,
    plant_spawners: PlantSpawners,
}

impl SimulationSave{
    pub fn open(self) -> (i32,Animals,Plants,Fruits,Eggs,SpeciesList,Stats,SimParams,RockMap,FruitSpawners,PlantSpawners){
        (self.step,self.animals,self.plants,self.fruits,self.eggs,self.species_list,self.stats,self.sim_params,self.rock_map,self.fruit_spawners,self.plant_spawners)
    }
}
use serde::{Deserialize, Serialize};
use crate::environment::animal::Animal;
use crate::utilities::simulation_parameters::SimParams;

#[derive(Serialize,Deserialize,Clone)]
struct Species{
    parent: usize,
    specimen: Animal,
    count: usize,
}
impl Species{
    fn compare(&self, animal: &Animal) -> f32{
        self.specimen.brain.network.compare(&animal.brain.network)
    }
}
#[derive(Default,Serialize,Deserialize,Clone)]
pub struct SpeciesList{
    species: Vec<Species>,
}
impl SpeciesList{
    pub fn speciate(&mut self, child: &Animal,parent_species_id: usize,sim_params: &SimParams) -> usize{
        if parent_species_id == 0 || self.species[parent_species_id -1].specimen.brain.network.compare(&child.brain.network) > sim_params.animals.speciation_threshold{
            let new_species = Species{
                parent: parent_species_id,
                specimen: child.clone(),
                count: 1,
            };
            self.species.push(new_species);
            self.species.len()
        }
        else{
            self.species[parent_species_id -1].count += 1;
            parent_species_id
        }
    }
    pub fn count(&self)->usize{
        self.species.len()
    }
}
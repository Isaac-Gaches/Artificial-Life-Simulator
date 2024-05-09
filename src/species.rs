use crate::animal::Animal;
use crate::neural_network::Network;

struct Species{
    specimen_brain: Network,
    specimen_animal: Animal,
    count: usize,
}
impl Species{
    fn compare(&self, animal: &Animal,network: &Network) -> f32{
        self.specimen_brain.compare(network)
    }
}
#[derive(Default)]
pub struct SpeciesList{
    species: Vec<Species>,
}
impl SpeciesList{
    pub fn speciate(&mut self, animal: &Animal,network: &Network) -> usize{
        match self.species.iter().enumerate().find(|(i,species)|{ species.compare(animal, network) < 30. }){
            Some((i,_)) => {
                i
            }
            None => {
                let new_species = Species{
                    specimen_brain: network.clone(),
                    specimen_animal: animal.clone(),
                    count: 1,
                };
                self.species.push(new_species);
                self.species.len()
            }
        }
    }
    pub fn count(&self)->usize{
        self.species.len()
    }
}
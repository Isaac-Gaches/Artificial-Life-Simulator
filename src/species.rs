use crate::animal::Animal;

struct Species{
    parent: usize,
    specimen: Animal,
    count: usize,
}
impl Species{
    fn compare(&self, animal: &Animal) -> f32{
        self.specimen.brain.compare(&animal.brain)
    }
}
#[derive(Default)]
pub struct SpeciesList{
    species: Vec<Species>,
}
impl SpeciesList{
    pub fn speciate(&mut self, animal: &Animal,parent_species: usize) -> usize{
        match self.species.iter().enumerate().find(|(i,species)|{ species.compare(animal) < 30. }){
            Some((i,_)) => {
                i
            }
            None => {
                let new_species = Species{
                    parent: parent_species,
                    specimen: animal.clone(),
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
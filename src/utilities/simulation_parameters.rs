use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone)]
pub struct SimParams{
    pub steps_per_frame: i32,
    pub plant_spawn_rate: i32,
    pub highlighted_species: i32,
    pub brain_mutation_rate: f32,
    pub physical_mutation_rate: f32,
}
impl Default for SimParams{
    fn default() -> Self {
        Self{
            steps_per_frame: 1,
            plant_spawn_rate: 10,
            highlighted_species: -1,
            brain_mutation_rate: 15.0,
            physical_mutation_rate: 10.0,
        }
    }
}
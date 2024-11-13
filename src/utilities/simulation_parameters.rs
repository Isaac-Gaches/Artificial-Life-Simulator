use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone)]
pub struct SimParams{
    pub plants: PlantSettings,
    pub fruit: FruitSettings,
    pub animals: AnimalSettings,
    pub steps_per_frame: i32,
    pub highlighted_species: i32,
    pub pen_size: i32,
    pub build_mode: bool
}
#[derive(Serialize,Deserialize,Clone)]
pub struct PlantSettings {
    pub spawn_rate: i32,
    pub energy: f32,
    pub protein: f32,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct FruitSettings {
    pub spawn_rate: i32,
    pub energy: f32,
    pub protein: f32,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct AnimalSettings{
    pub brain_mutation_rate: f32,
    pub brain_mutation_strength: f32,
    pub physical_mutation_rate: f32,
    pub physical_mutation_strength: f32,
}
impl Default for SimParams{
    fn default() -> Self {
        Self{
            steps_per_frame: 1,
            plants: PlantSettings {
                spawn_rate: 10,
                energy: 100.0,
                protein: 0.1,
            },
            fruit: FruitSettings {
                spawn_rate: 2,
                energy: 250.0,
                protein: 0.15,
            },
            animals: AnimalSettings{
                brain_mutation_rate: 5.0,
                brain_mutation_strength: 20.,
                physical_mutation_rate: 20.0,
                physical_mutation_strength: 20.0,
            },
            highlighted_species: -1,
            pen_size: 0,
            build_mode: false,
        }
    }
}
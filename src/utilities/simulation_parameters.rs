use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone)]
pub struct SimParams{
    pub plants: PlantSettings,
    pub fruit: FruitSettings,
    pub animals: AnimalSettings,
    pub build: BuildSettings,
    pub simulation: SimulationSettings,
    pub world: WorldSettings,
    pub highlighted_species: i32,
    pub save_id: usize,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct SimulationSettings {
    pub steps_per_frame: i8,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct BuildSettings {
    pub pen_size: i32,
    pub build_mode: bool
}
#[derive(Serialize,Deserialize,Clone)]
pub struct PlantSettings {
    pub spawn_rate: i8,
    pub spawn_radius: f32,
    pub energy: f32,
    pub protein: f32,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct FruitSettings {
    pub spawn_rate: i8,
    pub spawn_radius: f32,
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
#[derive(Serialize,Deserialize,Clone)]
pub struct WorldSettings{
    pub width: f32,
    pub height: f32,
    pub generate_plant_spawners: bool,
    pub generate_fruit_spawners: bool,
    pub generate_terrain: bool,
}
impl Default for SimParams{
    fn default() -> Self {
        Self{
            plants: PlantSettings {
                spawn_rate: 6,
                spawn_radius: 0.0,
                energy: 80.0,
                protein: 0.02,
            },
            fruit: FruitSettings {
                spawn_rate: 2,
                spawn_radius: 0.0,
                energy: 300.0,
                protein: 0.1,
            },
            animals: AnimalSettings{
                brain_mutation_rate: 6.0,
                brain_mutation_strength: 10.,
                physical_mutation_rate: 15.0,
                physical_mutation_strength: 10.0,
            },
            build: BuildSettings {
                pen_size: 0,
                build_mode: false
            },
            simulation: SimulationSettings {
                steps_per_frame: 1
            },
            world: WorldSettings {
                width: 120.0,
                height: 120.0,
                generate_plant_spawners: true,
                generate_fruit_spawners: true,
                generate_terrain: true,
            },
            highlighted_species: -1,
            save_id: 0,
        }
    }
}
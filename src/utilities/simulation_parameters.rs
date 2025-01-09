use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone)]
pub struct SimParams{
    pub plants: PlantSettings,
    pub fruit: FruitSettings,
    pub animals: AnimalSettings,
    pub build: BuildSettings,
    pub simulation: SimulationSettings,
    pub world: WorldSettings,
    pub save_id: usize,
    pub autosave: i32,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct SimulationSettings {
    pub steps_per_frame: i8,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct BuildSettings {
    pub pen_size: i32,
    pub place_fruit_spawner: bool,
    pub place_plant_spawner: bool,
    pub place_rock: bool,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct PlantSettings {
    pub global_spawn_rate: i8,
    pub spawn_rate: i8,
    pub spawn_radius: f32,
    pub energy: f32,
    pub protein: f32,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct FruitSettings {
    pub global_spawn_rate: i8,
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
    pub speciation_threshold: f32,
    pub carnivory_efficiency: f32,
    pub herbivory_efficiency: f32,
    pub speed_energy_cost: f32,
    pub turning_energy_cost: f32,
    pub size_energy_cost: f32,
    pub attack_energy_cost: f32,
    pub vision_energy_cost: f32,
    pub speed_protein_cost: f32,
    pub size_protein_cost: f32,
    pub attack_protein_cost: f32,
    pub movement_speed: f32,
    pub turning_speed: f32,
    pub reproduction_time: f32,
    pub reproduction_protein_cost: f32,
    pub reproduction_energy_cost: f32,
    pub lifespan: f32,
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
                global_spawn_rate: 5,
                spawn_rate: 6,
                spawn_radius: 15.,
                energy: 80.0,
                protein: 0.02,
            },
            fruit: FruitSettings {
                global_spawn_rate: 1,
                spawn_rate: 2,
                spawn_radius: 10.0,
                energy: 300.0,
                protein: 0.1,
            },
            animals: AnimalSettings{
                brain_mutation_rate: 6.0,
                brain_mutation_strength: 10.,
                physical_mutation_rate: 15.0,
                physical_mutation_strength: 10.0,
                speciation_threshold: 0.1,
                carnivory_efficiency: 1.0,
                herbivory_efficiency: 1.0,
                speed_energy_cost: 1.0,
                turning_energy_cost: 1.0,
                size_energy_cost: 1.0,
                attack_energy_cost: 1.0,
                vision_energy_cost: 1.0,
                speed_protein_cost: 1.0,
                size_protein_cost: 1.0,
                attack_protein_cost: 1.0,
                movement_speed: 1.0,
                turning_speed: 1.0,
                reproduction_time: 1.0,
                reproduction_protein_cost: 1.0,
                reproduction_energy_cost: 1.0,
                lifespan: 1.0,
            },
            build: BuildSettings {
                pen_size: 0,
                place_fruit_spawner: false,
                place_plant_spawner: false,
                place_rock: false,
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
            save_id: 0,
            autosave: 300,
        }
    }
}
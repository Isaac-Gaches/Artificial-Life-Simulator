
mod rendering;
mod utilities;
mod environment;

use std::ops::Index;
use rendering::render::Renderer;

use winit::event::WindowEvent;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::Key,
    window::WindowBuilder,
};
use std::sync::Arc;
use std::time::SystemTime;
use sysinfo::System;
use winit::dpi::PhysicalSize;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use crate::environment::animal::Animals;
use crate::environment::collisions::{DIV};
use crate::environment::fruit::FruitSpawners;
use crate::environment::plants::PlantSpawners;
use crate::environment::rocks::RockMap;
use crate::rendering::camera::Camera;
use crate::utilities::input_manager::Inputs;
use crate::utilities::save_system::SaveSystem;
use crate::utilities::state::State;

fn main() {
    pollster::block_on(run());
}

//const WORLD_WIDTH: f32 = 120.0;
//const WORLD_HEIGHT: f32 = 120.0;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title("EcoSim").with_inner_size(PhysicalSize::new(1200, 800)).build(&event_loop).unwrap());
    let mut renderer = Renderer::new(window).await;
    let mut step = 0;
    let mut animals = Animals::genesis();
    let mut plants = environment::plants::Plants::genesis();
    let mut fruit = environment::fruit::Fruits::genesis();
    let mut stats = utilities::statistics::Stats::default();
    let mut eggs = environment::eggs::Eggs::default();
    let mut sim_params = utilities::simulation_parameters::SimParams::default();
    let mut species_list = environment::species::SpeciesList::default();
    let mut graph_timer = SystemTime::now();
    let mut diagnostic_timer = SystemTime::now();
    let mut inputs = Inputs::default();
    let mut frames = 0;
    let mut system = System::default();
    let mut state = State{ menu: true, load_save: false, new: false };
    let mut camera = Camera{
        position: [sim_params.world.width/2.0,sim_params.world.height/2.0],
        zoom: 0.05,
        ratio: 1.0,
    };
    let mut collisions = environment::collisions::Collisions::new(&sim_params);
    let mut inspected_animal_id = 0;
    let mut inspected_animal = None;
    let mut follow = false;
    let mut rocks = RockMap::new(&collisions);
    let mut plant_spawners = PlantSpawners{ bodies: vec![] };
    plant_spawners.random(&sim_params);
    let mut fruit_spawners = FruitSpawners{ bodies: vec![] };
    fruit_spawners.random(&sim_params);

    let _ = event_loop.run(move |event, ewlt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() => {
            match event {
                WindowEvent::CloseRequested => ewlt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if !state.menu && event.state == ElementState::Pressed && !event.repeat {
                        match event.key_without_modifiers().as_ref() {
                            Key::Character("w") => { inputs.up = true; follow = false; },
                            Key::Character("s") => { inputs.down = true; follow = false; },
                            Key::Character("a") => { inputs.left = true; follow = false; },
                            Key::Character("d") => { inputs.right = true; follow = false;},
                            Key::Character("=") => inputs.plus = true,
                            Key::Character("-") => inputs.minus = true,
                            Key::Character("q") => {
                                animals.kill();
                                plants.kill();
                                fruit.kill();
                                SaveSystem::save(step, animals.clone(), plants.clone(), fruit.clone(), eggs.clone(), species_list.clone(),stats.clone(),sim_params.clone(),rocks.clone());

                                collisions.update_animal_grid(animals.instances().as_slice());
                                collisions.update_plant_grid(plants.instances());
                                collisions.update_fruit_grid(fruit.instances());
                            },
                            _ => (),
                        }
                    }
                    else if event.state == ElementState::Released {
                        match event.key_without_modifiers().as_ref() {
                            Key::Character("w") => inputs.up = false,
                            Key::Character("s") => inputs.down = false,
                            Key::Character("a") => inputs.left = false,
                            Key::Character("d") => inputs.right = false,
                            Key::Character("=") => inputs.plus = false,
                            Key::Character("-") => inputs.minus = false,
                            _ => (),
                        }
                    }
                }
                WindowEvent::MouseInput { button,state,..} =>{
                    match button {
                        MouseButton::Left =>{
                            inputs.left_mouse = state.is_pressed();
                        },
                        MouseButton::Right =>{
                            inputs.right_mouse = state.is_pressed();
                        },
                        _ => (),
                    }
                }
                WindowEvent::CursorMoved {position,..} =>{
                    inputs.mouse_pos = [(position.x as f32/renderer.window_width() - 0.5)*2.0,((renderer.window_height() - position.y as f32)/renderer.window_height() - 0.5)*2.0];
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(Some(*physical_size));
                }
                WindowEvent::RedrawRequested => {
                    if state.menu{
                        match renderer.main_menu(&mut state,&mut sim_params) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                renderer.resize(None);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                            Err(wgpu::SurfaceError::Timeout) => {},
                        }
                    }
                    else if state.load_save {
                        (step, animals, plants, fruit, eggs, species_list, stats, sim_params, rocks) = SaveSystem::load().open();
                        collisions = environment::collisions::Collisions::new(&sim_params);
                        collisions.update_animal_grid(animals.instances().as_slice());
                        collisions.update_plant_grid(plants.instances());
                        collisions.update_fruit_grid(fruit.instances());
                        state.load_save = false;
                    }
                    else if state.new {
                        sim_params.world.height = sim_params.world.width;
                        let world_settings = sim_params.world.clone();
                        sim_params = utilities::simulation_parameters::SimParams::default();
                        sim_params.world = world_settings;

                        camera.position = [sim_params.world.width/2.0,sim_params.world.height/2.0];

                        step = 0;
                        stats = utilities::statistics::Stats::default();

                        collisions = environment::collisions::Collisions::new(&sim_params);
                        animals = Animals::genesis();
                        eggs = environment::eggs::Eggs::default();
                        plants = environment::plants::Plants::genesis();
                        fruit = environment::fruit::Fruits::genesis();
                        species_list = environment::species::SpeciesList::default();

                        rocks = RockMap::new(&collisions);
                        plant_spawners = PlantSpawners{ bodies: vec![] };
                        fruit_spawners = FruitSpawners{ bodies: vec![] };

                        if sim_params.world.generate_terrain{
                            rocks.randomise();
                        }
                        if sim_params.world.generate_fruit_spawners{
                            fruit_spawners.random(&sim_params);
                        }
                        if sim_params.world.generate_plant_spawners{
                            plant_spawners.random(&sim_params);
                        }

                        state.new = true;
                    }
                    else {
                        inspected_animal = if let Some(animal) = animals.animals.iter().find(|animal|{
                            animal.id == inspected_animal_id
                        }){
                            Some(animal.clone())
                        } else{
                            None
                        };

                        if diagnostic_timer.elapsed().unwrap().as_millis() >= 1000{
                            stats.update_diagnostics(frames,&mut system);
                            frames = 0;
                            diagnostic_timer = SystemTime::now();
                        }
                        if !sim_params.build.build_mode {
                            for _ in 0..sim_params.simulation.steps_per_frame {
                                if graph_timer.elapsed().unwrap().as_millis() >= 1000 / sim_params.simulation.steps_per_frame as u128 {
                                    stats.update_graphs(animals.count(), plants.count(), &animals.animals);
                                    graph_timer = SystemTime::now();
                                }

                                if step % 3600 == 0 {
                                    for _ in 0..sim_params.plants.spawn_rate{
                                        plant_spawners.spawn(&mut plants,&rocks,&collisions,&sim_params);
                                    }
                                    for _ in 0..sim_params.fruit.spawn_rate{
                                        fruit_spawners.spawn(&mut fruit,&rocks,&collisions,&sim_params);
                                    }
                                    if animals.count() < 30{
                                        animals.spawn(&sim_params);
                                    }
                                }

                                if step % 6 == 0 {
                                    animals.kill();
                                    plants.kill();
                                    fruit.kill();
                                    collisions.update_animal_grid(animals.instances().as_slice());
                                    collisions.update_plant_grid(plants.instances());
                                    collisions.update_fruit_grid(fruit.instances());
                                }

                                collisions.handle_collisions(&mut animals, &mut plants, &mut fruit, &sim_params);
                                eggs.update(&mut animals);
                                animals.update(&mut plants,&mut fruit, &mut eggs, &mut sim_params, &collisions, &mut species_list, &rocks);

                                step += 1;
                            }
                        }
                        else if !renderer.egui_context().is_pointer_over_area() {
                            if inputs.left_mouse {
                                rocks.set(1, camera.screen_to_world_pos(inputs.mouse_pos), sim_params.build.pen_size);
                                plants.remove_plants_in_walls(&rocks);
                                fruit.remove_plants_in_walls(&rocks);
                                collisions.update_plant_grid(plants.instances());
                                collisions.update_fruit_grid(fruit.instances());
                            } else if inputs.right_mouse {
                                rocks.set(0, camera.screen_to_world_pos(inputs.mouse_pos), sim_params.build.pen_size);
                            }
                        }

                        if !renderer.egui_context().is_pointer_over_area() {
                            if inputs.left_mouse {
                                let pos = camera.screen_to_world_pos(inputs.mouse_pos);
                                if let Some(i) = collisions.animals_grid[(pos[0] * DIV) as usize * collisions.cells_height + (pos[1] * DIV) as usize].object_ids.last(){
                                    let animal = animals.animals.index(*i);
                                    inspected_animal_id = animal.id;
                                    follow = true;
                                }

                            }
                        }

                        camera.update(&inputs,&renderer.size(),follow,&inspected_animal);
                        renderer.update(&animals,&plants,&fruit,&eggs,&rocks,camera);

                        match renderer.render(&mut stats,&mut sim_params,&inspected_animal,&mut state) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                renderer.resize(None);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                            Err(wgpu::SurfaceError::Timeout) => {},
                        }

                        frames+=1;
                    }

                    renderer.window().request_redraw();
                }
                _ => {}
            };
            renderer.egui_handle_input(event);
        }
        _ => {}
    });
}
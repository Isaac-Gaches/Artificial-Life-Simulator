
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
use crate::environment::temperature::TemperatureMap;
use crate::rendering::camera::Camera;
use crate::utilities::highlighter::Highlighter;
use crate::utilities::input_manager::Inputs;
use crate::utilities::save_system::SaveSystem;
use crate::utilities::state::State;

fn main() {
    pollster::block_on(run());
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title("EcoSim").with_inner_size(PhysicalSize::new(1200, 800)).build(&event_loop).unwrap());
   // let window = Arc::new(ActiveEventLoop::create_window().unwrap());

    let mut save_syatem = SaveSystem::default();
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
    let mut state = State::Menu;
    let mut camera = Camera{
        position: [sim_params.world.width/2.0,sim_params.world.height/2.0],
        zoom: 0.05,
        ratio: 1.0,
    };
    let mut collisions = environment::collisions::Collisions::new(&sim_params);
    let mut inspected_animal_id = 0;
    let mut inspected_animal = None;
    let mut follow = false;
    let mut rocks = RockMap::new(collisions.cells_height);
    let mut plant_spawners = PlantSpawners{ bodies: vec![] };
    plant_spawners.random(&sim_params);
    let mut fruit_spawners = FruitSpawners{ bodies: vec![] };
    fruit_spawners.random(&sim_params);
    let mut highlighter = Highlighter::default();
    let mut temp_map = TemperatureMap::new(collisions.cells_height);

    let _ = event_loop.run(move |event, ewlt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() => {
            match event {
                WindowEvent::CloseRequested => ewlt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if state != State::Menu && event.state == ElementState::Pressed && !event.repeat {
                        match event.key_without_modifiers().as_ref() {
                            Key::Character("w") => { inputs.up = true; follow = false; },
                            Key::Character("s") => { inputs.down = true; follow = false; },
                            Key::Character("a") => { inputs.left = true; follow = false; },
                            Key::Character("d") => { inputs.right = true; follow = false;},
                            Key::Character("=") => inputs.plus = true,
                            Key::Character("-") => inputs.minus = true,
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
                    match state {
                        State::Exit =>{
                            ewlt.exit()
                        }
                        State::Menu | State::CreateSim => {
                            match renderer.main_menu(&mut state,&mut sim_params,&mut save_syatem) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    renderer.resize(None);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                                Err(wgpu::SurfaceError::Timeout) => {},
                            }
                        }
                        State::LoadSave =>{
                            (step, animals, plants, fruit, eggs, species_list, stats, sim_params, rocks,fruit_spawners,plant_spawners) = save_syatem.load(sim_params.save_id).open();
                            collisions = environment::collisions::Collisions::new(&sim_params);
                            collisions.update_animal_grid(animals.instances().as_slice());
                            collisions.update_plant_grid(plants.instances());
                            collisions.update_fruit_grid(fruit.instances());

                            temp_map = TemperatureMap::new(collisions.cells_height);
                            temp_map.set(sim_params.temp.plant_spawner_temp, plant_spawners.instances());
                            temp_map.set(sim_params.temp.fruit_spawner_temp, fruit_spawners.instances());
                            temp_map.update(sim_params.temp.spread,sim_params.temp.smooth,&rocks.rocks);

                            state = State::RunSim;
                        }
                        State::NewSim =>{
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

                            rocks = RockMap::new(collisions.cells_height);
                            plant_spawners = PlantSpawners{ bodies: vec![] };
                            fruit_spawners = FruitSpawners{ bodies: vec![] };

                            if sim_params.world.generate_terrain{
                                rocks.randomise();
                            }

                            fruit_spawners.random(&sim_params);
                            plant_spawners.random(&sim_params);

                            highlighter = Highlighter::default();

                            temp_map = TemperatureMap::new(collisions.cells_height);
                            temp_map.set(sim_params.temp.plant_spawner_temp, plant_spawners.instances());
                            temp_map.set(sim_params.temp.fruit_spawner_temp, fruit_spawners.instances());
                            temp_map.update(sim_params.temp.spread,sim_params.temp.smooth,&rocks.rocks);

                            state = State::RunSim;
                        }
                        State::SaveSim => {
                            animals.kill();
                            plants.kill();
                            fruit.kill();
                            save_syatem.save(step, animals.clone(), plants.clone(), fruit.clone(), eggs.clone(), species_list.clone(),stats.clone(),sim_params.clone(),rocks.clone(),fruit_spawners.clone(),plant_spawners.clone());
                            collisions.update_animal_grid(animals.instances().as_slice());
                            collisions.update_plant_grid(plants.instances());
                            collisions.update_fruit_grid(fruit.instances());
                            highlighter.set_highlights(&animals);

                            state = State::RunSim;
                        }
                        State::RunSim=>{
                            inspected_animal = animals.animals.iter().find(|animal|{
                                animal.id == inspected_animal_id
                            }).cloned();

                            if diagnostic_timer.elapsed().unwrap().as_millis() >= 1000{
                                stats.update_diagnostics(frames,&mut system);
                                frames = 0;
                                diagnostic_timer = SystemTime::now();
                            }

                            for _ in 0..sim_params.simulation.steps_per_frame {
                                if step % (sim_params.autosave*3600) == 0 && step > 0{
                                    state = State::SaveSim;
                                }

                                if graph_timer.elapsed().unwrap().as_millis() >= 1000 / sim_params.simulation.steps_per_frame as u128 {
                                    stats.update_graphs(animals.count(),fruit.count(), plants.count(), &animals.animals);
                                    graph_timer = SystemTime::now();
                                }

                                if step % 3600 == 0 {
                                    for _ in 0..sim_params.plants.spawn_rate{
                                        plant_spawners.spawn(&mut plants,&rocks,&collisions,&sim_params);
                                    }
                                    for _ in 0..sim_params.fruit.spawn_rate{
                                        fruit_spawners.spawn(&mut fruit,&rocks,&collisions,&sim_params);
                                    }

                                    for _ in 0..sim_params.plants.global_spawn_rate{
                                        plants.spawn_random(&rocks, &collisions, &sim_params);
                                    }
                                    for _ in 0..sim_params.fruit.global_spawn_rate{
                                        fruit.spawn_random(&rocks, &collisions, &sim_params);
                                    }

                                    if animals.count() < 20{
                                        animals.spawn(&sim_params);
                                        animals.spawn(&sim_params);
                                        animals.spawn(&sim_params);
                                        animals.spawn(&sim_params);
                                        animals.spawn(&sim_params);
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
                                    highlighter.set_highlights(&animals);
                                }

                                highlighter.move_highlights(&animals);

                                collisions.handle_collisions(&mut animals, &mut plants, &mut fruit, &sim_params);
                                eggs.update(&mut animals);
                                animals.update(&mut plants,&mut fruit, &mut eggs, &mut sim_params, &collisions, &mut species_list, &rocks);

                                step += 1;
                            }
                            if !renderer.egui_context().is_pointer_over_area(){
                                if !(sim_params.build.place_fruit_spawner || sim_params.build.place_rock || sim_params.build.place_plant_spawner){
                                    if inputs.left_mouse {
                                        let pos = camera.screen_to_world_pos(inputs.mouse_pos);
                                        if pos[0] > 0. && pos[0] < sim_params.world.width && pos[1] >0. && pos[1] < sim_params.world.height {
                                            if let Some(i) = collisions.animals_grid[(pos[0] * DIV) as usize * collisions.cells_height + (pos[1] * DIV) as usize].object_ids.last() {
                                                let animal = animals.animals.index(*i);
                                                inspected_animal_id = animal.id;
                                                follow = true;
                                            }
                                        }
                                    }
                                }
                                else if inputs.left_mouse {
                                    let mut update = false;
                                    update = update || if sim_params.build.place_rock { rocks.set(1, camera.screen_to_world_pos(inputs.mouse_pos), sim_params.build.pen_size) } else { false };
                                    update = update || if sim_params.build.place_fruit_spawner { fruit_spawners.place(camera.screen_to_world_pos(inputs.mouse_pos),&sim_params) } else { false };
                                    update = update || if sim_params.build.place_plant_spawner { plant_spawners.place(camera.screen_to_world_pos(inputs.mouse_pos),&sim_params) } else { false };

                                    if update {
                                        plants.remove_plants_in_walls(&rocks);
                                        fruit.remove_plants_in_walls(&rocks);

                                        collisions.update_plant_grid(plants.instances());
                                        collisions.update_fruit_grid(fruit.instances());

                                        temp_map.clear();
                                        temp_map.set(sim_params.temp.plant_spawner_temp, plant_spawners.instances());
                                        temp_map.set(sim_params.temp.fruit_spawner_temp, fruit_spawners.instances());
                                        temp_map.update(sim_params.temp.spread,sim_params.temp.smooth,&rocks.rocks);
                                    }
                                } else if inputs.right_mouse {
                                    let mut update = false;
                                    update = update || if sim_params.build.place_rock { rocks.set(0, camera.screen_to_world_pos(inputs.mouse_pos), sim_params.build.pen_size) } else { false };
                                    update = update || if sim_params.build.place_fruit_spawner { fruit_spawners.remove(camera.screen_to_world_pos(inputs.mouse_pos)) } else { false };
                                    update = update || if sim_params.build.place_plant_spawner { plant_spawners.remove(camera.screen_to_world_pos(inputs.mouse_pos)) } else { false };

                                    if update {
                                        temp_map.clear();
                                        temp_map.set(sim_params.temp.plant_spawner_temp, plant_spawners.instances());
                                        temp_map.set(sim_params.temp.fruit_spawner_temp, fruit_spawners.instances());
                                        temp_map.update(sim_params.temp.spread,sim_params.temp.smooth,&rocks.rocks);
                                    }
                                }
                            }

                            camera.update(&inputs,&renderer.size(),follow,&inspected_animal);

                            let circles = [highlighter.instances().as_slice(),fruit.instances().as_slice(),eggs.instances().as_slice(),plants.instances().as_slice()].concat();
                            let squares = [temp_map.instances().as_slice(),rocks.instances().as_slice(),fruit_spawners.instances().as_slice(),plant_spawners.instances().as_slice()].concat();
                            let triangles = animals.instances();

                            let circle_count = circles.len() as u32;
                            let square_count = squares.len() as u32;
                            let triangle_count = triangles.len() as u32;

                            renderer.update(circle_count,square_count,triangle_count,circles,squares,triangles,camera);

                            match renderer.render(&mut stats,&mut sim_params,&inspected_animal,&mut state,&mut highlighter) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    renderer.resize(None);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                                Err(wgpu::SurfaceError::Timeout) => {},
                            }

                            frames+=1;
                        }
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
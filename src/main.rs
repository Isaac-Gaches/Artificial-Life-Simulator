mod gui;
mod statistics;
mod render;
mod animal;
mod plants;
mod neural_network;
mod eggs;
mod collisions;
mod simulation_parameters;
mod species;
mod input_manager;
mod save_system;

use std::fs;
use render::Renderer;

use winit::event::WindowEvent;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key},
    window::{WindowBuilder},
};
use std::sync::Arc;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use animal::*;
use statistics::Stats;
use crate::collisions::Collisions;
use crate::eggs::Eggs;
use crate::input_manager::Inputs;
use crate::plants::Plants;
use crate::simulation_parameters::SimParams;
use crate::species::SpeciesList;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use crate::save_system::SaveSystem;

fn main() {
    pollster::block_on(run());
}

const WORLD_WIDTH: f32 = 80.0;
const WORLD_HEIGHT: f32 = 80.0;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let mut renderer = Renderer::new(window).await;

    let mut graph_timer = SystemTime::now();
    let mut diagnostic_timer = SystemTime::now();
    let mut inputs = Inputs::default();
    let mut frames = 0;
    let mut system = System::default();

   /* let step = 0;
    let animals = Animals::genesis();
    let plants = Plants::genesis();
    let stats = Stats::default();
    let eggs = Eggs::default();
    let collisions = Collisions::new();
    let sim_params = SimParams::default();
    let species_list = SpeciesList::default();*/

    let (mut step, mut animals, mut plants, mut eggs, mut collisions, mut species_list, mut stats, mut sim_params) = SaveSystem::load().open();

    let _ = event_loop.run(move |event, ewlt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() => {
            match event {
                WindowEvent::CloseRequested => ewlt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed && !event.repeat {
                        match event.key_without_modifiers().as_ref() {
                            Key::Character("w") => inputs.up = true,
                            Key::Character("s") => inputs.down = true,
                            Key::Character("a") => inputs.left = true,
                            Key::Character("d") => inputs.right = true,
                            Key::Character("=") => inputs.plus = true,
                            Key::Character("-") => inputs.minus = true,
                            Key::Character("q") => { SaveSystem::save(step, animals.clone(), plants.clone(), eggs.clone(), collisions.clone(), species_list.clone(),stats.clone(),sim_params.clone()); },
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
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(Some(*physical_size));
                }
                WindowEvent::RedrawRequested => {
                    if diagnostic_timer.elapsed().unwrap().as_millis() >= 1000{
                        stats.update_diagnostics(frames,&mut system);
                        frames = 0;
                        diagnostic_timer = SystemTime::now();
                    }
                    for _ in 0..sim_params.steps_per_frame{
                        if graph_timer.elapsed().unwrap().as_millis() >= 1000/sim_params.steps_per_frame as u128{
                            stats.update_graphs(animals.count(), plants.count(), &animals.animals);
                            graph_timer = SystemTime::now();
                        }

                        if step%60==0{
                            for _ in 0..sim_params.plant_spawn_rate{
                                plants.spawn();
                            }
                            for _ in 0..1{
                                animals.spawn();
                            }
                        }

                        if step%4 == 0{
                            animals.kill();
                            plants.kill();
                            collisions.update_animal_grid(animals.instances().as_slice());
                            collisions.update_plant_grid(plants.instances());
                        }

                        collisions.handle_collisions(&mut animals,&mut plants);

                        eggs.update(&mut animals);
                        animals.update(&mut plants,&mut eggs,&mut sim_params,&collisions,&mut species_list);

                        step+=1;
                    }

                    renderer.update(&animals,&plants,&eggs,&inputs);

                    let net = if animals.animals.len() > 0 { Some(&animals.animals[0]) } else { None };

                    match renderer.render(&mut stats,&mut sim_params,net) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            renderer.resize(None);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                        Err(wgpu::SurfaceError::Timeout) => {},
                    }

                    frames+=1;
                    renderer.window().request_redraw();
                }
                _ => {}
            };
            renderer.egui_handle_input(event);
        }
        _ => {}
    });
}
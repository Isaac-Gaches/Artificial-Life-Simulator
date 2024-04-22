mod gui;
mod statistics;
mod render;
mod animal;
mod plants;
mod neural_network;
mod meat;

use render::Renderer;

use winit::event::WindowEvent;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};
use std::sync::Arc;
use std::time::SystemTime;
use animal::*;
use statistics::Stats;
use crate::meat::Meat;
use crate::plants::Plants;

fn main() {
    pollster::block_on(run());
}

struct Main {
    renderer: Renderer,
    animals: Animals,
    meat: Meat,
    plants: Plants,
    stats: Stats,
}

impl Main {
    async fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(window).await;
        let animals = Animals::genesis();
        let plants = Plants::genesis();
        let stats = Stats::default();
        let meat = Meat::genesis();

        Self {
            renderer,
            animals,
            plants,
            stats,
            meat,
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        self.plants.update();
        self.meat.update();
        self.animals.update(&mut self.plants,&mut self.meat);
        self.renderer.update(&self.animals,&self.plants,&self.meat);
    }

    fn render(&mut self)-> Result<(), wgpu::SurfaceError>{
        self.renderer.render(&mut self.stats)
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let mut main = Main::new(window).await;
    let mut timer = SystemTime::now();
    let mut frames = 0;

    let _ = event_loop.run(move |event, ewlt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == main.renderer.window().id() => {
            if !main.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                        ..
                    } => ewlt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        main.renderer.resize(Some(*physical_size));
                    }
                    WindowEvent::RedrawRequested => {
                        if timer.elapsed().unwrap().as_secs() > 0 {
                            main.stats.update(frames-1,main.animals.count(),main.plants.count(),main.meat.count());
                            for _ in 0..12{
                                main.plants.spawn();
                            }
                            frames = 0;
                            timer = SystemTime::now();
                        }
                        main.update();
                        match main.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                main.renderer.resize(None);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                            Err(wgpu::SurfaceError::Timeout) => {},
                        }

                        frames+=1;
                        main.renderer.window().request_redraw();
                    }
                    _ => {}
                };
                main.renderer.egui_handle_input(event);
            }
        }
        _ => {}
    });
}
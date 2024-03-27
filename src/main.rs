mod gui;
mod statistics;
mod compute;
mod render;

use render::Render;
use compute::Compute;

use wgpu::{Device, Queue};
use winit::event::WindowEvent;
use std::iter;

use wgpu::util::DeviceExt;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowBuilder},
};
use std::sync::Arc;
use std::time::SystemTime;

use crate::statistics::Stats;

fn main() {
    pollster::block_on(run());
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 2],
    color: [f32; 3],
    pad:[u32;3], //align to 16 bytes
}

/*#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    position: [f32; 2],
}*/

struct State {
    device: Device,
    queue: Queue,
    compute: Compute,
    render: Render,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    stats: Stats,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Default::default(),
                    required_limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let instances = (0..200000).map(|i| {
            let x = i as f32 % 1000.;
            let y = (i as f32/1000.).trunc();
            Instance {
                position: [(x*0.01)-9., (y*0.01)-0.99],
                color: [x*0.001, 1.0 - y*0.01, y*0.01],
                pad: [0,0,0],
            }
        }).collect::<Vec<Instance>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(instances.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        let compute = Compute::new(&device,&instance_buffer,&shader);

        let render = Render::new(&device, &shader, surface,window, &adapter);

        let stats = Stats::default();

        Self {
            device,
            queue,
            instance_buffer,
            instances,
            compute,
            render,
            stats,
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Encoder"),
        });
        self.compute.compute_pass(&mut encoder);
        self.render.update(&mut self.queue);
        self.queue.submit(iter::once(encoder.finish()));
    }

    fn render(&mut self)-> Result<(), wgpu::SurfaceError>{
        self.render.render(&self.device, &self.queue, &self.stats, &self.instances, &self.instance_buffer)
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut state = State::new(window).await;

    let mut timer = SystemTime::now();
    let mut frames = 0;

    let _ = event_loop.run(move |event, ewlt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.render.window().id() => {
            if !state.input(event) {
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
                        state.render.resize(Some(*physical_size),&state.device);
                    }
                    WindowEvent::RedrawRequested => {
                        if timer.elapsed().unwrap().as_secs() > 0 {
                            state.stats.update(frames as f64);
                            frames = 0;
                            timer = SystemTime::now();
                        }
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.render.resize(None,&state.device);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        }

                        frames+=1;
                        state.render.window().request_redraw();
                    }
                    _ => {}
                };
                state.render.egui_handle_input(event);
            }
        }
        _ => {}
    });
}
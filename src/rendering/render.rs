use std::{iter, mem};
use std::sync::Arc;
use egui_wgpu::{ScreenDescriptor};
use serde::{Deserialize, Serialize};
use wgpu::{BindingType, Buffer, Queue, Surface, Device, TextureViewDescriptor};
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;
use crate::environment::animal::{Animal, Animals};
use crate::environment::eggs::Eggs;
use crate::rendering::gui::{EguiRenderer, gui};
use crate::utilities::input_manager::Inputs;
use crate::environment::plants::Plants;
use crate::utilities::simulation_parameters::SimParams;
use crate::utilities::statistics::Stats;
use crate::{WORLD_HEIGHT, WORLD_WIDTH};
use crate::environment::rocks::RockMap;
use crate::rendering::instance::Instance;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}
const TRIANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [0.8, 0.3],
    },
    Vertex {
        position: [0.0, 0.6],
    },
];

const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];
const NUM_INDICES: u32 = 6;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    position: [f32;2],
    zoom: f32,
    ratio: f32,
}

pub struct Renderer {
    device: Device,
    queue: Queue,
    window: Arc<Window>,
    surface: Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_circles: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    camera: Camera,
    egui: EguiRenderer,
    buffers: Buffers
}
struct Buffers{
    camera_buffer: Buffer,
    triangle_vertex_buffer: Buffer,
    quad_vertex_buffer: Buffer,
    quad_index_buffer: Buffer,
    animal_buffer: Buffer,
    animal_count: u32,
    plant_buffer: Buffer,
    plant_count: u32,
    egg_buffer: Buffer,
    egg_count: u32,
    rock_buffer: Buffer,
    rock_count: u32,
}

impl Renderer {
    pub async fn new(window: Arc<Window>)->Self{
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

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 0,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let camera = Camera{
            position: [WORLD_WIDTH/2.0,WORLD_HEIGHT/2.0],
            zoom: 0.05,
            ratio: 1.0,
        };

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: mem::size_of::<Camera>() as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry{
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry{
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                        ],
                    }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let render_pipeline_circles = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_circle",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                },
                    wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32,
                            },
                            wgpu::VertexAttribute {
                                offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                        ],
                    }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_circle",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState{
                        color: wgpu::BlendComponent{
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,},
                        alpha: wgpu::BlendComponent::OVER
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let triangle_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Buffer"),
            contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let egui = EguiRenderer::new(
            &device,
            config.format,
            None,
            1,
            &window,
        );

        let animal_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Buffer to render animals"),
            size: 32768,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let plant_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Buffer to render plants"),
            size: 10000000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let egg_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Buffer to eggs"),
            size: 8192,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rock_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Buffer to rocks"),
            size: 10000000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buffers = Buffers{
            camera_buffer,
            triangle_vertex_buffer,
            quad_vertex_buffer,
            quad_index_buffer,
            animal_buffer,
            animal_count: 0,
            plant_buffer,
            plant_count: 0,
            egg_buffer,
            egg_count: 0,
            rock_buffer,
            rock_count: 0,
        };

        Self{
            window,
            device,
            queue,
            surface,
            config,
            size,
            render_pipeline,
            render_pipeline_circles,
            camera_bind_group,
            camera,
            egui,
            buffers
        }
    }

    pub fn render(&mut self, stats: &mut Stats,sim_params: &mut SimParams,animal: Option<&Animal>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.02,
                        g: 0.1,
                        b: 0.2,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        //animals
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0,&self.camera_bind_group,&[]);
        render_pass.set_vertex_buffer(0, self.buffers.triangle_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.buffers.animal_buffer.slice(..));
        render_pass.draw(0..3,0..self.buffers.animal_count);

        //rocks
        render_pass.set_vertex_buffer(0, self.buffers.quad_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.buffers.rock_buffer.slice(..));
        render_pass.set_index_buffer(self.buffers.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..NUM_INDICES, 0, 0..self.buffers.rock_count);

        //plants
        render_pass.set_pipeline(&self.render_pipeline_circles);
        render_pass.set_vertex_buffer(1, self.buffers.plant_buffer.slice(..));
        render_pass.draw_indexed(0..NUM_INDICES, 0, 0..self.buffers.plant_count);

        //eggs
        render_pass.set_vertex_buffer(1, self.buffers.egg_buffer.slice(..));
        render_pass.draw_indexed(0..NUM_INDICES, 0, 0..self.buffers.egg_count);


        drop(render_pass);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.egui.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
            gui,
            stats,
            sim_params,
            animal,
        );

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: Option<winit::dpi::PhysicalSize<u32>>) {
        let new_size= match new_size {
            Some(new_size) =>{
                new_size
            }
            None=>{
                self.size
            }
        };

        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self,animals: &Animals,plants: &Plants,eggs: &Eggs,inputs: &Inputs,rocks: &RockMap){
        self.buffers.animal_count = animals.count() as u32;
        self.buffers.plant_count = plants.count() as u32;
        self.buffers.egg_count = eggs.count() as u32;
        self.buffers.rock_count = rocks.count() as u32;

        self.camera.position[1] += if inputs.up {0.1} else if inputs.down {-0.1} else {0.0};
        self.camera.position[0] += if inputs.right {0.1} else if inputs.left {-0.1} else {0.0};
        self.camera.zoom += if inputs.plus {0.002} else if inputs.minus {-0.002} else {0.0};
        self.camera.ratio = self.size.height as f32/self.size.width as f32;

        self.queue.write_buffer(&self.buffers.camera_buffer,0,bytemuck::cast_slice(&[self.camera]));
        self.queue.write_buffer(&self.buffers.animal_buffer, 0, bytemuck::cast_slice(animals.instances().as_slice()));
        self.queue.write_buffer(&self.buffers.plant_buffer,0,bytemuck::cast_slice(plants.instances().as_slice()));
        self.queue.write_buffer(&self.buffers.egg_buffer,0,bytemuck::cast_slice(eggs.instances().as_slice()));
        self.queue.write_buffer(&self.buffers.rock_buffer,0,bytemuck::cast_slice(rocks.instances().as_slice()));
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn egui_handle_input(&mut self,event: &WindowEvent){
        self.egui.handle_input(&self.window, event);
    }
}
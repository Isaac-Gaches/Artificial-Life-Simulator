use egui::{Align2, Color32, Context, emath, Frame, RichText, Sense, Stroke, Ui, Vec2, Visuals};
use egui::epaint::Shadow;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use epaint::Pos2;
use epaint::{CircleShape, Rect, Shape};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::keyboard::NamedKey::New;
use winit::window::Window;
use crate::environment::animal::Animal;
use crate::utilities::highlighter::{Condition, Highlighter, SelectedHighlight};
use crate::utilities::save_system::SaveSystem;
use crate::utilities::simulation_parameters::{Pen, SimParams};
use crate::utilities::state::State::{CreateSim, Exit, LoadSave, Menu, NewSim, RunSim, SaveSim};
use crate::utilities::statistics::Stats;

#[derive(Default)]
pub struct Toggles{
    population_graphs: bool,
    food_settings:bool,
    animal_settings: bool,
    simulation_settings: bool,
    build_settings: bool,
    diagnostics: bool,
    distributions: bool,
    animal_inspect: bool,
    highlighter_settings: bool,
    populations: Populations,
    highlight_selected: bool,
}
#[derive(Default)]
pub struct Populations{
    animals: bool,
    herbivores: bool,
    omnivores: bool,
    carnivores: bool,
    speed: bool,
    size: bool,
}
pub struct EguiRenderer {
    pub context: Context,
    toggles: Toggles,
    state: State,
    pub renderer: Renderer,
}

impl EguiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 5.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = State::new(egui_context.clone(), id, &window, None, None);

        let egui_renderer = Renderer::new(device, output_color_format, output_depth_format, msaa_samples);

        let mut toggles = Toggles::default();
        toggles.populations.animals = true;

        EguiRenderer {
            context: egui_context,
            state: egui_state,
            toggles,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: impl FnOnce(&Context,&mut Stats,&mut Toggles,&mut SimParams,&Option<Animal>,&mut crate::utilities::state::State, &mut Highlighter),
        stats: &mut Stats,
        sim_params: &mut SimParams,
        animal: &Option<Animal>,
        state: &mut crate::utilities::state::State,
        highlighter: &mut Highlighter
    ) {
        let raw_input = self.state.take_egui_input(window);

        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context,stats,&mut self.toggles,sim_params,animal,state,highlighter);
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
    pub fn draw_main_menu(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        run_ui: impl FnOnce(&Context,&mut crate::utilities::state::State,&mut SimParams,&mut SaveSystem,&PhysicalSize<u32>),
        state: &mut crate::utilities::state::State,
        sim_params: &mut SimParams,
        save_system: &mut SaveSystem,
        screen_size: &PhysicalSize<u32>
    ) {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context,state,sim_params,save_system,screen_size);
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}

pub fn gui(ui: &Context,stats: &mut Stats,toggles: &mut Toggles,sim_params: &mut SimParams,animal: &Option<Animal>, state: &mut crate::utilities::state::State,highlighter: &mut Highlighter) {
    egui::SidePanel::right("right")
        .resizable(false)
        .default_width(200.)
        .show(ui,|ui|{
            ui.heading("Statistics");
            ui.separator();

            if ui.selectable_label(toggles.animal_inspect, RichText::new("Inspector").heading()).clicked(){
                toggles.animal_inspect = !toggles.animal_inspect;
            }
            if ui.selectable_label(toggles.population_graphs, RichText::new("Population Graphs").heading()).clicked(){
                toggles.population_graphs = !toggles.population_graphs;
            }
            if ui.selectable_label(toggles.distributions, RichText::new("Distributions").heading()).clicked(){
                toggles.distributions = !toggles.distributions;
            }
            if ui.selectable_label(toggles.diagnostics, RichText::new("Diagnostics").heading()).clicked(){
                toggles.diagnostics = !toggles.diagnostics;
            }

            ui.separator();
            ui.heading("Settings");
            ui.separator();

            if ui.selectable_label(toggles.food_settings, RichText::new("Food Settings").heading()).clicked(){
                toggles.food_settings = !toggles.food_settings;
            }
            if ui.selectable_label(toggles.animal_settings, RichText::new("Animal Settings").heading()).clicked(){
                toggles.animal_settings = !toggles.animal_settings;
            }
            if ui.selectable_label(toggles.build_settings, RichText::new("Build Settings").heading()).clicked(){
                toggles.build_settings = !toggles.build_settings;
            }
            if ui.selectable_label(toggles.highlighter_settings, RichText::new("Highlighter Settings").heading()).clicked(){
                toggles.highlighter_settings = !toggles.highlighter_settings;
            }
            if ui.selectable_label(toggles.simulation_settings, RichText::new("Simulation Settings").heading()).clicked(){
                toggles.simulation_settings = !toggles.simulation_settings;
            }

            ui.add(egui::DragValue::new(&mut sim_params.temp.smooth).clamp_range(0..=80));

            ui.separator();
            ui.heading("System");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Autosave").heading());
                ui.add(egui::DragValue::new(&mut sim_params.autosave).clamp_range(60..=6000).speed(10).suffix(" min"));
            });
            if ui.add_sized([180.,30.],egui::Button::new(RichText::new("Save").heading())).clicked(){
                *state = SaveSim;
            }
            if ui.add_sized([180.,30.],egui::Button::new(RichText::new("Main Menu").heading())).clicked(){
                *state = Menu;
            }
        });

    if toggles.animal_inspect {
        egui::Window::new("Network")
            .default_width(550.0)
            .resizable(false)
            .collapsible(false)
            .show(ui, |ui| {
                if let Some(animal) = animal {
                    ui.horizontal(|ui| {
                        ui.label("  Plant vision ");
                        ui.separator();
                        ui.label("             Animal vision             ");
                        ui.separator();
                        ui.label("Rock vision");
                        ui.separator();
                        ui.label("Internal state");
                        ui.separator();
                        ui.label("      Endocrine");
                    });
                    Frame::canvas(ui.style()).show(ui, |ui| {
                        let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_width() * 0.5), Sense::hover());

                        let to_screen = emath::RectTransform::from_to(
                            Rect::from_min_size(epaint::Pos2::ZERO, response.rect.square_proportions()),
                            response.rect,
                        );

                        let network = &animal.brain.network;

                        let spacing_x = 0.9 / (network.layers.len() as f32 - 1.0);
                        let neurons: Vec<Shape> = network.layers.iter().enumerate().flat_map(|(i, layer)| {
                            let spacing = 1.9 / (layer.neurons.len() - 1) as f32;
                            layer.neurons.iter().enumerate().map(move |(j, neuron)| {
                                let c = (neuron.activation * 255.) as u8;
                                let fill = Color32::from_rgb(c, c, c);

                                Shape::Circle(CircleShape {
                                    center: to_screen * Pos2::new(0.05 + j as f32 * spacing, 0.05 + i as f32 * spacing_x),
                                    radius: 6.0,
                                    fill,
                                    stroke: Default::default(),
                                })
                            })
                        }).collect();

                        let synapses: Vec<Shape> = network.layers.iter().enumerate().flat_map(|(i, layer)| {
                            let spacing = 1.9 / (layer.neurons.len() - 1) as f32;

                            layer.neurons.iter().enumerate().flat_map(move |(j, neuron)| {
                                let p1 = Pos2::new(0.05 + j as f32 * spacing, 0.05 + i as f32 * spacing_x);
                                let spacing2 = 1.9 / (neuron.weights.len() as f32 - 1.);

                                neuron.weights.iter().enumerate().map(move |(k, weight)| {
                                    let color = if *weight > 0. { Color32::from_rgb(0, 255, 0) } else { Color32::from_rgb(255, 0, 0) };

                                    let width = (weight * 2.0).abs().min(3.0);

                                    let p2 = Pos2::new(0.05 + k as f32 * spacing2, 0.05 + (i as f32 - 1.0) * spacing_x);

                                    Shape::LineSegment { points: [to_screen.transform_pos(p1), to_screen.transform_pos(p2)], stroke: Stroke{ width, color }}
                                })
                            })
                        }).collect();

                        painter.extend(synapses);
                        painter.extend(neurons);
                    });
                    ui.horizontal(|ui| {
                        ui.label("    Move    ");
                        ui.separator();
                        ui.label("                           Turn                           ");
                        ui.separator();
                        ui.label("                               Endocrine");
                    });

                    ui.separator();

                    ui.horizontal(|ui|{
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Species: {}", animal.species_id)));
                            ui.label(RichText::new(format!("Maturity: {}", animal.maturity)));
                            ui.label(RichText::new(format!("Age (min): {:.2}", animal.age/60.)));
                            ui.label(RichText::new(format!("Generation: {}", animal.generation)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Energy: {:.2}", animal.resources.energy)));
                            ui.label(RichText::new(format!("Protein: {:.2}", animal.resources.protein)));
                            ui.label(RichText::new(format!("Mass: {:.2}", animal.lean_mass)));
                            ui.label(RichText::new(format!("Hue: {:.2}", animal.hue)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Animal vision: {:.2}", animal.senses.animal_vision)));
                            ui.label(RichText::new(format!("Plant vision: {:.2}", animal.senses.plant_vision)));
                            ui.label(RichText::new(format!("Fruit vision: {:.2}", animal.senses.fruit_vision)));
                            ui.label(RichText::new(format!("Rock vision: {:.2}", animal.senses.rock_vision)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Offspring invest: {:.2}", animal.reproduction_stats.offspring_investment)));
                            ui.label(RichText::new(format!("Birth recovery (s): {:.2}", animal.reproduction_stats.birth_timer)));
                            ui.label(RichText::new(format!("Aggression: {:.2}", animal.combat_stats.aggression)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Speed: {:.2}", animal.combat_stats.speed)));
                            ui.label(RichText::new(format!("Size: {:.2}", animal.body.scale)));
                            ui.label(RichText::new(format!("Carnivore factor: {:.2}", animal.combat_stats.carnivore_factor)));
                            ui.label(RichText::new(format!("Attack: {:.2}", animal.combat_stats.attack)));
                        });
                    });
                }
                else{
                    ui.label(RichText::new("No animal selected"));
                }
            });
    }
    if toggles.population_graphs {
        egui::Window::new("Population graphs")
            .default_width(550.0)
            .resizable(true)
            .collapsible(false)
            .show(ui, |ui| {
                ui.collapsing(RichText::new("Animals"),|ui|{
                    let animals =Line::new(PlotPoints::new(stats.populations.animals.clone())).color(Color32::WHITE);

                    let herb = Line::new(PlotPoints::new(stats.populations.herbivores.clone())).color(Color32::GREEN);
                    let omni = Line::new(PlotPoints::new(stats.populations.omnivores.clone())).color(Color32::GOLD);
                    let carn = Line::new(PlotPoints::new(stats.populations.carnivores.clone())).color(Color32::RED);

                    ui.horizontal(|ui|{
                        ui.vertical(|ui|{
                            ui.add(egui::Checkbox::new(&mut toggles.populations.animals,"All"));
                            ui.add(egui::Checkbox::new(&mut toggles.populations.herbivores,"Herbivore"));
                            ui.add(egui::Checkbox::new(&mut toggles.populations.omnivores,"Omnivores"));
                            ui.add(egui::Checkbox::new(&mut toggles.populations.carnivores,"Carnivores"));
                        });
                        Plot::new("animal population graph").view_aspect(2.0).show(ui, |plot_ui| {
                            if toggles.populations.animals{ plot_ui.line(animals); }
                            if toggles.populations.herbivores{ plot_ui.line(herb); }
                            if toggles.populations.omnivores{ plot_ui.line(omni); }
                            if toggles.populations.carnivores{ plot_ui.line(carn); }

                        });
                    });
                });

                ui.collapsing(RichText::new("Averages"),|ui|{
                    let speed = Line::new(PlotPoints::new(stats.populations.average_speed.clone())).color(Color32::LIGHT_BLUE);
                    let size = Line::new(PlotPoints::new(stats.populations.average_size.clone())).color(Color32::from_rgb(255,180,25));

                    ui.horizontal(|ui|{
                        ui.vertical(|ui|{
                            ui.add(egui::Checkbox::new(&mut toggles.populations.speed,"Avg speed"));
                            ui.add(egui::Checkbox::new(&mut toggles.populations.size,"Avg size"));
                        });
                        Plot::new("averages graph").view_aspect(2.0).show(ui, |plot_ui| {
                            if toggles.populations.speed{ plot_ui.line(speed); }
                            if toggles.populations.size{ plot_ui.line(size); }
                        });
                    });
                });

                ui.collapsing(RichText::new("Food"),|ui|{
                    let plants =Line::new(PlotPoints::new(stats.populations.plants.clone())).color(Color32::GREEN);
                    let fruit =Line::new(PlotPoints::new(stats.populations.fruit.clone())).color(Color32::LIGHT_GREEN);

                    Plot::new("plant population graph").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.line(plants);
                        plot_ui.line(fruit);
                    });
                });

                ui.separator();

                if ui.selectable_label(false, RichText::new("Clear graphs")).clicked(){
                    stats.clear_graph_data();
                }
            });
    }
    if toggles.diagnostics {
        egui::Window::new("Diagnostics")
            .resizable(false)
            .collapsible(false)
            .show(ui, |ui| {
                ui.label(RichText::new(format!("FPS: {}",stats.fps)));
                ui.label(RichText::new(format!("Total CPU usage: {:.2}%",stats.tot_cpu_usage)));
                ui.label(RichText::new(format!("Total memory: {} mB",stats.tot_mem/8000000)));
                ui.label(RichText::new(format!("Used memory: {} mB",stats.used_mem/8000000)));
                ui.collapsing(RichText::new("CPU usage breakdown"),|ui|{
                    stats.cpu_usages.iter().enumerate().for_each(|(i,usage)|{
                        ui.label(RichText::new(format!("CPU {} Usage: {:.1}%",i+1,usage)));
                    })
                });
            });
    }
    if toggles.distributions {
        egui::Window::new("Distributions")
            .default_width(550.0)
            .collapsible(false)
            .show(ui, |ui| {
                ui.collapsing(RichText::new("Diet"),|ui|{
                    let bars = stats.distributions.diet.iter().enumerate().map(|(i,diet)|{
                        Bar::new(i as f64 * 0.1, *diet).fill(Color32::from_rgba_unmultiplied(i as u8 * 25,255 - i as u8 * 25,25,80)).width(0.08)
                            .stroke(Stroke::new(1., Color32::from_rgb(i as u8 * 25,255 - i as u8 * 25,25)))
                    }).collect();
                    let chart = BarChart::new(bars);

                    Plot::new("Diet").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
                });
                ui.collapsing(RichText::new("Speed"),|ui|{
                    let bars = stats.distributions.speed.iter().enumerate().map(|(i,speed)|{
                        Bar::new(i as f64 * 0.1, *speed).fill(Color32::from_rgba_unmultiplied(i as u8 * 15,i as u8 * 25,255,80)).width(0.08)
                        .stroke(Stroke::new(1., Color32::from_rgb(i as u8 * 15,i as u8 * 25,255)))
                    }).collect();
                    let chart = BarChart::new(bars);

                    Plot::new("Speed").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
                });
                ui.collapsing(RichText::new("Size"),|ui|{
                    let bars = stats.distributions.size.iter().enumerate().map(|(i,size)|{
                        Bar::new(i as f64 * 0.1, *size).fill(Color32::from_rgba_unmultiplied(255,255-i as u8 * 18,25,80)).width(0.08)
                        .stroke(Stroke::new(1., Color32::from_rgb(255,255-i as u8 * 18,25)))
                    }).collect();
                    let chart = BarChart::new(bars);

                    Plot::new("Size").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.bar_chart(chart);
                    });
                });
            });
    }
    if toggles.simulation_settings {
        egui::Window::new("Simulation settings")
            .resizable(false)
            .collapsible(false)
            .show(ui, |ui| {
                ui.horizontal(|ui|{
                    ui.label("Speed");
                    ui.add(egui::DragValue::new(&mut sim_params.simulation.steps_per_frame).clamp_range(0..=200));
                });
                ui.horizontal(|ui| {
                    ui.label("Stats refresh time");
                    ui.add(egui::DragValue::new(&mut stats.step_time).clamp_range(1..=600));
                });
            });
    }
    if toggles.food_settings {
        egui::Window::new("Food settings")
            .resizable(false)
            .default_width(0.0)
            .collapsible(false)
            .show(ui, |ui| {
                ui.heading("Plants");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Global spawn rate");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.global_spawn_rate).clamp_range(0..=60));
                });
                ui.horizontal(|ui| {
                    ui.label("Feeder spawn rate");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.spawn_rate).clamp_range(0..=60));
                });
                ui.horizontal(|ui| {
                    ui.label("Feeder spawn radius");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.spawn_radius).clamp_range(5..=30));
                });
                ui.horizontal(|ui| {
                    ui.label("Energy");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.energy).clamp_range(0.0..=500.).max_decimals(2));
                });
                ui.horizontal(|ui| {
                    ui.label("Protein");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.protein).clamp_range(0.0..=10.).speed(0.1).max_decimals(2));
                });

                ui.separator();
                ui.heading("Fruit");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Global spawn rate");
                    ui.add(egui::DragValue::new(&mut sim_params.fruit.global_spawn_rate).clamp_range(0..=60));
                });
                ui.horizontal(|ui| {
                    ui.label("Feeder spawn rate");
                    ui.add(egui::DragValue::new(&mut sim_params.fruit.spawn_rate).clamp_range(0..=60));
                });
                ui.horizontal(|ui| {
                    ui.label("Feeder spawn radius");
                    ui.add(egui::DragValue::new(&mut sim_params.fruit.spawn_radius).clamp_range(5..=30));
                });
                ui.horizontal(|ui| {
                    ui.label("Energy");
                    ui.add(egui::DragValue::new(&mut sim_params.fruit.energy).clamp_range(0.0..=500.).max_decimals(2));
                });
                ui.horizontal(|ui| {
                    ui.label("Protein");
                    ui.add(egui::DragValue::new(&mut sim_params.fruit.protein).clamp_range(0.0..=10.).speed(0.1).max_decimals(2));
                });
            });
    }
    if toggles.animal_settings {
        egui::Window::new("Animal settings")
            .resizable(false)
            .collapsible(false)
            .default_width(0.0)
            .show(ui, |ui| {
                ui.heading("Mutations");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Brain mutation rate");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.brain_mutation_rate).clamp_range(0..=100));
                });
                ui.horizontal(|ui|{
                    ui.label("Brain mutation strength");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.brain_mutation_strength).clamp_range(0..=100));
                });
                ui.horizontal(|ui|{
                    ui.label("Physical mutation rate");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.physical_mutation_rate).clamp_range(0..=100));
                });
                ui.horizontal(|ui|{
                    ui.label("Physical mutation strength");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.physical_mutation_strength).clamp_range(0..=100));
                });

                ui.separator();
                ui.heading("Diet efficiency");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Herbivore efficiency");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.herbivory_efficiency).clamp_range(0.0..=1.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Carnivore efficiency");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.carnivory_efficiency).clamp_range(0.0..=1.0).speed(0.01).max_decimals(2));
                });

                ui.separator();
                ui.heading("Energy use");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Speed energy cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.speed_energy_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Turning energy cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.turning_energy_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Size energy cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.size_energy_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Attack energy cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.attack_energy_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Vision energy cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.vision_energy_cost).clamp_range(0.0..=205.0).speed(0.01).max_decimals(2));
                });

                ui.separator();
                ui.heading("Protein use");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Speed protein cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.speed_protein_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Size protein cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.size_protein_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Attack protein cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.attack_protein_cost).clamp_range(0.0..=20.0).speed(0.01).max_decimals(2));
                });

                ui.separator();
                ui.heading("Movement");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Movement speed");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.movement_speed).clamp_range(0.0..=4.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Turning speed");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.turning_speed).clamp_range(0.0..=4.0).speed(0.01).max_decimals(2));
                });

                ui.separator();
                ui.heading("Reproduction");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Reproduction energy cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.reproduction_energy_cost).clamp_range(0.0..=5.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Reproduction protein cost");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.reproduction_protein_cost).clamp_range(0.0..=5.0).speed(0.01).max_decimals(2));
                });
                ui.horizontal(|ui|{
                    ui.label("Reproduction time");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.reproduction_time).clamp_range(0.0..=20.0).speed(0.1).max_decimals(1));
                });

                ui.separator();
                ui.heading("Other");
                ui.separator();

                ui.horizontal(|ui|{
                    ui.label("Lifespan multiplier");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.lifespan).clamp_range(0.0..=100.0));
                });
                ui.horizontal(|ui|{
                    ui.label("Speciation threshold");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.speciation_threshold).clamp_range(0.0..=1.0).speed(0.01).max_decimals(2));
                });
            });
    }
    if toggles.build_settings {
        egui::Window::new("Build settings")
            .resizable(false)
            .collapsible(false)
            .default_width(0.0)
            .show(ui, |ui| {
                egui::ComboBox::from_label("Pen")
                    .selected_text(format!("{:?}", &sim_params.build.pen))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut  sim_params.build.pen, Pen::None, "None");
                        ui.selectable_value(&mut  sim_params.build.pen, Pen::Rock, "Rock");
                        ui.selectable_value(&mut sim_params.build.pen, Pen::PlantGenerator, "Plant Generator");
                        ui.selectable_value(&mut  sim_params.build.pen, Pen::FruitGenerator, "Fruit Generator");
                    });

                match sim_params.build.pen {
                    Pen::Rock =>{
                        ui.horizontal(|ui|{
                            ui.label("Pen size");
                            ui.add(egui::DragValue::new(&mut sim_params.build.pen_size).clamp_range(0..=6));
                        });
                    },
                    _ =>{},
                }
            });
    }
    if toggles.highlighter_settings {
        egui::Window::new("Highlighter settings")
            .resizable(false)
            .collapsible(false)
            .default_width(0.0)
            .show(ui, |ui| {

                egui::ComboBox::from_label("Highlight")
                    .selected_text(format!("{:?}", highlighter.selected_highlight))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut  highlighter.selected_highlight, SelectedHighlight::None, "None");
                        ui.selectable_value(&mut  highlighter.selected_highlight, SelectedHighlight::Diet, "Diet");
                        ui.selectable_value(&mut highlighter.selected_highlight, SelectedHighlight::Size, "Size");
                        ui.selectable_value(&mut  highlighter.selected_highlight, SelectedHighlight::Speed, "Speed");
                        ui.selectable_value(&mut  highlighter.selected_highlight, SelectedHighlight::Species, "Species");
                    });

                match highlighter.selected_highlight {
                    SelectedHighlight::Diet =>{
                        highlight_settings(ui,&mut highlighter.diet);
                    },
                    SelectedHighlight::Size =>{
                        highlight_settings(ui,&mut highlighter.size);
                    }
                    SelectedHighlight::Speed =>{
                        highlight_settings(ui,&mut highlighter.speed);
                    },
                    SelectedHighlight::Species =>{
                        ui.horizontal(|ui|{
                            ui.label("Species Id");
                            ui.add(egui::DragValue::new(&mut highlighter.species_id));
                        });
                    }
                    SelectedHighlight::None=>{},
                }
            });
    }
}
fn highlight_settings(ui: &mut Ui,con: &mut Condition){
    ui.horizontal(|ui|{
        ui.label("Bounded");
        ui.add(egui::Checkbox::new(&mut con.bounded,""));
    });
    ui.horizontal(|ui|{
        ui.label("Lower bound");
        ui.add(egui::DragValue::new(&mut con.lower).clamp_range(0..=1).speed(0.01));
    });
    ui.horizontal(|ui|{
        ui.label("Upper bound");
        ui.add(egui::DragValue::new(&mut con.upper).clamp_range(0..=1).speed(0.01));
    });
}

pub fn main_menu_gui(ui: &Context, state: &mut crate::utilities::state::State,sim_params: &mut SimParams, save_system: &mut SaveSystem, screen_size: &PhysicalSize<u32>) {
    egui::Window::new("Main Menu").default_width(0.).anchor(Align2::CENTER_CENTER, [0.,0.]).collapsible(false).resizable(false).fixed_pos([screen_size.width as f32/2. , screen_size.height as f32/2.]).show(ui, |ui|{
        match state{
            Menu =>{
                if ui.add_sized([288., 50.], egui::Button::new(RichText::new("New Simulation").heading())).clicked(){
                    *state = CreateSim;
                }

                ui.separator();

                egui::ScrollArea::vertical().max_height(108.).show(ui, |ui| {
                    for i in (0..save_system.saves.len()).rev() {
                        ui.horizontal(|ui|{
                            if ui.add_sized([240., 50.], egui::Button::new(RichText::new(["Load",&save_system.saves[i]].join(" ")).heading())).clicked(){
                                sim_params.save_id = i;
                                *state = LoadSave;
                            }
                            if ui.add_sized([40., 50.], egui::Button::new(RichText::new("X"))).clicked(){
                                save_system.delete(i);
                            }
                        });
                    }
                    if save_system.saves.is_empty(){
                        ui.label(RichText::new("No Saves").heading());
                    }
                });

                ui.separator();

                if ui.add_sized([288., 50.], egui::Button::new(RichText::new("Exit").heading())).clicked(){
                    *state = Exit;
                }
            }
            CreateSim => {
                if ui.add_sized([288., 50.], egui::Button::new(RichText::new("Create Simulation").heading())).clicked(){
                    *state = NewSim;
                }
                if ui.add_sized([288., 40.], egui::Button::new(RichText::new("Back").heading())).clicked(){
                    *state = Menu;
                }

                ui.separator();

                ui.add(egui::DragValue::new(&mut sim_params.world.width).prefix("World size: ").clamp_range(10..=200));
                ui.add(egui::Checkbox::new(&mut sim_params.world.generate_terrain,"Terrain"));
                ui.add(egui::DragValue::new(&mut sim_params.world.plant_spawners).prefix("Plant feeders: ").clamp_range(0..=200));
                ui.add(egui::DragValue::new(&mut sim_params.world.fruit_spawners).prefix("Fruit feeders: ").clamp_range(0..=200));
            }
            _ => {}
        }
    });
}
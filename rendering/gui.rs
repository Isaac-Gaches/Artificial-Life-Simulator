use egui::{Color32, Context, emath, Frame, Pos2, RichText, Sense, Stroke, Vec2, Visuals};
use egui::epaint::Shadow;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use epaint::{CircleShape, Rect, Shape};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window;
use crate::environment::animal::Animal;
use crate::utilities::simulation_parameters::SimParams;
use crate::utilities::statistics::Stats;

#[derive(Default)]
pub struct Toggles{
    population_graphs: bool,
    plant_settings: bool,
    animal_settings: bool,
    animals: bool,
    herbivores: bool,
    omnivores: bool,
    carnivores: bool,
    diagnostics: bool,
    distributions: bool,
    animal_inspect: bool
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

        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
        );

        let mut toggles = Toggles::default();
        toggles.animals = true;

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
        run_ui: impl FnOnce(&Context,&mut Stats,&mut Toggles,&mut SimParams,Option<&Animal>),
        stats: &mut Stats,
        sim_params: &mut SimParams,
        animal: Option<&Animal>
    ) {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context,stats,&mut self.toggles,sim_params,animal);
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

pub fn gui(ui: &Context,stats: &mut Stats,toggles: &mut Toggles,sim_params: &mut SimParams,animal: Option<&Animal>) {
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

            if ui.selectable_label(toggles.plant_settings, RichText::new("Plant Settings").heading()).clicked(){
                toggles.plant_settings = !toggles.plant_settings;
            }
            if ui.selectable_label(toggles.animal_settings, RichText::new("Animal Settings").heading()).clicked(){
                toggles.animal_settings = !toggles.animal_settings;
            }
            ui.horizontal(|ui| {
                ui.label("Stats Refresh Time");
                ui.add(egui::DragValue::new(&mut stats.step_time).clamp_range(1..=300));
            });
            ui.horizontal(|ui|{
                ui.label("Steps Per Frame");
                ui.add(egui::DragValue::new(&mut sim_params.steps_per_frame).clamp_range(0..=100));
            });
            ui.horizontal(|ui|{
                ui.label("Build Mode");
                ui.add(egui::Checkbox::new(&mut sim_params.build_mode,""));
            });
            ui.horizontal(|ui|{
                ui.label("Pen Size");
                ui.add(egui::DragValue::new(&mut sim_params.pen_size).clamp_range(0..=6));
            });

            ui.separator();

            ui.horizontal(|ui|{
                ui.label("Species");
                ui.add(egui::DragValue::new(&mut sim_params.highlighted_species).clamp_range(-1..=1000));
            });
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
                            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
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

                                    Shape::LineSegment { points: [to_screen.transform_pos(p1), to_screen.transform_pos(p2)], stroke: Stroke { width, color } }
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
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Energy: {:.2}", animal.resources.energy)));
                            ui.label(RichText::new(format!("Protein: {:.2}", animal.resources.protein)));
                            ui.label(RichText::new(format!("Mass: {:.2}", animal.lean_mass)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Carnivore factor: {:.2}", animal.combat_stats.carnivore_factor)));
                            ui.label(RichText::new(format!("Attack: {:.2}", animal.combat_stats.attack)));
                            ui.label(RichText::new(format!("Offspring Invest: {:.2}", animal.reproduction_stats.offspring_investment)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Speed: {:.2}", animal.combat_stats.speed)));
                            ui.label(RichText::new(format!("Size: {:.2}", animal.body.scale)));
                            ui.label(RichText::new(format!("Hue: {:.2}", animal.hue)));
                        });
                        ui.vertical(|ui|{
                            ui.label(RichText::new(format!("Animal vision: {:.2}", animal.senses.animal_vision)));
                            ui.label(RichText::new(format!("Plant vision: {:.2}", animal.senses.plant_vision)));
                            ui.label(RichText::new(format!("Rock vision: {:.2}", animal.senses.rock_vision)));
                        });
                    });
                }
                else{
                    ui.label(RichText::new("No Animal Selected"));
                }
            });
    }
    if toggles.population_graphs {
        egui::Window::new("Population Graphs")
            .default_width(550.0)
            .resizable(true)
            .collapsible(false)
            .show(ui, |ui| {
                ui.collapsing(RichText::new("Animals"),|ui|{
                    let animals =Line::new(PlotPoints::new(stats.animal_pop.clone())).color(Color32::WHITE);
                    let herb = Line::new(PlotPoints::new(stats.herb_pop.clone())).color(Color32::GREEN);
                    let omni = Line::new(PlotPoints::new(stats.omni_pop.clone())).color(Color32::GOLD);
                    let carn = Line::new(PlotPoints::new(stats.carn_pop.clone())).color(Color32::RED);

                    ui.horizontal(|ui|{
                        ui.vertical(|ui|{
                            ui.add(egui::Checkbox::new(&mut toggles.animals,"All"));
                            ui.add(egui::Checkbox::new(&mut toggles.herbivores,"Herbivore"));
                            ui.add(egui::Checkbox::new(&mut toggles.omnivores,"Omnivores"));
                            ui.add(egui::Checkbox::new(&mut toggles.carnivores,"Carnivores"));
                        });
                        Plot::new("animal population graph").view_aspect(2.0).show(ui, |plot_ui| {
                            if toggles.animals{ plot_ui.line(animals); }
                            if toggles.herbivores{ plot_ui.line(herb); }
                            if toggles.omnivores{ plot_ui.line(omni); }
                            if toggles.carnivores{ plot_ui.line(carn); }
                        });
                    });
                });

                ui.collapsing(RichText::new("Plants"),|ui|{
                    let plants =Line::new(PlotPoints::new(stats.plant_pop.clone())).color(Color32::GREEN);

                    Plot::new("plant population graph").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.line(plants);
                    });
                });

                ui.separator();

                if ui.selectable_label(false, RichText::new("Clear Graphs")).clicked(){
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
                ui.label(RichText::new(format!("Total CPU Usage: {:.2}%",stats.tot_cpu_usage)));
                ui.label(RichText::new(format!("Total Memory: {} mB",stats.tot_mem/8000000)));
                ui.label(RichText::new(format!("Used Memory: {} mB",stats.used_mem/8000000)));
                ui.collapsing(RichText::new("CPU Usage Breakdown"),|ui|{
                    stats.cpu_usages.iter().enumerate().for_each(|(i,usage)|{
                        ui.label(RichText::new(format!("CPU {} Usage: {:.1}%",i+1,usage)));
                    })
                });
            });
    }
    if toggles.distributions {
        egui::Window::new("Distributions")
            .default_width(550.0)
            .resizable(false)
            .collapsible(false)
            .show(ui, |ui| {
                ui.collapsing(RichText::new("Diet"),|ui|{
                    let bars = stats.diet_dist.iter().enumerate().map(|(i,diet)|{
                        let bar =Bar::new(i as f64 + 1.0, *diet);
                        let bar = bar.fill(Color32::from_rgba_unmultiplied(i as u8 * 25,255 - i as u8 * 25,25,80));
                        bar.stroke(Stroke::new(1., Color32::from_rgb(i as u8 * 25,255 - i as u8 * 25,25)))
                    }).collect();
                    let test = BarChart::new(bars);

                    Plot::new("Diet").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.bar_chart(test);
                    });
                });
            });
    }
    if toggles.plant_settings {
        egui::Window::new("Plant Settings")
            .resizable(false)
            .collapsible(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Spawn rate");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.spawn_rate).clamp_range(0..=50));
                });
                ui.horizontal(|ui| {
                    ui.label("Max density");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.max_density).clamp_range(0.0..=2.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Energy");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.energy).clamp_range(0.0..=500.));
                });
                ui.horizontal(|ui| {
                    ui.label("Protein");
                    ui.add(egui::DragValue::new(&mut sim_params.plants.protein).clamp_range(0.0..=10.));
                });
            });
    }
    if toggles.animal_settings {
        egui::Window::new("Animal Settings")
            .resizable(false)
            .collapsible(false)
            .show(ui, |ui| {
                ui.horizontal(|ui|{
                    ui.label("Brain Mutation Rate");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.brain_mutation_rate).clamp_range(0..=100));
                });
                ui.horizontal(|ui|{
                    ui.label("Brain Mutation Strength");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.brain_mutation_strength).clamp_range(0..=100));
                });
                ui.horizontal(|ui|{
                    ui.label("Physical Mutation Rate");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.physical_mutation_rate).clamp_range(0..=100));
                });
                ui.horizontal(|ui|{
                    ui.label("Physical Mutation Strength");
                    ui.add(egui::DragValue::new(&mut sim_params.animals.physical_mutation_strength).clamp_range(0..=100));
                });
            });
    }
}
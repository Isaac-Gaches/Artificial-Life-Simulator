use egui::{Color32, Context, RichText, Stroke, Visuals};
use egui::epaint::Shadow;
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window;
use crate::simulation_parameters::SimParams;
use crate::statistics::Stats;

#[derive(Default)]
pub struct Toggles{
    population_graphs: bool,
    diagnostics: bool,
    distributions: bool,
}
pub struct EguiRenderer {
    pub context: Context,
    toggles: Toggles,
    state: State,
    renderer: Renderer,
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

        let toggles = Toggles::default();

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
        run_ui: impl FnOnce(&Context,&mut Stats,&mut Toggles,&mut SimParams),
        stats: &mut Stats,
        sim_params: &mut SimParams,
    ) {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context,stats,&mut self.toggles,sim_params);
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

pub fn gui(ui: &Context,stats: &mut Stats,toggles: &mut Toggles,sim_params: &mut SimParams) {
    egui::SidePanel::right("right")
        .resizable(false)
        .default_width(200.)
        .show(ui,|ui|{
            ui.heading("Statistics");
            ui.separator();
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
            ui.horizontal(|ui| {
                ui.label("Stats Refresh Time");
                ui.add(egui::DragValue::new(&mut stats.step_time).clamp_range(1..=300));
            });
            ui.horizontal(|ui|{
                ui.label("Steps Per Frame");
                ui.add(egui::DragValue::new(&mut sim_params.steps_per_frame).clamp_range(0..=8));
            });
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Species");
                ui.add(egui::DragValue::new(&mut sim_params.highlighted_species).clamp_range(-1..=1000));
            });
        });

    if toggles.population_graphs {
        egui::Window::new("Population Graphs")
            .default_open(true)
            .default_width(400.0)
            .resizable(false)
            .show(ui, |ui| {
                ui.collapsing(RichText::new("Plant Population"),|ui|{
                    let pop =Line::new(PlotPoints::new(stats.plant_pop.clone()));
                    let pop = pop.fill(0.).color(Color32::GREEN);

                    Plot::new("plant population").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.line(pop);
                    });
                });

                ui.collapsing(RichText::new("Animal Population"),|ui|{
                    let pop =Line::new(PlotPoints::new(stats.animal_pop.clone()));
                    let pop = pop.fill(0.).color(Color32::WHITE);

                    Plot::new("animal population").view_aspect(2.0).show(ui, |plot_ui| {
                        plot_ui.line(pop);
                    });
                });
            });
    }
    if toggles.diagnostics {
        egui::Window::new("Diagnostics")
            .default_open(true)
            .default_width(400.0)
            .resizable(false)
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
            .default_open(true)
            .default_width(400.0)
            .resizable(false)
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
}
use egui::{Align2, Color32, Context, Visuals};

use egui::epaint::Shadow;
use egui::WidgetType::Slider;
use egui_plot::{Line, Plot, PlotPoints};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window;
use crate::statistics::Stats;

pub struct EguiRenderer {
    pub context: Context,
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

        EguiRenderer {
            context: egui_context,
            state: egui_state,
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
        run_ui: impl FnOnce(&Context,&mut Stats),
        stats: &mut Stats,
    ) {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, |_ui| {
            run_ui(&self.context,stats);
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

pub fn gui(ui: &Context,stats: &mut Stats) {
    egui::Window::new("Statistics")
        .default_open(false)
        .default_width(400.0)
        .resizable(false)
        .anchor(Align2::LEFT_TOP, [10.0, 10.0])
        .show(ui, |ui| {
            ui.add(egui::Slider::new(&mut stats.step_time, 1..=10).text("Statistic Update Time"));

            ui.collapsing("diagnostics",|ui|{
                ui.label(format!("fps: {}",stats.fps));
                ui.label(format!("total cpu usage: {:.2}%",stats.tot_cpu_usage));
                ui.label(format!("total memory: {} mB",stats.tot_mem/8000000));
                ui.label(format!("used memory: {} mB",stats.used_mem/8000000));
                ui.collapsing("cpu usage breakdown",|ui|{
                    stats.cpu_usages.iter().enumerate().for_each(|(i,usage)|{
                        ui.label(format!("cpu {} usage: {:.1}%",i+1,usage));
                    })
                });
            });

            ui.collapsing("plant population",|ui|{
                let pop =Line::new(PlotPoints::new(stats.plant_pop.clone()));
                let pop = pop.fill(0.).color(Color32::GREEN);

                Plot::new("animal population").view_aspect(2.0).show(ui, |plot_ui| {
                    plot_ui.line(pop);
                });
            });

            ui.collapsing("animal population",|ui|{
                let pop =Line::new(PlotPoints::new(stats.animal_pop.clone()));
                let pop = pop.fill(0.).color(Color32::RED);

                Plot::new("animal population").view_aspect(2.0).show(ui, |plot_ui| {
                    plot_ui.line(pop);
                });
            });
        });
}
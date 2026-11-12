use std::fmt;

use winit::window::Window;

use crate::engine::render::wgpu_ctx::OverlayTarget;
use crate::game::level::list_levels;

// What the player did on the main menu this frame.
pub enum MainMenuAction {
    None,
    Load(String),
    Quit,
}

// Owns everything egui needs: the context, the winit input bridge and the wgpu renderer.
pub struct Gui {
    ctx: egui::Context,
    winit_state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    levels: Vec<String>,
}

impl fmt::Debug for Gui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gui").finish_non_exhaustive()
    }
}

impl Gui {
    pub fn new(window: &Window, device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Gui {
        let ctx = egui::Context::default();
        let winit_state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );
        Gui {
            ctx,
            winit_state,
            renderer,
            levels: list_levels(),
        }
    }

    // Re-scan src/mapdata for levels (call when the menu is shown again).
    pub fn refresh_levels(&mut self) {
        self.levels = list_levels();
    }

    // Feed every winit event through this first. `consumed` means egui used it
    // (e.g. a click on a button), so the game should ignore it.
    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.winit_state.on_window_event(window, event)
    }

    // Builds the main menu, then records an egui render pass on top of whatever the
    // game already drew. Returns true if egui wants another frame right away
    // (hover animations, etc.).
    pub fn render_main_menu(
        &mut self,
        window: &Window,
        target: OverlayTarget<'_>,
        action: &mut MainMenuAction,
    ) -> bool {
        let raw_input = self.winit_state.take_egui_input(window);

        let levels = &self.levels;
        let mut full_output = self.ctx.run_ui(raw_input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.15);
                    ui.heading(egui::RichText::new("HEXENG").size(48.0).strong());
                    ui.add_space(24.0);

                    if levels.is_empty() {
                        ui.label("No levels found in src/mapdata/");
                    }
                    for name in levels {
                        let button = egui::Button::new(egui::RichText::new(name).size(20.0))
                            .min_size(egui::vec2(260.0, 40.0));
                        if ui.add(button).clicked() {
                            *action = MainMenuAction::Load(name.clone());
                        }
                    }

                    ui.add_space(24.0);
                    let quit = egui::Button::new(egui::RichText::new("Quit").size(20.0))
                        .min_size(egui::vec2(260.0, 40.0));
                    if ui.add(quit).clicked() {
                        *action = MainMenuAction::Quit;
                    }
                });
            });
        });

        self.winit_state
            .handle_platform_output(window, full_output.platform_output);

        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = self.ctx.tessellate(full_output.shapes, pixels_per_point);

        // egui panics (in debug builds) if a TexturesDelta is dropped with entries still
        // in it, so take them out of `full_output` instead of just borrowing them.
        for (id, deltas) in std::mem::take(&mut full_output.textures_delta.set) {
            for delta in &deltas {
                self.renderer
                    .update_texture(target.device, target.queue, id, delta);
            }
        }

        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: target.size_in_pixels,
            pixels_per_point,
        };
        self.renderer.update_buffers(
            target.device,
            target.queue,
            target.encoder,
            &paint_jobs,
            &screen,
        );

        {
            let mut rpass = target
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load, // keep the game underneath
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            self.renderer.render(&mut rpass, &paint_jobs, &screen);
        }

        for id in std::mem::take(&mut full_output.textures_delta.free) {
            self.renderer.free_texture(&id);
        }

        full_output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .is_some_and(|viewport| viewport.repaint_delay.is_zero())
    }
}

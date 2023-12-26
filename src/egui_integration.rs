use egui::{Context, TexturesDelta, ViewportId};
use egui_wgpu::{
    renderer::ScreenDescriptor,
    wgpu::{CommandEncoderDescriptor, Device, Queue, RenderPassDescriptor, TextureFormat},
    Renderer,
};
use egui_winit::State;
use winit::{dpi::PhysicalSize, window::Window};

pub struct EguiIntegration {
    context: Context,
    delta: TexturesDelta,
    renderer: Renderer,
    state: State,
    triangles: Vec<egui::ClippedPrimitive>,
    window_size: PhysicalSize<u32>,
}

impl EguiIntegration {
    pub fn new(window: &Window, device: &Device, texture_format: TextureFormat) -> Self {
        let mut state = State::new(
            ViewportId::default(),
            window,
            Some(window.scale_factor() as f32),
            Some(device.limits().max_texture_dimension_2d as usize),
        );

        let context = Context::default();
        context.set_zoom_factor(2.0);

        let renderer = Renderer::new(device, texture_format, None, 1);

        Self {
            context,
            delta: TexturesDelta::default(),
            renderer,
            state,
            triangles: Vec::new(),
            window_size: window.inner_size(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.window_size = new_size;
    }

    pub fn prepare(&mut self, window: &Window, run_ui: impl FnOnce(&Context)) {
        let raw_input = self.state.take_egui_input(window);
        let output = self.context.run(raw_input, run_ui);

        self.state
            .handle_platform_output(window, &self.context, output.platform_output);
        self.triangles = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        self.delta = output.textures_delta;
    }

    pub fn render<'rp>(&'rp mut self, device: &Device, queue: &Queue, desc: &RenderPassDescriptor) {
        let delta = std::mem::take(&mut self.delta);
        for (id, image_delta) in delta.set.iter() {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.window_size.width, self.window_size.height],
            pixels_per_point: self.context.pixels_per_point(),
        };

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("egui encoder"),
        });

        self.renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &self.triangles,
            &screen_descriptor,
        );

        let mut render_pass = encoder.begin_render_pass(desc);

        self.renderer
            .render(&mut render_pass, &self.triangles, &screen_descriptor);
        drop(render_pass);

        for id in delta.free.iter() {
            self.renderer.free_texture(id);
        }

        queue.submit([encoder.finish()]);
    }

    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> egui_winit::EventResponse {
        self.state.on_window_event(&self.context, event)
    }
}

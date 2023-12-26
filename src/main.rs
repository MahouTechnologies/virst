#![allow(unused_variables)]
#![allow(dead_code)]
use std::{sync::Arc, time::Instant};

use displayed_model::DisplayedModel;
use egui_integration::EguiIntegration;
use egui_wgpu::wgpu;
use example_scene_controller::ExampleSceneController;
use glam::{uvec2, Vec2};
use gui::Gui;
use inox2d::model::Model;
use inox2d_wgpu::Renderer;
use tracker::TrackerSystem;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
mod displayed_model;
mod egui_integration;
mod example_scene_controller;
mod gui;
mod tracker;

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1024, 1024))
        .with_resizable(true)
        .with_title("Virst")
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
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
                features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    // Fallback to first alpha mode if PreMultiplied is not supported
    let alpha_modes = surface.get_capabilities(&adapter).alpha_modes;
    let alpha_mode = if alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
        wgpu::CompositeAlphaMode::PreMultiplied
    } else {
        alpha_modes[0]
    };

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode,
        view_formats: Vec::new(),
    };

    let displayed_model = Arc::new(DisplayedModel::default());

    let mut current: Option<(Model, ExampleSceneController, Renderer)> = None;
    let mut generation: u32 = 0;

    let mut integration = EguiIntegration::new(&window, &device, wgpu::TextureFormat::Bgra8Unorm);
    let mut tracker_system = TrackerSystem::new();

    let mut gui = Gui::new(displayed_model.clone());

    use simple_moving_average::SMA;
    let mut ma = simple_moving_average::SumTreeSMA::<_, f64, 300>::new();
    let mut x = false;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            let now = Instant::now();
            let next_displayed = displayed_model.current_model();
            if next_displayed.1 > generation {
                if let Some(next) = next_displayed.0 {
                    let mut next_renderer = Renderer::new(
                        &device,
                        &queue,
                        wgpu::TextureFormat::Bgra8Unorm,
                        &next,
                        uvec2(window.inner_size().width, window.inner_size().height),
                    );
                    next_renderer.camera.scale = Vec2::splat(0.12);

                    let next_controller = ExampleSceneController::new(&next_renderer.camera, 0.5);

                    current = Some((next.as_ref().clone(), next_controller, next_renderer));
                } else {
                    current = None;
                }
                generation = next_displayed.1;
            }

            integration.prepare(&window, |ctx| {
                let data = gui.draw(ctx, &mut tracker_system);
                if data.exited {
                    control_flow.set_exit();
                }
            });

            let output = if let Ok(t) = surface.get_current_texture() {
                t
            } else {
                return;
            };

            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let temp_texture = device.create_texture(&wgpu::TextureDescriptor{
                label: None,
                size: output.texture.size(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: output.texture.format(),
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[output.texture.format()],
            });

            let temp_view = temp_texture.create_view(&wgpu::TextureViewDescriptor::default());

            if let Some((model, scene_ctrl, renderer)) = &mut current {
                if !x {
                    // apply_bindings(
                    //     &mut model.puppet,
                    //     &displayed_model.bindings.lock().unwrap(),
                    //     &tracker_system,
                    // );
                    x = true;
                }

                scene_ctrl.update(&mut renderer.camera);
                renderer.render(&queue, &device, &model.puppet, &temp_view);
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Part Render Encoder"),
                });
                encoder.copy_texture_to_texture(temp_texture.as_image_copy(), output.texture.as_image_copy(), output.texture.size());
                queue.submit(std::iter::once(encoder.finish()));
            }

            let desc = wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..wgpu::RenderPassDescriptor::default()
            };

            integration.render(&device, &queue, &desc);
            let res = Instant::now();
            let diff = res - now;
            ma.add_sample(diff.as_secs_f64());
            let fps = 1.0 / ma.get_average();
            println!("{:?}", fps);

            output.present();
        }
        Event::WindowEvent { ref event, .. } => {
            let res = integration.handle_event(event);

            if !res.consumed {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => {
                        // Reconfigure the surface with the new size
                        if size.width > 0 && size.height > 0 {
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);

                            // Update the renderer's internal viewport
                            if let Some((_, _, renderer)) = &mut current {
                                renderer.resize(uvec2(size.width, size.height));
                            }
                            integration.resize(*size);

                            // On macos the window needs to be redrawn manually after resizing
                            window.request_redraw();
                        }
                    }
                    _ => {
                        if let Some((_, scene_ctrl, renderer)) = &mut current {
                            scene_ctrl.interact(&window, event, &renderer.camera)
                        }
                    }
                }
            }

            if res.repaint {
                window.request_redraw();
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}

fn map_value(val: f32, (x1, x2): (f32, f32), (y1, y2): (f32, f32)) -> f32 {
    (val - x1) * (y2 - y1) / (x2 - x1) + y1
}

fn main() {
    pollster::block_on(run());
}

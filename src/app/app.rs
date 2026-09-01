use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::core::event::{AppEvent, EventDispatcher};
use crate::core::state::AppState;
use crate::ui::renderer::GpuContext;
use crate::ui::theme::Theme;

pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    state: AppState,
    dispatcher: EventDispatcher,
    theme: Theme,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            state: AppState::new(),
            dispatcher: EventDispatcher::new(),
            theme: Theme::dark(),
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new()?;

        let window = Arc::new(
            WindowBuilder::new()
                .with_title("Zero Explorer")
                .with_inner_size(winit::dpi::LogicalSize::new(1200, 800))
                .with_min_inner_size(winit::dpi::LogicalSize::new(800, 600))
                .build(&event_loop)?,
        );

        let gpu = GpuContext::new(window.clone())?;
        self.gpu = Some(gpu);
        self.window = Some(window);

        log::info!("Zero Explorer started");

        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            log::info!("Close requested");
                            elwt.exit();
                        }
                        WindowEvent::Resized(size) => {
                            log::debug!("Window resized: {}x{}", size.width, size.height);
                            if let Some(gpu) = &mut self.gpu {
                                gpu.resize(size.width, size.height);
                            }
                            self.dispatcher.dispatch(&AppEvent::WindowResized(
                                size.width,
                                size.height,
                            ));
                        }
                        WindowEvent::Focused(focused) => {
                            self.dispatcher
                                .dispatch(&AppEvent::WindowFocused(focused));
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            if event.state == winit::event::ElementState::Pressed {
                                if let winit::keyboard::PhysicalKey::Code(keycode) =
                                    event.physical_key
                                {
                                    self.dispatcher
                                        .dispatch(&AppEvent::KeyPressed(keycode as u32));
                                }
                            }
                        }
                        WindowEvent::MouseInput {
                            state: _, button, ..
                        } => {
                            let button_value = match button {
                                winit::event::MouseButton::Left => 0,
                                winit::event::MouseButton::Right => 1,
                                winit::event::MouseButton::Middle => 2,
                                winit::event::MouseButton::Back => 3,
                                winit::event::MouseButton::Forward => 4,
                                winit::event::MouseButton::Other(n) => n as u32,
                            };
                            self.dispatcher
                                .dispatch(&AppEvent::MouseButtonPressed(button_value));
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            self.dispatcher.dispatch(&AppEvent::MouseMoved(
                                position.x,
                                position.y,
                            ));
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            let scroll = match delta {
                                winit::event::MouseScrollDelta::LineDelta(_, y) => y as f64,
                                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y,
                            };
                            self.dispatcher.dispatch(&AppEvent::MouseWheel(scroll));
                        }
                        WindowEvent::RedrawRequested => {
                            self.render();
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        })?;

        Ok(())
    }

    fn render(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            if let Some(frame) = gpu.begin_frame() {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = gpu
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Zero Explorer Encoder"),
                    });

                // Clear with background color (dark theme)
                gpu.clear(
                    &mut encoder,
                    &view,
                    wgpu::Color {
                        r: 0.11,
                        g: 0.11,
                        b: 0.11,
                        a: 1.0,
                    },
                );

                let screen_width = gpu.surface_config.width as f32;
                let screen_height = gpu.surface_config.height as f32;

                // Draw sidebar (left panel)
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    0.0,
                    0.0,
                    200.0,
                    screen_height,
                    [0.15, 0.15, 0.15, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw sidebar border
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    200.0,
                    0.0,
                    1.0,
                    screen_height,
                    [0.3, 0.3, 0.3, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw tab bar
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    0.0,
                    screen_width - 201.0,
                    40.0,
                    [0.18, 0.18, 0.18, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw tab bar border
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    40.0,
                    screen_width - 201.0,
                    1.0,
                    [0.3, 0.3, 0.3, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw breadcrumb/address bar
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    41.0,
                    screen_width - 201.0,
                    35.0,
                    [0.13, 0.13, 0.13, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw breadcrumb border
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    76.0,
                    screen_width - 201.0,
                    1.0,
                    [0.3, 0.3, 0.3, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw file list area
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    77.0,
                    screen_width - 201.0,
                    screen_height - 107.0,
                    [0.11, 0.11, 0.11, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw status bar
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    0.0,
                    screen_height - 30.0,
                    screen_width,
                    30.0,
                    [0.15, 0.15, 0.15, 1.0],
                    screen_width,
                    screen_height,
                );

                // Draw status bar border
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    0.0,
                    screen_height - 30.0,
                    screen_width,
                    1.0,
                    [0.3, 0.3, 0.3, 1.0],
                    screen_width,
                    screen_height,
                );

                gpu.queue.submit(std::iter::once(encoder.finish()));
                gpu.end_frame(frame);
            }
        }
    }
}

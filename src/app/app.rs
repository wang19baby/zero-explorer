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
    mouse_x: f32,
    mouse_y: f32,
    hovered_area: HoveredArea,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum HoveredArea {
    None,
    Sidebar,
    TabBar,
    AddressBar,
    FileList,
    StatusBar,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            state: AppState::new(),
            dispatcher: EventDispatcher::new(),
            theme: Theme::dark(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            hovered_area: HoveredArea::None,
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
                            self.mouse_x = position.x as f32;
                            self.mouse_y = position.y as f32;
                            self.update_hovered_area();
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

    fn update_hovered_area(&mut self) {
        let _screen_width = self
            .gpu
            .as_ref()
            .map(|g| g.surface_config.width as f32)
            .unwrap_or(1200.0);
        let screen_height = self
            .gpu
            .as_ref()
            .map(|g| g.surface_config.height as f32)
            .unwrap_or(800.0);

        self.hovered_area = if self.mouse_x < 200.0 {
            HoveredArea::Sidebar
        } else if self.mouse_y < 40.0 {
            HoveredArea::TabBar
        } else if self.mouse_y < 76.0 {
            HoveredArea::AddressBar
        } else if self.mouse_y < screen_height - 30.0 {
            HoveredArea::FileList
        } else {
            HoveredArea::StatusBar
        };
    }

    fn get_area_color_static(hovered: HoveredArea, area: HoveredArea, base_color: [f32; 4]) -> [f32; 4] {
        if area == hovered {
            [
                base_color[0] + 0.05,
                base_color[1] + 0.05,
                base_color[2] + 0.05,
                base_color[3],
            ]
        } else {
            base_color
        }
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
                let hovered_area = self.hovered_area;

                // Draw sidebar (left panel) with hover effect
                let sidebar_color = Self::get_area_color_static(hovered_area, HoveredArea::Sidebar, [0.15, 0.15, 0.15, 1.0]);
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    0.0,
                    0.0,
                    200.0,
                    screen_height,
                    sidebar_color,
                    screen_width,
                    screen_height,
                );

                // Draw sidebar items with text
                let sidebar_items = [
                    ("This PC", true),
                    ("Desktop", false),
                    ("Documents", false),
                    ("Downloads", false),
                    ("Pictures", false),
                ];

                for (i, (name, is_expanded)) in sidebar_items.iter().enumerate() {
                    let y = 50.0 + i as f32 * 35.0;
                    let item_color = if self.mouse_y >= y && self.mouse_y < y + 35.0 && self.mouse_x < 200.0 {
                        [0.25, 0.25, 0.25, 1.0]
                    } else {
                        [0.18, 0.18, 0.18, 1.0]
                    };

                    gpu.draw_rect(
                        &mut encoder,
                        &view,
                        10.0,
                        y,
                        180.0,
                        30.0,
                        item_color,
                        screen_width,
                        screen_height,
                    );

                    // Draw expand indicator
                    if *is_expanded {
                        gpu.draw_rect(
                            &mut encoder,
                            &view,
                            15.0,
                            y + 10.0,
                            10.0,
                            10.0,
                            [0.4, 0.7, 1.0, 1.0],
                            screen_width,
                            screen_height,
                        );
                    }

                    // Draw text
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        name,
                        30.0,
                        y + 8.0,
                        [0.9, 0.9, 0.9, 1.0],
                        screen_width,
                        screen_height,
                    );
                }

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

                // Draw tab bar with hover effect
                let tab_color = Self::get_area_color_static(hovered_area, HoveredArea::TabBar, [0.18, 0.18, 0.18, 1.0]);
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    0.0,
                    screen_width - 201.0,
                    40.0,
                    tab_color,
                    screen_width,
                    screen_height,
                );

                // Draw tab items with text
                let tabs = ["Home", "Documents", "Downloads"];
                let mut tab_x = 210.0;
                for (i, tab_name) in tabs.iter().enumerate() {
                    let tab_width = 100.0;
                    let is_active = i == 0;
                    let tab_item_color = if is_active {
                        [0.22, 0.22, 0.22, 1.0]
                    } else {
                        [0.16, 0.16, 0.16, 1.0]
                    };

                    gpu.draw_rect(
                        &mut encoder,
                        &view,
                        tab_x,
                        5.0,
                        tab_width,
                        30.0,
                        tab_item_color,
                        screen_width,
                        screen_height,
                    );

                    // Active tab indicator
                    if is_active {
                        gpu.draw_rect(
                            &mut encoder,
                            &view,
                            tab_x,
                            35.0,
                            tab_width,
                            3.0,
                            [0.4, 0.7, 1.0, 1.0],
                            screen_width,
                            screen_height,
                        );
                    }

                    // Draw tab text
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        tab_name,
                        tab_x + 10.0,
                        15.0,
                        [0.9, 0.9, 0.9, 1.0],
                        screen_width,
                        screen_height,
                    );

                    tab_x += tab_width + 5.0;
                }

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

                // Draw breadcrumb/address bar with hover effect
                let address_color = Self::get_area_color_static(hovered_area, HoveredArea::AddressBar, [0.13, 0.13, 0.13, 1.0]);
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    41.0,
                    screen_width - 201.0,
                    35.0,
                    address_color,
                    screen_width,
                    screen_height,
                );

                // Draw breadcrumb segments with text
                let segments = ["This PC", "Documents", "Projects"];
                let mut seg_x = 215.0;
                for (i, segment) in segments.iter().enumerate() {
                    // Segment background
                    gpu.draw_rect(
                        &mut encoder,
                        &view,
                        seg_x,
                        48.0,
                        80.0,
                        20.0,
                        [0.2, 0.2, 0.2, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Draw segment text
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        segment,
                        seg_x + 5.0,
                        53.0,
                        [0.9, 0.9, 0.9, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Separator
                    if i < segments.len() - 1 {
                        gpu.draw_rect(
                            &mut encoder,
                            &view,
                            seg_x + 85.0,
                            53.0,
                            2.0,
                            10.0,
                            [0.4, 0.4, 0.4, 1.0],
                            screen_width,
                            screen_height,
                        );
                    }

                    seg_x += 95.0;
                }

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

                // Draw file list area with hover effect
                let file_color = Self::get_area_color_static(hovered_area, HoveredArea::FileList, [0.11, 0.11, 0.11, 1.0]);
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    201.0,
                    77.0,
                    screen_width - 201.0,
                    screen_height - 107.0,
                    file_color,
                    screen_width,
                    screen_height,
                );

                // Draw column headers with text
                let headers = [
                    ("Name", 215.0, 200.0),
                    ("Date", 420.0, 120.0),
                    ("Type", 545.0, 100.0),
                    ("Size", 650.0, 80.0),
                ];

                for (header_name, x, width) in headers.iter() {
                    gpu.draw_rect(
                        &mut encoder,
                        &view,
                        *x,
                        82.0,
                        *width,
                        25.0,
                        [0.16, 0.16, 0.16, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Draw header text
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        header_name,
                        *x + 10.0,
                        88.0,
                        [0.7, 0.7, 0.7, 1.0],
                        screen_width,
                        screen_height,
                    );
                }

                // Draw file items with text
                let files = [
                    ("Project_A", "2024-01-15", "Folder", "4.2 GB"),
                    ("Document.pdf", "2024-01-14", "PDF", "2.3 MB"),
                    ("Image.png", "2024-01-13", "PNG", "1.5 MB"),
                    ("Video.mp4", "2024-01-12", "MP4", "250 MB"),
                    ("Archive.zip", "2024-01-11", "ZIP", "45 MB"),
                ];

                for (i, (name, date, file_type, size)) in files.iter().enumerate() {
                    let y = 115.0 + i as f32 * 35.0;
                    let is_selected = i == 0;
                    let is_hovered = self.mouse_y >= y
                        && self.mouse_y < y + 35.0
                        && self.mouse_x >= 201.0
                        && self.mouse_x < screen_width;

                    let file_item_color = if is_selected {
                        [0.25, 0.35, 0.5, 1.0]
                    } else if is_hovered {
                        [0.18, 0.18, 0.18, 1.0]
                    } else {
                        [0.13, 0.13, 0.13, 1.0]
                    };

                    gpu.draw_rect(
                        &mut encoder,
                        &view,
                        215.0,
                        y,
                        screen_width - 230.0,
                        30.0,
                        file_item_color,
                        screen_width,
                        screen_height,
                    );

                    // Draw file name
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        name,
                        225.0,
                        y + 8.0,
                        [0.9, 0.9, 0.9, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Draw date
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        date,
                        430.0,
                        y + 8.0,
                        [0.7, 0.7, 0.7, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Draw type
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        file_type,
                        555.0,
                        y + 8.0,
                        [0.7, 0.7, 0.7, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Draw size
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        size,
                        660.0,
                        y + 8.0,
                        [0.7, 0.7, 0.7, 1.0],
                        screen_width,
                        screen_height,
                    );
                }

                // Draw status bar with hover effect
                let status_color = Self::get_area_color_static(hovered_area, HoveredArea::StatusBar, [0.15, 0.15, 0.15, 1.0]);
                gpu.draw_rect(
                    &mut encoder,
                    &view,
                    0.0,
                    screen_height - 30.0,
                    screen_width,
                    30.0,
                    status_color,
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

                // Draw status bar sections with text
                let status_sections = [
                    ("5 items", 10.0, 80.0),
                    ("1 selected", 100.0, 100.0),
                    ("Layout: Single", screen_width - 150.0, 140.0),
                ];

                for (text, x, width) in status_sections.iter() {
                    gpu.draw_rect(
                        &mut encoder,
                        &view,
                        *x,
                        screen_height - 25.0,
                        *width,
                        20.0,
                        [0.2, 0.2, 0.2, 1.0],
                        screen_width,
                        screen_height,
                    );

                    // Draw status text
                    gpu.draw_text(
                        &mut encoder,
                        &view,
                        text,
                        *x + 5.0,
                        screen_height - 20.0,
                        [0.7, 0.7, 0.7, 1.0],
                        screen_width,
                        screen_height,
                    );
                }

                gpu.queue.submit(std::iter::once(encoder.finish()));
                gpu.end_frame(frame);
            }
        }
    }
}

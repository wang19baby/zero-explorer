use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::core::event::EventDispatcher;
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
    fn text_y_centered(gpu: &GpuContext, container_y: f32, container_h: f32) -> f32 {
        let lh = gpu.line_height();
        let asc = gpu.ascent();
        container_y + (container_h - lh) / 2.0 + asc
    }

    pub fn new() -> Self {
        Self {
            window: None,
            gpu: None,
            state: AppState::new(),
            dispatcher: EventDispatcher::new(),
            theme: Theme::light(),
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

        self.window = Some(window.clone());
        self.gpu = Some(gpu);

        event_loop.run(move |event, target| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::Resized(size) => {
                        if let Some(gpu) = &mut self.gpu {
                            gpu.resize(size.width, size.height);
                        }
                        self.window.as_ref().unwrap().request_redraw();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.mouse_x = position.x as f32;
                        self.mouse_y = position.y as f32;
                        self.update_hovered_area();
                        self.window.as_ref().unwrap().request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        self.render();
                    }
                    _ => {}
                },
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
        let _w = self.window.as_ref().unwrap().inner_size().width as f32;
        let h = self.window.as_ref().unwrap().inner_size().height as f32;
        let sidebar_w = 200.0f32;
        let breadcrumb_h = 36.0f32;
        let tab_h = 32.0f32;
        let status_h = 30.0f32;

        self.hovered_area = if self.mouse_x < sidebar_w {
            HoveredArea::Sidebar
        } else if self.mouse_y < breadcrumb_h {
            HoveredArea::AddressBar
        } else if self.mouse_y < breadcrumb_h + tab_h {
            HoveredArea::TabBar
        } else if self.mouse_y > h - status_h {
            HoveredArea::StatusBar
        } else {
            HoveredArea::FileList
        };
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

                // Win11 light theme colors
                let bg_base: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
                let bg_secondary: [f32; 4] = [0.976, 0.976, 0.976, 1.0];
                let bg_tertiary: [f32; 4] = [0.953, 0.953, 0.953, 1.0];
                let border: [f32; 4] = [0.898, 0.898, 0.898, 1.0];
                let text_primary: [f32; 4] = [0.102, 0.102, 0.102, 1.0];
                let text_secondary: [f32; 4] = [0.380, 0.380, 0.380, 1.0];
                let text_tertiary: [f32; 4] = [0.620, 0.620, 0.620, 1.0];
                let primary: [f32; 4] = [0.0, 0.471, 0.831, 1.0];
                let primary_light: [f32; 4] = [0.910, 0.957, 0.992, 1.0];

                gpu.clear(&mut encoder, &view, wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });

                let sw = gpu.surface_config.width as f32;
                let sh = gpu.surface_config.height as f32;
                let sidebar_w = 200.0f32;
                let breadcrumb_h = 36.0f32;
                let tab_h = 32.0f32;
                let status_h = 30.0f32;
                let row_h = 28.0f32;
                let header_h = 28.0f32;

                // === SIDEBAR ===
                gpu.draw_rect(&mut encoder, &view, 0.0, 0.0, sidebar_w, sh, bg_secondary, sw, sh);
                gpu.draw_rect(&mut encoder, &view, sidebar_w, 0.0, 1.0, sh, border, sw, sh);

                let mut sy = 12.0f32;

                // Section: 此电脑
                gpu.draw_text(&mut encoder, &view, "此电脑", 12.0, Self::text_y_centered(gpu, sy, 20.0), text_secondary, sw, sh);
                sy += 24.0;

                let disks = [
                    ("本地磁盘 (C:)", 0.65f32, "120/186 GB"),
                    ("工作磁盘 (D:)", 0.42f32, "168/400 GB"),
                    ("数据磁盘 (E:)", 0.78f32, "312/400 GB"),
                ];
                for (name, pct, info) in disks.iter() {
                    gpu.draw_text(&mut encoder, &view, name, 12.0, Self::text_y_centered(gpu, sy, row_h), text_primary, sw, sh);
                    sy += row_h;
                    gpu.draw_rect(&mut encoder, &view, 12.0, sy + 6.0, sidebar_w - 24.0, 4.0, bg_tertiary, sw, sh);
                    gpu.draw_rect(&mut encoder, &view, 12.0, sy + 6.0, (sidebar_w - 24.0) * *pct, 4.0, primary, sw, sh);
                    gpu.draw_text(&mut encoder, &view, info, 12.0, Self::text_y_centered(gpu, sy + 12.0, 16.0), text_tertiary, sw, sh);
                    sy += 28.0;
                }

                sy += 8.0;
                // Section: 标签
                gpu.draw_text(&mut encoder, &view, "标签", 12.0, Self::text_y_centered(gpu, sy, 20.0), text_secondary, sw, sh);
                sy += 24.0;
                let bookmarks = ["D:\\work_space", "E:\\backup", "D:\\projects"];
                for bm in bookmarks.iter() {
                    let item_color = if self.mouse_y >= sy && self.mouse_y < sy + row_h && self.mouse_x < sidebar_w {
                        bg_tertiary
                    } else {
                        bg_secondary
                    };
                    gpu.draw_rect(&mut encoder, &view, 4.0, sy, sidebar_w - 8.0, row_h, item_color, sw, sh);
                    gpu.draw_text(&mut encoder, &view, bm, 12.0, Self::text_y_centered(gpu, sy, row_h), text_primary, sw, sh);
                    sy += row_h;
                }
                gpu.draw_text(&mut encoder, &view, "+ 添加文件夹", 12.0, Self::text_y_centered(gpu, sy, row_h), primary, sw, sh);
                sy += row_h + 8.0;

                // Section: 最近访问
                gpu.draw_text(&mut encoder, &view, "最近访问", 12.0, Self::text_y_centered(gpu, sy, 20.0), text_secondary, sw, sh);
                sy += 24.0;
                let recents = [
                    ("main.rs", "2分钟前"),
                    ("README.md", "1小时前"),
                    ("logo.png", "昨天"),
                    ("report.pdf", "3天前"),
                ];
                for (name, time) in recents.iter() {
                    gpu.draw_text(&mut encoder, &view, name, 12.0, Self::text_y_centered(gpu, sy, row_h), text_primary, sw, sh);
                    gpu.draw_text(&mut encoder, &view, time, 120.0, Self::text_y_centered(gpu, sy, row_h), text_tertiary, sw, sh);
                    sy += row_h;
                }

                // === MAIN CONTENT AREA ===
                let main_x = sidebar_w + 1.0;
                let main_w = sw - sidebar_w - 1.0;

                // Breadcrumb bar
                gpu.draw_rect(&mut encoder, &view, main_x, 0.0, main_w, breadcrumb_h, bg_base, sw, sh);
                gpu.draw_rect(&mut encoder, &view, main_x, breadcrumb_h - 1.0, main_w, 1.0, border, sw, sh);

                let crumbs = ["D:", "work_space", "personal_workspace", "src"];
                let mut cx = main_x + 12.0;
                for (i, crumb) in crumbs.iter().enumerate() {
                    let color = if i == crumbs.len() - 1 { text_primary } else { text_secondary };
                    gpu.draw_text(&mut encoder, &view, crumb, cx, Self::text_y_centered(gpu, 0.0, breadcrumb_h), color, sw, sh);
                    cx += gpu.measure_text(crumb) + 4.0;
                    if i < crumbs.len() - 1 {
                        gpu.draw_text(&mut encoder, &view, "›", cx, Self::text_y_centered(gpu, 0.0, breadcrumb_h), text_tertiary, sw, sh);
                        cx += gpu.measure_text("›") + 8.0;
                    }
                }

                // Tab bar
                let tab_y = breadcrumb_h;
                gpu.draw_rect(&mut encoder, &view, main_x, tab_y, main_w, tab_h, bg_tertiary, sw, sh);
                gpu.draw_rect(&mut encoder, &view, main_x, tab_y + tab_h - 1.0, main_w, 1.0, border, sw, sh);

                let tabs = ["src", "docs"];
                let mut tx = main_x + 4.0;
                for (i, tab_name) in tabs.iter().enumerate() {
                    let tab_w = gpu.measure_text(tab_name) + 24.0;
                    let tab_color = if i == 0 { bg_base } else { bg_tertiary };
                    gpu.draw_rect(&mut encoder, &view, tx, tab_y + 2.0, tab_w, tab_h - 2.0, tab_color, sw, sh);
                    if i == 0 {
                        gpu.draw_rect(&mut encoder, &view, tx, tab_y + 2.0, tab_w, 2.0, primary, sw, sh);
                    }
                    gpu.draw_text(&mut encoder, &view, tab_name, tx + 12.0, Self::text_y_centered(gpu, tab_y + 2.0, tab_h - 2.0), text_primary, sw, sh);
                    tx += tab_w + 1.0;
                }

                // File list area
                let list_y = tab_y + tab_h;
                let list_h = sh - list_y - status_h;
                gpu.draw_rect(&mut encoder, &view, main_x, list_y, main_w, list_h, bg_base, sw, sh);

                // Column header
                let header_y = list_y;
                gpu.draw_rect(&mut encoder, &view, main_x, header_y, main_w, header_h, bg_tertiary, sw, sh);
                gpu.draw_rect(&mut encoder, &view, main_x, header_y + header_h - 1.0, main_w, 1.0, border, sw, sh);

                let col_name = main_x + 36.0;
                let col_type = main_x + main_w * 0.45;
                let col_size = main_x + main_w * 0.7;
                let col_date = main_x + main_w * 0.82;

                gpu.draw_text(&mut encoder, &view, "名称", col_name, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);
                gpu.draw_text(&mut encoder, &view, "类型", col_type, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);
                gpu.draw_text(&mut encoder, &view, "大小", col_size, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);
                gpu.draw_text(&mut encoder, &view, "修改时间", col_date, Self::text_y_centered(gpu, header_y, header_h), text_secondary, sw, sh);

                // File rows
                let files = [
                    ("📁", "components", "文件夹", "", "2026-08-31"),
                    ("📁", "utils", "文件夹", "", "2026-08-30"),
                    ("🦀", "main.rs", "Rust 源代码", "4.2 KB", "2026-08-29"),
                    ("🦀", "lib.rs", "Rust 源代码", "1.8 KB", "2026-08-28"),
                    ("📝", "README.md", "Markdown", "2.4 KB", "2026-08-27"),
                    ("🖼️", "logo.png", "PNG 图片", "128 KB", "2026-08-26"),
                    ("📕", "report.pdf", "PDF 文档", "5.8 MB", "2026-08-25"),
                ];

                for (i, (icon, name, ftype, size, date)) in files.iter().enumerate() {
                    let ry = header_y + header_h + i as f32 * row_h;
                    if ry + row_h > sh - status_h { break; }

                    let is_selected = i == 2;
                    let row_color = if is_selected {
                        primary
                    } else if self.mouse_y >= ry && self.mouse_y < ry + row_h && self.mouse_x > sidebar_w {
                        primary_light
                    } else {
                        bg_base
                    };
                    let text_color = if is_selected { [1.0, 1.0, 1.0, 1.0] } else { text_primary };
                    let sub_color = if is_selected { [0.8, 0.9, 1.0, 1.0] } else { text_secondary };

                    gpu.draw_rect(&mut encoder, &view, main_x, ry, main_w, row_h, row_color, sw, sh);
                    gpu.draw_rect(&mut encoder, &view, main_x, ry + row_h - 1.0, main_w, 1.0, border, sw, sh);

                    gpu.draw_text(&mut encoder, &view, icon, main_x + 8.0, Self::text_y_centered(gpu, ry, row_h), text_color, sw, sh);
                    gpu.draw_text(&mut encoder, &view, name, col_name, Self::text_y_centered(gpu, ry, row_h), text_color, sw, sh);
                    gpu.draw_text(&mut encoder, &view, ftype, col_type, Self::text_y_centered(gpu, ry, row_h), sub_color, sw, sh);
                    if !size.is_empty() {
                        gpu.draw_text(&mut encoder, &view, size, col_size, Self::text_y_centered(gpu, ry, row_h), sub_color, sw, sh);
                    }
                    gpu.draw_text(&mut encoder, &view, date, col_date, Self::text_y_centered(gpu, ry, row_h), sub_color, sw, sh);
                }

                // === STATUS BAR ===
                let status_y = sh - status_h;
                gpu.draw_rect(&mut encoder, &view, 0.0, status_y, sw, status_h, bg_secondary, sw, sh);
                gpu.draw_rect(&mut encoder, &view, 0.0, status_y, sw, 1.0, border, sw, sh);

                // Vim mode indicator
                gpu.draw_rect(&mut encoder, &view, 12.0, status_y + 8.0, 52.0, 14.0, text_primary, sw, sh);
                gpu.draw_text(&mut encoder, &view, "NORMAL", 14.0, Self::text_y_centered(gpu, status_y + 8.0, 14.0), [1.0, 1.0, 1.0, 1.0], sw, sh);

                // Path
                gpu.draw_text(&mut encoder, &view, "D:\\work_space\\personal_workspace\\src", 72.0, Self::text_y_centered(gpu, status_y, status_h), text_secondary, sw, sh);

                // File count
                gpu.draw_text(&mut encoder, &view, "7 个项目", sw - 80.0, Self::text_y_centered(gpu, status_y, status_h), text_secondary, sw, sh);

                gpu.queue.submit(std::iter::once(encoder.finish()));
                gpu.end_frame(frame);
            }
        }
    }
}

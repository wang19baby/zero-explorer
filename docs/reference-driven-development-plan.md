# Zero Explorer - 参考驱动开发计划

**版本**: v3.0  
**日期**: 2026-09-01  
**原则**: 每个功能标注参考来源，开发时直接参考竞品实现

---

## 一、技术栈对齐

### 1.1 核心依赖 (参考 MTT File Manager + FileMan)

```toml
[dependencies]
# UI框架 - 参考 MTT File Manager
eframe = { version = "0.35", features = ["persistence", "wgpu", "glow"] }

# GPU渲染 - 参考 MTT File Manager
wgpu = { version = "29.0", default-features = false, features = ["std", "dx12"] }

# 异步运行时
tokio = { version = "1.41", features = ["rt-multi-thread", "sync", "time", "fs"] }

# 文件系统 - 参考 MTT File Manager
walkdir = "2.5"
notify = "6.1"

# 缓存 - 参考 MTT File Manager
lru = "0.16"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 图像处理 - 参考 MTT File Manager
image = { version = "0.25", features = ["webp", "gif"] }

# 语法高亮 - 参考 FileMan
syntect = { version = "5", default-features = false }

# 归档支持 - 参考 MTT File Manager
zip = "2"
tar = "0.4"
flate2 = "1.0"
sevenz-rust = "0.6"

# Windows集成 - 参考 MTT File Manager
windows = { version = "0.62", features = [...] }
```

### 1.2 渲染方案选择

| 方案 | 参考项目 | 选择理由 |
|-----|---------|---------|
| **egui + wgpu** | MTT File Manager | 生态成熟，组件丰富，GPU加速 |
| **blade-egui** | FileMan | 轻量级，但生态较小 |
| **GPUI** | Nexus Explorer | 文档差，组件少，不适合 |

**结论**: 选择 **egui + wgpu**，参考 MTT File Manager 的实现

---

## 二、布局结构 (1:1复现 design-preview.html)

**参考**: `wireframes/design-preview.html` (单一事实来源)

### 2.1 主布局结构 (full-layout)

```
┌─────────────────────────────────────────────────────────────────────────┐
│  .full-layout (display: flex, flex: 1, overflow: hidden)              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────┬─┬──────────────────────────────────────────────────┐  │
│  │  .layout-   │ │  .layout-main (flex: 1, display: flex)          │  │
│  │  sidebar    │ │                                                  │  │
│  │  (width:    │ │  ┌──────────────┬──────────────┬──────────────┐ │  │
│  │   var(--    │ │  │ .wireframe-  │ .wireframe-  │ .wireframe-  │ │  │
│  │   sidebar-  │ │  │ panel        │ panel        │ panel        │ │  │
│  │   width)    │ │  │ (flex: 1)    │ (flex: 1)    │ (flex: 1)    │ │  │
│  │             │ │  │              │              │              │ │  │
│  │  默认隐藏   │ │  │  面板内部:    │  面板内部:    │  面板内部:    │ │  │
│  │  Ctrl+Shift │ │  │  ┌────────┐ │  ┌────────┐ │  ┌────────┐ │ │  │
│  │  +B 切换    │ │  │  │面包屑   │ │  │面包屑   │ │  │面包屑   │ │ │  │
│  │             │ │  │  ├────────┤ │  ├────────┤ │  ├────────┤ │ │  │
│  │             │ │  │  │Tab栏   │ │  │Tab栏   │ │  │Tab栏   │ │ │  │
│  │             │ │  │  ├────────┤ │  ├────────┤ │  ├────────┤ │ │  │
│  │             │ │  │  │文件列表│ │  │文件列表│ │  │文件列表│ │ │  │
│  │             │ │  │  ├────────┤ │  ├────────┤ │  ├────────┤ │ │  │
│  │             │ │  │  │面板状态│ │  │面板状态│ │  │面板状态│ │ │  │
│  └─────────────┴─┴──┴──┴────────┴─┴──┴────────┴─┴──┴────────┴─┴─┘  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  .layout-statusbar (height: 32px)                              │  │
│  │  [模式切换] | [Space管理] | [面板信息] | [路径] | [布局切换]     │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2.2 侧边栏结构 (.layout-sidebar)

```
┌─────────────────────────────────────┐
│  .layout-sidebar                    │
│  width: var(--sidebar-width)        │
│  默认: hidden (display: none)       │
│  显示: display: flex                │
├─────────────────────────────────────┤
│  .sidebar                           │
│  ├─ .sidebar-section                │
│  │  ├─ .sidebar-title: 此电脑 ▾     │
│  │  ├─ .sidebar-item: 💾 本地磁盘   │
│  │  ├─ .sidebar-disk-bar            │
│  │  │  ├─ .disk-bar                 │
│  │  │  │  └─ .disk-used (width: X%) │
│  │  │  └─ .disk-text: 65% · 120GB  │
│  │  ├─ .sidebar-item: 💾 工作磁盘   │
│  │  └─ .sidebar-item: 💾 数据磁盘   │
│  │                                  │
│  ├─ .sidebar-section                │
│  │  ├─ .sidebar-title: 标签         │
│  │  ├─ .sidebar-item: 📁 D:\work   │
│  │  ├─ .sidebar-item: 📁 E:\backup │
│  │  └─ .sidebar-item.add-folder     │
│  │                                  │
│  ├─ .sidebar-section                │
│  │  ├─ .sidebar-title: 最近访问     │
│  │  ├─ .sidebar-item: 📄 main.rs    │
│  │  └─ .sidebar-item: 📝 README.md │
│  │                                  │
│  └─ .sidebar-section.sidebar-space  │
│     ├─ .sidebar-title: 空间         │
│     ├─ .sidebar-item.active: 🏠默认 │
│     ├─ .sidebar-item: 💼 Work       │
│     ├─ .sidebar-item: 💻 Dev        │
│     └─ .sidebar-item: + 新建空间    │
└─────────────────────────────────────┘

侧边栏调整大小:
┌─────────────────────────────────────┐
│  .sidebar-resize-handle             │
│  width: 4px                         │
│  cursor: col-resize                 │
│  拖拽范围: 150px - 400px            │
│  默认宽度: 200px                    │
│  双击重置: 200px                    │
└─────────────────────────────────────┘
```

### 2.3 面板内部结构 (.wireframe-panel)

```
┌─────────────────────────────────────┐
│  .wireframe-panel (flex: 1)         │
├─────────────────────────────────────┤
│  .panel-breadcrumb-wrapper          │
│  └─ .panel-breadcrumb               │
│     ├─ .breadcrumb                  │
│     │  ├─ .breadcrumb-item: D:      │
│     │  ├─ .breadcrumb-sep: ›        │
│     │  ├─ .breadcrumb-item: work    │
│     │  └─ .breadcrumb-item.current  │
│     └─ .breadcrumb-input (hidden)   │
│        └─ placeholder: 输入路径...  │
├─────────────────────────────────────┤
│  .panel-tabs                        │
│  ├─ .panel-tab.active: src          │
│  ├─ .panel-tab: docs                │
│  ├─ .panel-tabs-spacer (flex: 1)    │
│  └─ .panel-view-toggle              │
│     ├─ .view-toggle-btn.active: 列表│
│     └─ .view-toggle-btn: 树形       │
├─────────────────────────────────────┤
│  .panel-content                     │
│  └─ .file-list                      │
│     ├─ .file-list-header            │
│     │  ├─ (空) | 名称 | 类型 | 大小 | 修改时间 │
│     ├─ .file-list-row               │
│     │  ├─ .file-icon: 📁            │
│     │  ├─ .file-name: components    │
│     │  ├─ .file-type: 文件夹        │
│     │  ├─ .file-size: (空)          │
│     │  └─ .file-date: 2026-08-31   │
│     └─ .file-list-row.selected      │
│        └─ (选中样式)                 │
├─────────────────────────────────────┤
│  .panel-statusbar                   │
│  ├─ span: 5 个项目                  │
│  └─ span: 1 个选中                  │
└─────────────────────────────────────┘
```

### 2.4 状态栏结构 (.layout-statusbar)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  .layout-statusbar (height: 32px)                                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  .statusbar-left (flex: 1, display: flex, align-items: center)            │
│  ├─ .mode-toggle (display: flex)                                          │
│  │  ├─ .mode-btn.active: 多面板模式 (SVG图标)                              │
│  │  └─ .mode-btn: 级联分栏模式 (SVG图标)                                  │
│  │                                                                          │
│  ├─ span: | (分隔符)                                                       │
│  │                                                                          │
│  ├─ .space-toggle (display: flex)                                          │
│  │  ├─ .space-btn.active: 🏠 默认                                          │
│  │  ├─ .space-btn: 💼 Work                                                 │
│  │  ├─ .space-btn: 💻 Dev                                                  │
│  │  └─ .space-add: + (管理空间)                                            │
│  │                                                                          │
│  ├─ .space-dropdown (hidden, position: absolute)                           │
│  │  ├─ .space-dropdown-header: 空间管理 ×                                   │
│  │  ├─ .space-dropdown-item.active: 🏠 默认                                │
│  │  ├─ .space-dropdown-item: 💼 Work ×                                     │
│  │  ├─ .space-dropdown-item: 💻 Dev ×                                      │
│  │  └─ .space-dropdown-footer: + 新建空间                                   │
│  │                                                                          │
│  ├─ span: | (分隔符)                                                       │
│  ├─ span: 4 个面板 · 1 个选中                                              │
│  ├─ span: | (分隔符)                                                       │
│  ├─ span.statusbar-path: D:\work_space\personal_workspace\zero-explorer   │
│  └─ span#copyToast: 已复制! (hidden)                                      │
│                                                                             │
│  右侧区域 (display: flex, align-items: center)                            │
│  ├─ .layout-toggle (display: flex)                                         │
│  │  ├─ .layout-btn.active: 1分栏 (layout-1x1)                             │
│  │  ├─ .layout-btn: 2左右分栏 (layout-1x2)                                │
│  │  ├─ .layout-btn: 上下分栏 (layout-1x2-v)                               │
│  │  ├─ .layout-btn: 3左中右 (layout-1x3)                                  │
│  │  ├─ .layout-btn: 上2下1 (layout-top2-bottom1)                          │
│  │  ├─ .layout-btn: 上1下2 (layout-top1-bottom2)                          │
│  │  └─ .layout-btn: 4分栏 (layout-2x2)                                    │
│  │                                                                          │
│  └─ span#layoutSeparator: | (分隔符)                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.5 布局模式 (CSS Grid)

**参考**: `wireframes/design-preview.html` CSS Grid 定义

| 模式 | data-layout | CSS类 | Grid模板 | 面板分布 |
|------|-------------|-------|----------|----------|
| 1分栏 | 1 | layout-1x1 | 1列1行 | [1] |
| 2左右分栏 | 2 | layout-1x2 | 2列1行 | [1][2] |
| 上下分栏 | 7 | layout-1x2-v | 1列2行 | [1] / [2] |
| 3左中右 | 3 | layout-1x3 | 3列1行 | [1][2][3] |
| 上2下1 | 4 | layout-top2-bottom1 | 2列2行 | [1][2] / [1:-1 3] |
| 上1下2 | 5 | layout-top1-bottom2 | 2列2行 | [1:-1 1] / [2][3] |
| 4分栏 | 6 | layout-2x2 | 2列2行 | [1][2] / [3][4] |

**CSS Grid定义** (来自 design-preview.html):
```css
/* 1分栏 */
.layout-1x1 { display: grid; grid-template-columns: 1fr; grid-template-rows: 1fr; }

/* 2左右分栏 */
.layout-1x2 { display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: 1fr; }

/* 上下分栏 */
.layout-1x2-v { display: grid; grid-template-columns: 1fr; grid-template-rows: 1fr 1fr; }

/* 3左中右 */
.layout-1x3 { display: grid; grid-template-columns: 1fr 1fr 1fr; grid-template-rows: 1fr; }

/* 上2下1 */
.layout-top2-bottom1 { 
    display: grid; 
    grid-template-columns: 1fr 1fr; 
    grid-template-rows: 1fr 1fr; 
}
/* 面板1: grid-column: 1; grid-row: 1; */
/* 面板2: grid-column: 2; grid-row: 1; */
/* 面板3: grid-column: 1 / -1; grid-row: 2; */

/* 上1下2 */
.layout-top1-bottom2 { 
    display: grid; 
    grid-template-columns: 1fr 1fr; 
    grid-template-rows: 1fr 1fr; 
}
/* 面板1: grid-column: 1 / -1; grid-row: 1; */
/* 面板2: grid-column: 1; grid-row: 2; */
/* 面板3: grid-column: 2; grid-row: 2; */

/* 4分栏 */
.layout-2x2 { 
    display: grid; 
    grid-template-columns: 1fr 1fr; 
    grid-template-rows: 1fr 1fr; 
}
```

### 2.6 级联面板模式 (.cascade-container)

```
┌─────────────────────────────────────┐
│  .cascade-container (hidden)        │
│  display: none                      │
│  显示时: display: flex              │
├─────────────────────────────────────┤
│  .cascade-column                    │
│  ├─ .cascade-header: 根目录         │
│  └─ .cascade-list                   │
│     ├─ .cascade-item.folder: 📁 D:  │
│     ├─ .cascade-item.folder: 📁 E:  │
│     └─ .cascade-item.folder: 📁 C:  │
├─────────────────────────────────────┤
│  .cascade-column                    │
│  ├─ .cascade-header: D:            │
│  └─ .cascade-list                   │
│     ├─ .cascade-item.folder: 📁 work│
│     └─ .cascade-item.folder: 📁 back│
├─────────────────────────────────────┤
│  .cascade-column                    │
│  ├─ .cascade-header: work_space    │
│  └─ .cascade-list                   │
│     └─ .cascade-item.folder.selected│
├─────────────────────────────────────┤
│  .cascade-column                    │
│  ├─ .cascade-header: personal_work │
│  └─ .cascade-list                   │
│     └─ .cascade-item.folder: 📁 zero│
└─────────────────────────────────────┘
```

### 2.7 关键交互逻辑

| 交互 | 触发方式 | 实现逻辑 |
|------|---------|---------|
| 侧边栏显示/隐藏 | Ctrl+Shift+B | 切换 .layout-sidebar 的 display |
| 侧边栏位置 | 右键菜单 | 切换 sidebar 在 main 的左侧/右侧 |
| 侧边栏宽度拖拽 | 鼠标拖拽 | 更新 --sidebar-width CSS变量 |
| 侧边栏双击重置 | 双击resize handle | 重置为200px |
| 面包屑跳转 | 单击 | 导航到对应目录 |
| 面包屑输入 | 双击 | 切换为绝对路径输入框 |
| Tab关闭 | 双击 | 关闭该Tab |
| Tab添加 | 点击面板空白处 | 新建Tab |
| 视图切换 | 点击视图按钮 | 切换列表/树形视图 |
| 布局切换 | 点击状态栏布局按钮 | 切换面板布局 |
| Space切换 | 点击状态栏Space按钮 | 切换工作空间 |
| 模式切换 | 点击状态栏模式按钮 | 切换多面板/级联模式 |

---

## 三、功能开发计划 (按参考来源标注)

### Phase 0: 核心框架 (8周)

#### Week 1: 项目初始化
**参考**: MTT File Manager (`src/main.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| Cargo项目初始化 | MTT `Cargo.toml` | 复制依赖配置 |
| 项目结构 | MTT `src/` | 参考模块划分 |
| CI/CD搭建 | MTT `.github/` | 参考构建流程 |

#### Week 2-3: GPU渲染管线
**参考**: MTT File Manager (`src/app/mod.rs` + `gpu_backend.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| wgpu初始化 | MTT `gpu_backend.rs` | 多后端降级策略 |
| 基础Shader | MTT 着色器 | 复制WGSL着色器 |
| 纹理渲染 | MTT 纹理系统 | 参考纹理图集实现 |
| 文本渲染 | MTT 字体渲染 | 参考字体加载和渲染 |
| 批量渲染 | MTT 渲染优化 | 参考Draw Call优化 |

**关键代码参考**:
```rust
// MTT File Manager - 多后端降级
// src/main.rs:run_main_app_with_fallback
fn run_main_app_with_fallback(...) -> (eframe::Result<()>, bool) {
    let renderers = startup_renderers(preference);
    for (index, renderer) in renderers.iter().copied().enumerate() {
        let (result, app_started) = run_main_app_attempt(viewport.clone(), renderer);
        match result {
            Ok(()) => return (Ok(()), app_started),
            Err(error) if !app_started && index + 1 < renderers.len() => {
                log::error!("{} initialization failed: {}. Trying {}.", ...);
            }
            Err(error) => return (Err(error), app_started),
        }
    }
}
```

#### Week 4: 文件系统抽象
**参考**: MTT File Manager (`src/app/file_system/`) + FileMan (`src/core/`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| FileSystem trait | MTT trait定义 | 复制接口设计 |
| LocalFileSystem | MTT 本地实现 | 参考Windows API调用 |
| tokio异步 | MTT 异步读取 | 参考异步文件操作 |
| 文件元数据 | MTT 元数据获取 | 参考Windows Shell API |

#### Week 5-6: 基础UI框架
**参考**: MTT File Manager (`src/app/ui/`) + FileMan (`src/ui/`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| Component trait | MTT 组件系统 | 复制trait设计 |
| Button组件 | MTT Button | 参考按钮样式和交互 |
| Input组件 | MTT Input | 参考输入框实现 |
| Label组件 | MTT Label | 参考标签实现 |
| 布局引擎 | MTT 布局 | 参考Flexbox实现 |

#### Week 7-8: 主题系统 + 单面板
**参考**: MTT File Manager (`src/app/theme.rs`) + FileMan (`src/theme.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| Theme结构 | MTT Theme | 复制主题定义 |
| 浅色主题 | MTT 浅色 | 参考颜色配置 |
| Panel组件 | MTT Panel | 参考面板容器 |
| 标签页 | MTT TabBar | 参考标签页实现 |
| 文件列表 | MTT FileList | 参考列表实现 |

**关键代码参考**:
```rust
// MTT File Manager - 主题系统
// src/app/theme.rs
pub struct Theme {
    pub colors: ThemeColors,
    pub fonts: FontConfig,
    pub spacing: SpacingConfig,
}

// FileMan - 主题应用
// src/theme.rs
pub fn apply_theme(ctx: &egui::Context, colors: &ThemeColors) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
    style.visuals.window_fill = color32(colors.preview_bg);
    // ...
    ctx.set_global_style(style);
}
```

---

### Phase 1: 核心功能 (14周)

#### Week 9-10: 多面板布局
**参考**: Q-Dir (四面板) + MTT File Manager (双面板)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| PanelContainer组件 | MTT `panel_container.rs` | 参考面板容器实现 |
| 1-4面板动态布局 | Q-Dir 四面板 | 参考布局逻辑 |
| 面板拖拽调整 | MTT 拖拽 | 参考分割线拖拽 |
| 面板最小宽度 | MTT 限制 | 参考200px限制 |

**关键代码参考**:
```rust
// MTT File Manager - 面板容器
// 参考 MTT 源码中的 panel 模块
pub struct PanelContainer {
    panels: Vec<Panel>,
    layout: LayoutMode,
    drag_state: Option<DragState>,
}

impl PanelContainer {
    pub fn new(layout: LayoutMode) -> Self {
        Self {
            panels: Vec::new(),
            layout,
            drag_state: None,
        }
    }
    
    // 参考 MTT 的布局计算
    pub fn calculate_layout(&self, bounds: Rect) -> Vec<Rect> {
        match self.layout {
            LayoutMode::Single => vec![bounds],
            LayoutMode::DualHorizontal => {
                let half_width = bounds.width() / 2.0;
                vec![
                    Rect::new(bounds.min, vec2(half_width, bounds.height())),
                    Rect::new(vec2(half_width, 0.0), bounds.max),
                ]
            }
            // ... 其他布局
        }
    }
}
```

#### Week 11-12: 标签页+面包屑
**参考**: MTT File Manager (`src/app/tab_bar.rs` + `src/app/breadcrumb.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| Tab创建 | MTT TabBar | 参考Ctrl+T实现 |
| Tab关闭 | MTT TabBar | 参考Ctrl+W实现 |
| Tab切换 | MTT TabBar | 参考Ctrl+Tab实现 |
| 面包屑显示 | MTT Breadcrumb | 参考路径层级显示 |
| 面包屑交互 | MTT Breadcrumb | 参考点击/双击交互 |

**关键代码参考**:
```rust
// MTT File Manager - 标签页
// 参考 MTT 源码中的 tab 模块
pub struct TabBar {
    tabs: Vec<Tab>,
    active_index: usize,
}

impl TabBar {
    pub fn handle_shortcut(&mut self, key: &str, modifiers: &Modifiers) {
        match (key, modifiers) {
            ("T", Modifiers::CTRL) => self.new_tab(),
            ("W", Modifiers::CTRL) => self.close_current_tab(),
            ("Tab", Modifiers::CTRL) => self.next_tab(),
            _ => {}
        }
    }
}
```

#### Week 13-15: 文件列表+操作
**参考**: MTT File Manager (`src/app/file_list.rs`) + Total Commander (操作逻辑)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| 列定义 | MTT FileList | 参考5列布局 |
| 行高设置 | MTT 36px | 复制行高配置 |
| 悬停状态 | MTT Hover | 参考Primary Light |
| 选中状态 | MTT Selected | 参考Primary背景 |
| 排序功能 | MTT Sort | 参考列头点击排序 |
| 虚拟滚动 | MTT Virtual | 参考虚拟化实现 |
| 文件操作 | TC操作逻辑 | 参考Ctrl+C/X/V/Delete |

**关键代码参考**:
```rust
// MTT File Manager - 文件列表
// 参考 MTT 源码中的 file_list 模块
pub struct FileList {
    entries: Vec<DirEntry>,
    selected_index: usize,
    scroll_offset: f32,
    sort_mode: SortMode,
    sort_desc: bool,
}

impl FileList {
    pub fn render(&self, ui: &mut egui::Ui) {
        let row_height = 36.0;
        let columns = [
            ("Icon", 32.0),
            ("Name", ui.available_width() - 32.0 - 120.0 - 80.0 - 140.0),
            ("Type", 120.0),
            ("Size", 80.0),
            ("Date", 140.0),
        ];
        
        // 虚拟化渲染
        let visible_rows = (ui.available_height() / row_height) as usize;
        let start = (self.scroll_offset / row_height) as usize;
        let end = (start + visible_rows).min(self.entries.len());
        
        for i in start..end {
            let entry = &self.entries[i];
            self.render_row(ui, entry, i == self.selected_index);
        }
    }
}
```

#### Week 16-17: 拖拽+地址栏
**参考**: MTT File Manager (`src/app/drag_drop.rs` + `src/app/address_bar.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| 跨面板拖拽 | MTT drag_drop | 参考拖拽逻辑 |
| 拖拽视觉 | MTT 视觉反馈 | 参考高亮实现 |
| 地址栏 | MTT address_bar | 参考Ctrl+L激活 |
| 历史路径 | MTT 历史 | 参考下拉列表 |
| Tab路径 | MTT Tab路径 | 参考已打开Tab显示 |

#### Week 18-20: 侧边栏
**参考**: MTT File Manager (`src/app/sidebar.rs`) + DOpus (侧边栏设计)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| 侧边栏组件 | MTT Sidebar | 参考侧边栏实现 |
| 显示/隐藏 | MTT Ctrl+Shift+B | 参考切换逻辑 |
| 位置切换 | MTT 位置切换 | 参考左/右切换 |
| 拖拽调整 | MTT 拖拽 | 参考150px-400px限制 |
| 此电脑 | MTT ThisPC | 参考磁盘列表 |
| 标签 | MTT Tags | 参考用户标签 |
| 最近访问 | MTT Recent | 参考最近文件 |
| 空间管理 | MTT Space | 参考Space切换 |

**关键代码参考**:
```rust
// MTT File Manager - 侧边栏
// 参考 MTT 源码中的 sidebar 模块
pub struct Sidebar {
    visible: bool,
    position: SidebarPosition,
    width: f32,
    sections: Vec<SidebarSection>,
}

impl Sidebar {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
    
    pub fn set_position(&mut self, position: SidebarPosition) {
        self.position = position;
    }
    
    pub fn handle_drag(&mut self, delta: f32) {
        self.width = (self.width + delta).clamp(150.0, 400.0);
    }
    
    pub fn double_click_reset(&mut self) {
        self.width = 200.0;
    }
}
```

#### Week 21-22: 状态栏+主题+快捷键
**参考**: MTT File Manager (`src/app/status_bar.rs` + `src/app/shortcuts.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| 状态栏 | MTT StatusBar | 参考状态栏实现 |
| 深色主题 | MTT 深色 | 参考深色配置 |
| 跟随系统 | MTT 系统主题 | 参考自动切换 |
| 全局快捷键 | MTT Shortcuts | 参考快捷键系统 |
| 面板快捷键 | MTT 面板快捷键 | 参考Ctrl+T等 |
| 文件操作快捷键 | MTT 文件快捷键 | 参考Ctrl+C等 |

---

### Phase 2: 增强功能 (12周)

#### Week 23-24: 文件预览
**参考**: MTT File Manager (`src/app/preview_panel.rs`) + XYplorer (预览设计)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| PreviewPanel组件 | MTT PreviewPanel | 参考预览面板实现 |
| Space触发 | MTT Space键 | 参考触发逻辑 |
| 1/3或2/3屏幕 | MTT 宽度切换 | 参考宽度计算 |
| 图片预览 | MTT 图片预览 | 参考缩略图实现 |
| 文本预览 | MTT 文本预览 | 参考语法高亮 |
| PDF预览 | MTT PDF预览 | 参考PDFium集成 |

**关键代码参考**:
```rust
// MTT File Manager - 文件预览
// 参考 MTT 源码中的 preview 模块
pub struct PreviewPanel {
    visible: bool,
    width_ratio: f32, // 1/3 或 2/3
    content: Option<PreviewContent>,
}

impl PreviewPanel {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
    
    pub fn cycle_width(&mut self) {
        self.width_ratio = if self.width_ratio < 0.5 {
            2.0 / 3.0
        } else {
            1.0 / 3.0
        };
    }
    
    pub fn render(&self, ui: &mut egui::Ui) {
        if let Some(content) = &self.content {
            match content {
                PreviewContent::Image(texture) => {
                    ui.image(texture);
                }
                PreviewContent::Text(text, highlights) => {
                    // 语法高亮渲染
                }
                PreviewContent::Pdf(pages) => {
                    // PDF页面渲染
                }
            }
        }
    }
}
```

#### Week 25-26: 全局搜索
**参考**: MTT File Manager (`src/app/search.rs`) + Everything (搜索设计)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| SearchEngine | MTT SearchEngine | 参考搜索引擎实现 |
| 实时过滤 | MTT 实时过滤 | 参考输入即搜索 |
| 文件名搜索 | MTT 文件名 | 参考文件名匹配 |
| 内容搜索 | MTT 内容 | 参考内容搜索 |
| 正则支持 | MTT 正则 | 参考正则实现 |

#### Week 27-28: 文件标签
**参考**: Tessoa (彩色标签) + MTT File Manager (标签系统)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| TagManager | Tessoa Tags | 参考标签管理 |
| 彩色标签 | Tessoa 彩色 | 参考颜色定义 |
| 标签分配 | Tessoa 分配 | 参考分配逻辑 |
| 标签过滤 | Tessoa 过滤 | 参见过滤实现 |

#### Week 29-30: 批量重命名
**参考**: Total Commander (批量重命名) + MTT File Manager

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| BatchRename对话框 | TC 批量重命名 | 参考对话框设计 |
| 正则表达式 | TC 正则 | 参考正则支持 |
| 预览结果 | TC 预览 | 参考预览实现 |

#### Week 31-32: 压缩包操作
**参考**: MTT File Manager (`src/app/archive.rs`) + FileMan

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| 压缩包导航 | MTT ArchiveNav | 参考zip/7z/tar浏览 |
| 压缩包创建 | MTT ArchiveCreate | 参考ZIP/7Z创建 |
| 压缩包解压 | MTT ArchiveExtract | 参考解压实现 |
| 支持格式 | MTT FormatList | zip/7z/tar/gz/bz2/xz |

#### Week 33-34: 右键菜单+文件分组
**参考**: MTT File Manager (`src/app/context_menu.rs` + `src/app/file_group.rs`)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| Shell右键菜单 | MTT ContextMenu | 参考Windows Shell集成 |
| 原生新建菜单 | MTT NewMenu | 参考新建子菜单 |
| 文件分组 | MTT FileGroup | 参考按类型/日期/大小分组 |
| 分组折叠/展开 | MTT GroupFold | 参考折叠交互 |

#### Week 35-36: 快速预览+回收站
**参考**: Nexus Explorer (Quick Look) + MTT File Manager (回收站)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| Quick Look | Nexus Space键 | 参考快速预览触发 |
| 预览动画 | Nexus 动画 | 参考淡入淡出效果 |
| 回收站浏览 | MTT Trash | 参考回收站列表 |
| 回收站恢复 | MTT Restore | 参考文件恢复 |
| 永久删除 | MTT PermanentDelete | 参考彻底删除 |

---

### Phase 3: 高级功能 (10周)

#### Week 37-38: 分栏视图+网格排布
**参考**: macOS Finder (Column View) + Tessoa (网格排布)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| ColumnView组件 | Finder Column | 参考分栏设计 |
| 点击展开 | Finder 交互 | 参考展开逻辑 |
| 键盘导航 | Finder 导航 | 参考左右导航 |
| 规则网格 | Tessoa Grid | 参考规则网格排布 |
| 等高行 | Tessoa RowHeight | 参考等高行排布 |
| 瀑布流 | Tessoa Waterfall | 参考瀑布流排布 |
| 马赛克拼贴 | Tessoa Mosaic | 参考马赛克排布 |

#### Week 39-40: 画廊视图+文件分组
**参考**: MTT File Manager (`src/app/gallery_view.rs`)

| 任务 | 参考文件 | 实现要点 |
|-----|---------|---------|
| GalleryView组件 | MTT GalleryView | 参考画廊实现 |
| 缩略图网格 | MTT 网格 | 参考网格布局 |
| 大图预览 | MTT 预览 | 参考大图实现 |
| 幻灯片 | MTT 幻灯片 | 参考幻灯片播放 |

#### Week 41-42: 会话恢复+快速访问
**参考**: Tessoa (会话恢复) + MTT File Manager (快速访问)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| 会话状态保存 | Tessoa Session | 参考状态序列化 |
| 会话自动恢复 | Tessoa Restore | 参考启动时恢复 |
| 快速访问固定 | MTT QuickAccess | 参考右键固定文件夹 |
| 书签管理 | MTT Bookmarks | 参考书签增删改 |
| 拖拽排序 | MTT DragSort | 参考拖拽重排序 |

#### Week 43-44: 设置面板+底栏控件
**参考**: Tessoa (设置面板) + MTT File Manager (状态栏)

| 任务 | 参考来源 | 实现要点 |
|-----|---------|---------|
| 设置面板 | Tessoa Settings | 参考命令面板风格 |
| 外观设置 | MTT Appearance | 参考主题/字体配置 |
| 快捷键设置 | MTT Shortcuts | 参考快捷键自定义 |
| 底栏布局控件 | Tessoa BottomBar | 参考布局切换按钮 |
| 底栏Space控件 | Tessoa SpaceBar | 参考Space切换 |
| 底栏信息显示 | MTT StatusBar | 参考面板/路径信息 |

---

## 四、竞品参考索引

### 3.1 MTT File Manager (主要参考)
**GitHub**: https://github.com/MTTamurex/MTT-File-Manager-RUST

| 功能模块 | 参考文件 | 实现要点 |
|---------|---------|---------|
| GPU渲染 | `src/app/mod.rs`, `gpu_backend.rs` | 多后端降级，DirectComposition |
| 主题系统 | `src/app/theme.rs` | 颜色定义，主题切换 |
| 文件列表 | `src/app/file_list.rs` | 虚拟化，排序，选择 |
| 标签页 | `src/app/tab_bar.rs` | 创建/关闭/切换 |
| 面包屑 | `src/app/breadcrumb.rs` | 路径层级，点击跳转 |
| 侧边栏 | `src/app/sidebar.rs` | 磁盘/标签/最近/空间 |
| 状态栏 | `src/app/status_bar.rs` | 面板信息，布局切换 |
| 预览面板 | `src/app/preview_panel.rs` | 图片/文本/PDF预览 |
| 搜索 | `src/app/search.rs` | 实时过滤，内容搜索 |
| 拖拽 | `src/app/drag_drop.rs` | 跨面板拖拽 |
| 地址栏 | `src/app/address_bar.rs` | Ctrl+L，历史路径 |

### 3.2 FileMan (次要参考)
**GitHub**: https://github.com/kvark/fileman

| 功能模块 | 参考文件 | 实现要点 |
|---------|---------|---------|
| 双面板 | `src/main.rs` | 面板布局 |
| SSH/SFTP | SSH模块 | 远程连接 |
| 语法高亮 | syntect集成 | 代码高亮 |
| 主题 | `src/theme.rs` | 主题应用 |

### 3.3 Nexus Explorer (性能参考)
**GitHub**: https://github.com/Augani/nexus-explorer

| 功能模块 | 参考文件 | 实现要点 |
|---------|---------|---------|
| 并行遍历 | jwalk集成 | 4x加速 |
| 模糊搜索 | nucleo集成 | 8x搜索速度 |
| LRU缓存 | 缓存实现 | 即时返回 |

### 3.4 其他参考

| 项目 | 参考点 |
|-----|--------|
| **Q-Dir** | 四面板布局设计 |
| **Total Commander** | 操作逻辑，批量重命名 |
| **Directory Opus** | 侧边栏设计 |
| **Tessoa** | Vim操作，彩色标签 |
| **macOS Finder** | 分栏视图 |

---

## 五、开发原则

### 4.1 代码复用优先
- **不要自己写轮子**：优先参考竞品实现
- **复制+修改**：先复制竞品代码，再根据需求修改
- **保持一致性**：遵循竞品的代码风格和架构

### 4.2 性能优化参考
- **虚拟化渲染**：参考 MTT 的虚拟化实现
- **异步操作**：参考 MTT 的 tokio 异步
- **缓存策略**：参考 Nexus 的 LRU 缓存

### 4.3 交互设计参考
- **快捷键**：参考 Total Commander 的操作逻辑
- **视觉反馈**：参考 MTT 的主题系统
- **布局**：参考 Q-Dir 的多面板设计

---

## 六、验收标准

### 每周验收
- **代码质量**: `cargo fmt` + `cargo clippy` 无警告
- **功能完整**: 按参考实现完成
- **性能达标**: 帧率 > 60fps

### 里程碑验收
- **M1** (第8周): 核心框架可用
- **M2** (第22周): 核心功能完整
- **M3** (第30周): **1:1还原设计原型**
- **M4** (第36周): 增强功能完整
- **M5** (第44周): 高级功能完整

---

*文档版本: v3.1*  
*最后更新: 2026-09-01*

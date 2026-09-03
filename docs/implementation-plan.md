# Zero Explorer - 竞品分析与1:1复现开发计划

**版本**: v2.0  
**日期**: 2026-09-01  
**目标**: 准确还原原型设计，1:1复现 UI/UX

---

## 一、竞品信息整理

### 1.1 核心竞品技术栈对比

| 竞品 | 渲染方案 | UI框架 | 语言 | 特点 |
|-----|---------|--------|------|------|
| **MTT File Manager** | egui + wgpu (DX12/Vulkan/OpenGL) | eframe | Rust | 最成熟(1635 commits)，多后端切换 |
| **Nexus Explorer** | GPUI (Zed框架) | gpui | Rust | GPU加速，并行遍历，模糊搜索 |
| **Tessoa** | 自研GPU | 自研 | Rust | 任意分屏，Vim操作，彩色标签 |
| **Filane** | egui | eframe | Rust | 双面板，GUI+TUI双版本 |
| **Q-Dir** | Win32 GDI | 原生Win32 | C++ | 四面板，轻量级(<1000KB) |
| **Total Commander** | Win32 | 原生Win32 | Delphi | 经典双面板，插件生态丰富 |
| **Directory Opus** | Win32 | 原生Win32 | C++ | 功能最全面，高度可定制 |

### 1.2 技术方案分析

#### 渲染架构选择
| 方案 | 优点 | 缺点 | 适用场景 |
|-----|------|------|----------|
| **自研wgpu** | 完全控制，最高性能 | 开发成本高 | 极致性能需求 |
| **egui** | 开发快，组件丰富 | 性能受限于CPU渲染 | 快速原型，工具类应用 |
| **GPUI** | GPU加速，生产级 | 文档差，组件少 | 高性能编辑器 |
| **Iced** | Elm架构，类型安全 | 生态较新 | 标准CRUD应用 |

#### 文件遍历优化
| 竞品 | 方案 | 性能 |
|-----|------|------|
| **Nexus Explorer** | jwalk并行遍历 | 4x加速 |
| **MTT File Manager** | tokio异步 + 服务索引 | 实时响应 |
| **Tessoa** | 后台扫描 + 缓存 | 毫秒级 |

#### 图标系统
| 竞品 | 方案 | 特点 |
|-----|------|------|
| **Zero Explorer** | Nerd Fonts | TrueType矢量，GPU直接渲染 |
| **MTT File Manager** | Shell API | 系统原生图标 |
| **Nexus Explorer** | 内嵌图标字体 | 自定义图标 |

### 1.3 可借鉴的成熟经验

#### 1. MTT File Manager 经验
```rust
// 多后端降级策略
enum GpuBackend {
    WgpuDirectX12,  // 默认，最佳性能
    WgpuVulkan,     // 备选
    GlowOpenGL,     // 回退
}

// 缩略图多级生成
enum ThumbnailStage {
    ImageCrate,     // 基础图像处理
    Wic,            // Windows Imaging Component
    ShellApi,       // Windows Shell
    MediaFoundation, // 视频缩略图
}
```

#### 2. Nexus Explorer 经验
```rust
// 并行目录遍历
use jwalk::WalkDir;
let entries: Vec<_> = WalkDir::new(path)
    .parallelism(Parallelism::RayonNewPool(0))
    .into_iter()
    .collect();

// LRU缓存
use lru::LruCache;
let cache: LruCache<PathBuf, Vec<FileEntry>> = LruCache::new(1000);

// 代际请求丢弃
struct AppState {
    generation: u64,
    tx: mpsc::Sender<AppEvent>,
}
```

#### 3. 通用优化经验
- **虚拟化渲染**：只渲染可见行，支持百万文件
- **批量更新**：100项或16ms批次，防止渲染抖动
- **主线程永不阻塞**：所有I/O在后台线程
- **LRU缓存**：已访问目录缓存，即时返回

---

## 二、1:1复现开发计划

### 2.1 设计原型核心要素

根据 `DESIGN.md` 和 `ui-design.md`，1:1复现需要覆盖：

#### 颜色系统
```rust
pub struct Theme {
    // 主色调 - Windows 系统蓝
    pub primary: Color,           // #0078D4
    pub primary_hover: Color,     // #106EBE
    pub primary_active: Color,    // #005A9E
    pub primary_light: Color,     // #E8F4FD
    
    // 中性色
    pub background: Color,        // #FFFFFF
    pub secondary: Color,         // #F9F9F9
    pub tertiary: Color,          // #F3F3F3
    pub border: Color,            // #E5E5E5
    pub border_strong: Color,     // #D1D1D1
    pub text_primary: Color,      // #1A1A1A
    pub text_secondary: Color,    // #616161
    pub text_tertiary: Color,     // #9E9E9E
    
    // 状态色
    pub success: Color,           // #0F7B0F
    pub warning: Color,           // #9D5D00
    pub error: Color,             // #C42B1C
}
```

#### 字体系统
```rust
pub struct FontConfig {
    // 字体家族
    pub display: String,          // "Segoe UI Variable Display"
    pub body: String,             // "Segoe UI Variable Text"
    pub monospace: String,        // "Cascadia Code"
    
    // 字体比例
    pub display_size: f32,        // 28px
    pub title_size: f32,          // 20px
    pub subtitle_size: f32,       // 16px
    pub body_size: f32,           // 14px
    pub caption_size: f32,        // 12px
    pub mono_size: f32,           // 13px
}
```

#### 间距系统 (4px网格)
```rust
pub struct Spacing {
    pub space_1: f32,  // 4px
    pub space_2: f32,  // 8px
    pub space_3: f32,  // 12px
    pub space_4: f32,  // 16px
    pub space_5: f32,  // 20px
    pub space_6: f32,  // 24px
    pub space_8: f32,  // 32px
}
```

#### 圆角系统
```rust
pub struct BorderRadius {
    pub sm: f32,  // 4px - 按钮、标签
    pub md: f32,  // 6px - 输入框、列表
    pub lg: f32,  // 8px - 卡片、面板
}
```

#### 阴影系统
```rust
pub struct Shadow {
    pub sm: String,  // "0 1px 2px rgba(0,0,0,0.05)"
    pub md: String,  // "0 4px 8px rgba(0,0,0,0.08)"
    pub lg: String,  // "0 8px 16px rgba(0,0,0,0.12)"
}
```

### 2.2 布局结构1:1复现

#### 主布局结构
```
┌─────────────────────────────────────────────────────────────────┐
│  工具栏: [← 后退] [↑ 上级] [⟲ 刷新] [面包屑] [🔍 搜索] [☰ 视图] [⊞ 侧栏] [◧左|◨右] [⚙ 设置] │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┬──────────────┐                                │
│  │  面板 1      │  面板 2      │  ← 无标题栏/工具栏              │
│  │  ┌────────┐ │  ┌────────┐ │                                │
│  │  │面包屑   │ │  │面包屑   │ │  ← Tab 上方                   │
│  │  ├────────┤ │  ├────────┤ │                                │
│  │  │ 标签页  │ │  │ 标签页  │ │  ← Tab 下方                   │
│  │  ├────────┤ │  ├────────┤ │                                │
│  │  │ 文件列表│ │  │ 文件列表│ │                                │
│  │  └────────┘ │  └────────┘ │                                │
│  └──────────────┴──────────────┘                                │
├─────────────────────────────────────────────────────────────────┤
│  状态栏 (项目数量/选中状态/当前路径)                                │
└─────────────────────────────────────────────────────────────────┘
```

#### 侧边栏布局
```
左侧侧边栏（默认）：              右侧侧边栏：
┌──────────┬─────────────┐        ┌─────────────┬──────────┐
│          │             │        │             │          │
│  侧边栏   │  主内容区    │        │  主内容区    │  侧边栏   │
│  (导航)   │             │        │             │  (导航)   │
│  ┌──────┐│             │        │             │  ┌──────┐│
│  │此电脑 ││  ← 磁盘空间进度条     │             │  │此电脑 ││
│  │标签   ││  ← 用户自定义的常用文件夹和标签       │  │标签   ││
│  │最近   ││  ← 最近访问文件      │             │  │最近   ││
│  │空间   ││  ← Space 管理（切换/新建）           │  │空间   ││
│  └──────┘│             │        │             │  └──────┘│
│          │             │        │             │          │
└──────────┴─────────────┘        └─────────────┴──────────┘
```

#### 状态栏布局
```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│ [多面板] [级联] | [🏠默认] [💼Work] [💻Dev] [+] | 4个面板·1个选中 | D:\...\ | [1] [2] [3] [4] │
└─────────────────────────────────────────────────────────────────────────────────────┘
  ↑ 模式切换          ↑ Space管理              ↑ 面板信息      ↑ 路径       ↑ 布局切换
```

### 2.3 组件1:1复现规格

#### 按钮组件
```rust
pub struct Button {
    pub height: f32,           // 32px
    pub padding_x: f32,        // 8px
    pub padding_y: f32,        // 16px
    pub border_radius: f32,    // 4px
    pub font_size: f32,        // 14px
    pub font_weight: u32,      // 500
    pub button_type: ButtonType, // Primary/Secondary/Ghost
}

pub enum ButtonType {
    Primary,    // 蓝色背景，白色文字
    Secondary,  // 白色背景，边框
    Ghost,      // 透明背景，无边框
}
```

#### 输入框组件
```rust
pub struct Input {
    pub height: f32,           // 32px
    pub padding_x: f32,        // 12px
    pub border_radius: f32,    // 4px
    pub border: Border,        // 1px solid #D1D1D1
    pub focus_border: Color,   // 蓝色边框
    pub focus_shadow: String,  // 蓝色阴影
}
```

#### 标签页组件
```rust
pub struct Tab {
    pub background: Color,     // #F3F3F3
    pub active_background: Color, // 白色背景
    pub active_border: Color,  // 底部蓝色边框
    pub hover_background: Color, // 白色背景
    pub font_size: f32,        // 13px
    pub font_weight: u32,      // 500
}
```

#### 面包屑组件
```rust
pub struct Breadcrumb {
    pub position: BreadcrumbPosition, // Tab上方
    pub click_action: ClickAction,    // 单击跳转
    pub double_click_action: ClickAction, // 双击切换输入框
    pub input_action: InputAction,    // Enter跳转
    pub cancel_action: CancelAction,  // Esc取消
}
```

#### 文件列表组件
```rust
pub struct FileList {
    pub row_height: f32,       // 36px
    pub columns: Vec<Column>,  // 列定义
    pub hover_style: Style,    // Primary Light背景
    pub selected_style: Style, // Primary背景，白色文字
}

pub struct Column {
    pub icon_width: f32,       // 32px
    pub name_width: f32,       // 1fr
    pub type_width: f32,       // 120px
    pub size_width: f32,       // 80px
    pub date_width: f32,       // 140px
}
```

---

## 三、分阶段开发计划

### Phase 0: 核心框架 (0-2个月)

#### 0.1 项目架构搭建 (1周)
**任务清单**：
- [ ] Cargo项目初始化，配置依赖
- [ ] 分层架构：`core/` `ui/` `fs/` `app/`
- [ ] CI/CD搭建 (GitHub Actions)
- [ ] 基础错误处理框架

**依赖选型**：
```toml
[dependencies]
winit = "0.29"
wgpu = "0.19"
pollster = "0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

#### 0.2 GPU渲染管线 (2周)
**任务清单**：
- [ ] wgpu初始化 (Device/Queue/Surface)
- [ ] 基础Vertex/Fragment Shader
- [ ] 纹理渲染支持
- [ ] 文本渲染 (glyph-brush)
- [ ] 批量渲染优化

**关键文件**：
- `src/ui/renderer.rs` - GPU渲染器核心
- `src/ui/shaders/` - WGSL着色器

#### 0.3 文件系统抽象 (1周)
**任务清单**：
- [ ] FileSystem trait定义
- [ ] LocalFileSystem实现
- [ ] tokio异步文件操作
- [ ] 文件元数据获取
- [ ] 路径处理工具

#### 0.4 基础UI框架 (2周)
**任务清单**：
- [ ] Component trait定义
- [ ] Button/Input/Label基础组件
- [ ] 布局引擎 (Flexbox)
- [ ] 事件分发系统
- [ ] 主题系统基础

### Phase 1: 核心功能 (2-4个月)

#### 1.1 多面板布局 (2周)
**任务清单**：
- [ ] PanelContainer组件
- [ ] 1-4面板动态布局
- [ ] 面板拖拽调整宽度
- [ ] 面板最小宽度限制 (200px)
- [ ] 面板平均分配

**布局模式**：
```rust
pub enum LayoutMode {
    Single,           // 单面板
    DualHorizontal,   // 左右分栏
    DualVertical,     // 上下分栏
    TripleLeft,       // 左中右
    TripleRight,      // 上2下1
    TripleTop,        // 上1下2
    Quad,             // 四面板
}
```

#### 1.2 标签页系统 (1周)
**任务清单**：
- [ ] TabBar组件
- [ ] Tab创建/关闭/切换
- [ ] Ctrl+T新建，Ctrl+W关闭
- [ ] 最少保留一个Tab
- [ ] Tab拖拽调整顺序

#### 1.3 面包屑导航 (1周)
**任务清单**：
- [ ] Breadcrumb组件
- [ ] 路径层级显示
- [ ] 单击跳转目录
- [ ] 双击切换输入框
- [ ] Enter跳转，Esc取消

#### 1.4 文件列表 (2周)
**任务清单**：
- [ ] FileList组件
- [ ] 列定义 (图标/名称/类型/大小/时间)
- [ ] 行高设置 (36px)
- [ ] 悬停/选中状态
- [ ] 排序功能
- [ ] 虚拟滚动

**列宽配置**：
```rust
pub struct FileListColumns {
    pub icon: f32,      // 32px
    pub name: f32,      // 1fr
    pub type_name: f32, // 120px
    pub size: f32,      // 80px
    pub modified: f32,  // 140px
}
```

#### 1.5 文件操作 (2周)
**任务清单**：
- [ ] 复制/移动/删除/重命名
- [ ] Ctrl+C/X/V/Delete/F2
- [ ] 确认对话框
- [ ] 进度显示

#### 1.6 拖拽交互 (1周)
**任务清单**：
- [ ] 跨面板拖拽
- [ ] 拖拽移动/复制
- [ ] 视觉反馈
- [ ] 放下反馈

#### 1.7 地址栏 (1周)
**任务清单**：
- [ ] AddressBar组件
- [ ] Ctrl+L激活
- [ ] 历史路径下拉
- [ ] Tab路径显示
- [ ] Enter跳转

#### 1.8 侧边栏 (2周)
**任务清单**：
- [ ] Sidebar组件
- [ ] 显示/隐藏 (Ctrl+Shift+B)
- [ ] 位置切换 (左/右)
- [ ] 拖拽调整宽度 (150px-400px)
- [ ] 双击恢复默认 (200px)
- [ ] 此电脑/标签/最近/空间

#### 1.9 状态栏 (1周)
**任务清单**：
- [ ] StatusBar组件
- [ ] 面板信息显示
- [ ] 路径显示
- [ ] 路径复制
- [ ] 布局切换

#### 1.10 主题系统 (1周)
**任务清单**：
- [ ] Theme结构体
- [ ] 浅色/深色主题
- [ ] 跟随系统
- [ ] 主题切换
- [ ] 主题持久化

#### 1.11 快捷键系统 (1周)
**任务清单**：
- [ ] ShortcutManager
- [ ] 全局/面板/文件操作快捷键
- [ ] 导航/视图快捷键
- [ ] 快捷键提示

### Phase 2: 增强功能 (4-6个月)

#### 2.1 文件预览 (2周)
**任务清单**：
- [ ] PreviewPanel组件
- [ ] Space触发
- [ ] 1/3或2/3屏幕宽度
- [ ] 图片/文本/PDF预览
- [ ] 文件信息显示

#### 2.2 全局搜索 (2周)
**任务清单**：
- [ ] SearchEngine
- [ ] 实时过滤
- [ ] 文件名/内容搜索
- [ ] 正则支持

#### 2.3 文件标签 (1周)
**任务清单**：
- [ ] TagManager
- [ ] 彩色标签
- [ ] 标签分配/过滤

#### 2.4 批量重命名 (1周)
**任务清单**：
- [ ] BatchRename对话框
- [ ] 正则表达式支持
- [ ] 预览结果

#### 2.5 Vim模式 (2周)
**任务清单**：
- [ ] VimManager
- [ ] Normal/Insert/Visual模式
- [ ] j/k/gg/G/v/y/x/p等快捷键

### Phase 3: 高级功能 (6-8个月)

#### 3.1 分栏视图 (2周)
**任务清单**：
- [ ] ColumnView组件
- [ ] Finder风格列导航
- [ ] 点击展开新一栏
- [ ] 键盘左右导航

#### 3.2 画廊视图 (1周)
**任务清单**：
- [ ] GalleryView组件
- [ ] 媒体文件缩略图网格
- [ ] 大图预览/幻灯片

#### 3.3 文件比较 (2周)
**任务清单**：
- [ ] FileCompare组件
- [ ] 并排比较
- [ ] 差异高亮

#### 3.4 远程协议 (3周)
**任务清单**：
- [ ] SSH/SFTP连接
- [ ] 远程浏览/传输

#### 3.5 插件系统 (3周)
**任务清单**：
- [ ] Plugin接口定义
- [ ] 动态加载
- [ ] 管理UI

---

## 四、验收标准

### Phase 0 验收
- [ ] Windows 10/11可运行
- [ ] GPU渲染正常
- [ ] 单面板+标签页可用
- [ ] 启动时间 < 500ms

### Phase 1 验收
- [ ] 多面板布局正常
- [ ] 文件操作完整
- [ ] 侧边栏功能完整
- [ ] 主题切换正常
- [ ] 启动时间 < 200ms
- [ ] 10万文件响应 < 100ms
- [ ] **1:1还原设计原型**

### Phase 2 验收
- [ ] 文件预览完整
- [ ] 全局搜索可用
- [ ] 标签系统完整
- [ ] 可替代系统管理器

### Phase 3 验收
- [ ] 分栏/画廊视图正常
- [ ] 远程连接可用
- [ ] 插件系统可用

---

## 五、技术风险与应对

### 5.1 渲染性能风险
**风险**：GPU渲染在大目录下卡顿
**应对**：
- 虚拟化渲染，只渲染可见行
- 批量更新，100项或16ms批次
- LRU缓存已访问目录

### 5.2 文件系统风险
**风险**：大量文件时遍历缓慢
**应对**：
- 引入jwalk并行遍历
- tokio异步操作
- 后台扫描+缓存

### 5.3 内存风险
**风险**：大量文件时内存占用高
**应对**：
- 虚拟化渲染
- 分页加载
- 及时释放不可见数据

---

*文档版本: v2.0*  
*最后更新: 2026-09-01*

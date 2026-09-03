# Zero Explorer - 设计规范速查表

**版本**: v1.0  
**日期**: 2026-09-01  
**用途**: 1:1复现设计原型的快速参考

---

## 一、颜色系统速查

### 主色调
```rust
// Primary - Windows 系统蓝
pub const PRIMARY: Color = Color::from_hex("#0078D4");
pub const PRIMARY_HOVER: Color = Color::from_hex("#106EBE");
pub const PRIMARY_ACTIVE: Color = Color::from_hex("#005A9E");
pub const PRIMARY_LIGHT: Color = Color::from_hex("#E8F4FD");
```

### 中性色
```rust
pub const BG_BASE: Color = Color::from_hex("#FFFFFF");
pub const BG_SECONDARY: Color = Color::from_hex("#F9F9F9");
pub const BG_TERTIARY: Color = Color::from_hex("#F3F3F3");
pub const BORDER: Color = Color::from_hex("#E5E5E5");
pub const BORDER_STRONG: Color = Color::from_hex("#D1D1D1");
pub const TEXT_PRIMARY: Color = Color::from_hex("#1A1A1A");
pub const TEXT_SECONDARY: Color = Color::from_hex("#616161");
pub const TEXT_TERTIARY: Color = Color::from_hex("#9E9E9E");
```

### 状态色
```rust
pub const SUCCESS: Color = Color::from_hex("#0F7B0F");
pub const WARNING: Color = Color::from_hex("#9D5D00");
pub const ERROR: Color = Color::from_hex("#C42B1C");
```

---

## 二、字体系统速查

### 字体家族
```rust
pub const FONT_DISPLAY: &str = "Segoe UI Variable Display";
pub const FONT_BODY: &str = "Segoe UI Variable Text";
pub const FONT_MONO: &str = "Cascadia Code";
```

### 字体大小
```rust
pub const SIZE_DISPLAY: f32 = 28.0;  // Display
pub const SIZE_TITLE: f32 = 20.0;    // Title
pub const SIZE_SUBTITLE: f32 = 16.0; // Subtitle
pub const SIZE_BODY: f32 = 14.0;     // Body
pub const SIZE_CAPTION: f32 = 12.0;  // Caption
pub const SIZE_MONO: f32 = 13.0;     // Monospace
```

### 字重
```rust
pub const WEIGHT_REGULAR: u32 = 400;
pub const WEIGHT_MEDIUM: u32 = 500;
pub const WEIGHT_SEMIBOLD: u32 = 600;
```

---

## 三、间距系统速查 (4px网格)

```rust
pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_5: f32 = 20.0;
pub const SPACE_6: f32 = 24.0;
pub const SPACE_8: f32 = 32.0;
```

---

## 四、圆角系统速查

```rust
pub const RADIUS_SM: f32 = 4.0;  // 按钮、标签
pub const RADIUS_MD: f32 = 6.0;  // 输入框、列表
pub const RADIUS_LG: f32 = 8.0;  // 卡片、面板
```

---

## 五、阴影系统速查

```rust
pub const SHADOW_SM: &str = "0 1px 2px rgba(0,0,0,0.05)";
pub const SHADOW_MD: &str = "0 4px 8px rgba(0,0,0,0.08)";
pub const SHADOW_LG: &str = "0 8px 16px rgba(0,0,0,0.12)";
```

---

## 六、组件规格速查

### 按钮
```rust
pub struct ButtonSpec {
    pub height: f32 = 32.0,
    pub padding_x: f32 = 16.0,
    pub padding_y: f32 = 8.0,
    pub border_radius: f32 = 4.0,
    pub font_size: f32 = 14.0,
    pub font_weight: u32 = 500,
}
```

### 输入框
```rust
pub struct InputSpec {
    pub height: f32 = 32.0,
    pub padding_x: f32 = 12.0,
    pub border_radius: f32 = 4.0,
    pub border_width: f32 = 1.0,
}
```

### 标签页
```rust
pub struct TabSpec {
    pub font_size: f32 = 13.0,
    pub font_weight: u32 = 500,
    pub padding_x: f32 = 12.0,
    pub padding_y: f32 = 8.0,
}
```

### 文件列表
```rust
pub struct FileListSpec {
    pub row_height: f32 = 36.0,
    pub icon_width: f32 = 32.0,
    pub name_width: f32 = 1.0,  // 1fr
    pub type_width: f32 = 120.0,
    pub size_width: f32 = 80.0,
    pub date_width: f32 = 140.0,
}
```

---

## 七、布局规格速查

### 面板
```rust
pub struct PanelSpec {
    pub min_width: f32 = 200.0,
    pub default_width: f32 = 0.0,  // 平均分配
}
```

### 侧边栏
```rust
pub struct SidebarSpec {
    pub min_width: f32 = 150.0,
    pub max_width: f32 = 400.0,
    pub default_width: f32 = 200.0,
    pub resize_handle_width: f32 = 4.0,
}
```

### 状态栏
```rust
pub struct StatusBarSpec {
    pub height: f32 = 24.0,
    pub font_size: f32 = 11.0,
}
```

---

## 八、布局模式速查

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

### CSS Grid实现
```css
.layout-1 { grid-template-columns: 1fr; grid-template-rows: 1fr; }
.layout-2 { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr; }
.layout-3 { grid-template-columns: 1fr 1fr 1fr; grid-template-rows: 1fr; }
.layout-4 { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; }
.layout-5 { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; }
.layout-6 { grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; }
.layout-7 { grid-template-columns: 1fr; grid-template-rows: 1fr 1fr; }
```

---

## 九、快捷键速查

### 全局
```rust
pub const SHORTCUT_SPACE: &str = "Space";           // 预览面板
pub const SHORTCUT_SIDEBAR: &str = "Ctrl+Shift+B"; // 侧边栏
pub const SHORTCUT_ADDRESS: &str = "Ctrl+L";       // 地址栏
```

### 标签页
```rust
pub const SHORTCUT_TAB_NEW: &str = "Ctrl+T";       // 新建
pub const SHORTCUT_TAB_CLOSE: &str = "Ctrl+W";     // 关闭
pub const SHORTCUT_TAB_SWITCH: &str = "Ctrl+Tab";  // 切换
```

### 文件操作
```rust
pub const SHORTCUT_COPY: &str = "Ctrl+C";
pub const SHORTCUT_CUT: &str = "Ctrl+X";
pub const SHORTCUT_PASTE: &str = "Ctrl+V";
pub const SHORTCUT_DELETE: &str = "Delete";
pub const SHORTCUT_RENAME: &str = "F2";
pub const SHORTCUT_NEW_FOLDER: &str = "Ctrl+Shift+N";
```

### 导航
```rust
pub const SHORTCUT_BACK: &str = "Alt+Left";
pub const SHORTCUT_FORWARD: &str = "Alt+Right";
pub const SHORTCUT_UP: &str = "Alt+Up";
```

---

## 十、文件图标速查

```rust
pub fn icon_for_file(extension: &str) -> &'static str {
    match extension {
        "rs" => "\u{e7a8}",      // Rust
        "js" => "\u{e74e}",      // JavaScript
        "ts" => "\u{3b5}",       // TypeScript
        "tsx" | "jsx" => "\u{e7ba}", // React
        "py" => "\u{e73c}",      // Python
        "md" => "\u{f48a}",      // Markdown
        "txt" => "\u{f15c}",     // Text
        "png" | "jpg" | "gif" => "\u{f1c5}", // Image
        "mp4" | "avi" => "\u{f1c6}", // Video
        "mp3" | "wav" => "\u{f1c7}", // Audio
        "pdf" => "\u{f1c1}",     // PDF
        "zip" | "rar" => "\u{f1c2}", // Archive
        "exe" | "msi" => "\u{f1c0}", // Executable
        _ => "\u{f15c}",         // Default
    }
}
```

---

## 十一、侧边栏内容速查

### 此电脑
- 显示各磁盘及使用率进度条
- 进度条高度: 4px
- 颜色: Primary

### 标签
- 用户自定义的常用文件夹
- 支持添加/删除

### 最近访问
- 最近打开的文件/文件夹
- 显示访问时间

### 空间
- Space管理
- 支持切换和新建

---

## 十二、状态栏内容速查

### 左侧
- 分栏模式切换 (多面板/级联)
- Space空间切换

### 中间
- 面板数量
- 选中文件数
- 当前路径

### 右侧
- 分栏布局切换 (1-4分栏)

---

## 十三、动效规格速查

```rust
pub struct AnimationSpec {
    pub duration: u32 = 150,  // 100-200ms
    pub easing: &str = "ease-in-out",
}
```

### 用途
- 状态变化
- 展开/折叠
- 面板切换

---

## 十四、无障碍规格速查

```rust
pub struct AccessibilitySpec {
    pub contrast_ratio: f32 = 4.5,  // WCAG 2.1 AA
    pub focus_indicator: bool = true,
    pub keyboard_navigation: bool = true,
}
```

---

*文档版本: v1.0*  
*最后更新: 2026-09-01*

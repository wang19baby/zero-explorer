# Zero Explorer - 快速参考卡片

**用途**: 开发时快速查找参考来源

---

## 一、核心功能参考速查

| 功能 | 主要参考 | 次要参考 | 关键文件 |
|-----|---------|---------|---------|
| **GPU渲染** | MTT File Manager | FileMan | `gpu_backend.rs`, `main.rs` |
| **主题系统** | MTT File Manager | FileMan | `theme.rs` |
| **文件列表** | MTT File Manager | - | `file_list.rs` |
| **标签页** | MTT File Manager | - | `tab_bar.rs` |
| **面包屑** | MTT File Manager | - | `breadcrumb.rs` |
| **侧边栏** | MTT File Manager | DOpus | `sidebar.rs` |
| **状态栏** | MTT File Manager | - | `status_bar.rs` |
| **预览面板** | MTT File Manager | XYplorer | `preview_panel.rs` |
| **搜索** | MTT File Manager | Everything | `search.rs` |
| **拖拽** | MTT File Manager | - | `drag_drop.rs` |
| **地址栏** | MTT File Manager | - | `address_bar.rs` |
| **多面板** | Q-Dir | MTT File Manager | `panel_container.rs` |
| **分栏视图** | macOS Finder | - | `column_view.rs` |
| **画廊视图** | MTT File Manager | - | `gallery_view.rs` |
| **文件比较** | Total Commander | - | `file_compare.rs` |
| **远程协议** | FileMan | - | `ssh.rs`, `sftp.rs` |
| **插件系统** | Total Commander | - | `plugin.rs` |

---

## 二、依赖库参考速查

| 依赖 | 参考项目 | 用途 |
|-----|---------|------|
| `eframe` | MTT File Manager | UI框架 |
| `wgpu` | MTT File Manager | GPU渲染 |
| `tokio` | MTT File Manager | 异步运行时 |
| `walkdir` | MTT File Manager | 文件遍历 |
| `notify` | MTT File Manager | 文件监控 |
| `lru` | MTT File Manager | 缓存 |
| `image` | MTT File Manager | 图像处理 |
| `syntect` | FileMan | 语法高亮 |
| `zip/tar/flate2` | MTT File Manager | 归档支持 |
| `windows` | MTT File Manager | Windows API |
| `jwalk` | Nexus Explorer | 并行遍历 |
| `nucleo` | Nexus Explorer | 模糊搜索 |
| `ssh2` | FileMan | SSH/SFTP |

---

## 三、设计规范参考速查

### 颜色系统 (参考 MTT File Manager)
```rust
// 主色调
PRIMARY: #0078D4
PRIMARY_HOVER: #106EBE
PRIMARY_ACTIVE: #005A9E
PRIMARY_LIGHT: #E8F4FD

// 中性色
BG_BASE: #FFFFFF
BG_SECONDARY: #F9F9F9
BG_TERTIARY: #F3F3F3
BORDER: #E5E5E5
TEXT_PRIMARY: #1A1A1A
TEXT_SECONDARY: #616161
```

### 字体系统 (参考 MTT File Manager)
```rust
DISPLAY: "Segoe UI Variable Display"
BODY: "Segoe UI Variable Text"
MONO: "Cascadia Code"

SIZE_DISPLAY: 28px
SIZE_TITLE: 20px
SIZE_BODY: 14px
SIZE_CAPTION: 12px
SIZE_MONO: 13px
```

### 间距系统 (参考 MTT File Manager)
```rust
SPACE_1: 4px
SPACE_2: 8px
SPACE_3: 12px
SPACE_4: 16px
SPACE_5: 20px
SPACE_6: 24px
SPACE_8: 32px
```

### 组件规格 (参考 MTT File Manager)
```rust
// 按钮
HEIGHT: 32px
PADDING: 8px 16px
RADIUS: 4px

// 输入框
HEIGHT: 32px
PADDING: 0 12px
RADIUS: 4px

// 文件列表
ROW_HEIGHT: 36px
COLUMNS: 图标32px, 名称1fr, 类型120px, 大小80px, 时间140px
```

---

## 四、快捷键参考速查

### 全局 (参考 MTT File Manager)
```rust
SPACE: 预览面板
CTRL+SHIFT+B: 侧边栏
CTRL+L: 地址栏
```

### 标签页 (参考 MTT File Manager)
```rust
CTRL+T: 新建
CTRL+W: 关闭
CTRL+TAB: 切换
```

### 文件操作 (参考 Total Commander)
```rust
CTRL+C: 复制
CTRL+X: 移动
CTRL+V: 粘贴
DELETE: 删除
F2: 重命名
CTRL+SHIFT+N: 新建文件夹
```

### 导航 (参考 MTT File Manager)
```rust
ALT+LEFT: 后退
ALT+RIGHT: 前进
ALT+UP: 上级
```

---

## 五、实现要点速查

### GPU渲染 (参考 MTT File Manager)
- 多后端降级: DX12 → Vulkan → OpenGL
- DirectComposition呈现
- 纹理图集优化
- 批量Draw Call

### 虚拟化渲染 (参考 MTT File Manager)
- 只渲染可见行
- 批量更新: 100项或16ms
- LRU缓存已访问目录

### 异步操作 (参考 MTT File Manager + Nexus Explorer)
- 所有I/O在后台线程
- tokio异步文件操作
- 代际请求丢弃
- 分页加载

### 文件遍历 (参考 Nexus Explorer)
- jwalk并行遍历
- work-stealing策略
- 4x性能提升

### 搜索优化 (参考 Nexus Explorer)
- nucleo模糊搜索
- 8x搜索速度
- 实时过滤

---

## 六、竞品GitHub链接

| 项目 | 链接 | Stars |
|-----|------|-------|
| **MTT File Manager** | https://github.com/MTTamurex/MTT-File-Manager-RUST | 51 |
| **FileMan** | https://github.com/kvark/fileman | - |
| **Nexus Explorer** | https://github.com/Augani/nexus-explorer | 19 |
| **Filane** | https://github.com/alcatraz-alf/Filane | 3 |
| **BlazePilot** | https://github.com/Jhanfer/blazepilot | - |

---

*文档版本: v1.0*  
*最后更新: 2026-09-01*

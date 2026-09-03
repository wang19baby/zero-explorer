# Zero Explorer - 竞品分析与开发计划总结

**日期**: 2026-09-01  
**目标**: 1:1复现原型设计，制定可执行的开发计划

---

## 一、创建的文档

| 文档 | 路径 | 用途 |
|-----|------|------|
| **竞品分析与1:1复现开发计划** | `docs/implementation-plan.md` | 竞品技术分析 + 完整开发计划 |
| **每周执行计划** | `docs/weekly-execution-plan.md` | 按周分解的具体任务 |
| **设计规范速查表** | `docs/design-spec-quickref.md` | 开发时的快速参考 |

---

## 二、竞品技术经验总结

### 2.1 渲染架构
| 竞品 | 方案 | 可借鉴点 |
|-----|------|---------|
| **MTT File Manager** | egui + wgpu | 多后端降级策略 (DX12→Vulkan→OpenGL) |
| **Nexus Explorer** | GPUI | 并行遍历 (jwalk) + 模糊搜索 (nucleo) |
| **Tessoa** | 自研GPU | 任意分屏、Vim操作、彩色标签 |

### 2.2 性能优化
| 优化点 | 竞品方案 | Zero Explorer 实现 |
|-------|---------|-------------------|
| 目录遍历 | jwalk并行 | 引入jwalk，4x加速 |
| 文件索引 | tokio异步 | 已有异步基础 |
| 渲染优化 | 虚拟化 | 只渲染可见行 |
| 缓存策略 | LRU缓存 | 已访问目录缓存 |

### 2.3 功能参考
| 功能 | 竞品方案 | Zero Explorer 规格 |
|-----|---------|-------------------|
| 多面板 | Q-Dir四面板 | 1-4面板动态布局 |
| 标签页 | 每面板独立 | 独立管理Tab |
| 面包屑 | TC地址栏 | 双击切换输入框 |
| 文件列表 | 5列布局 | 图标/名称/类型/大小/时间 |
| 侧边栏 | DOpus侧栏 | 此电脑/标签/最近/空间 |
| 状态栏 | 模式切换 | 多面板/级联 + 布局切换 |

---

## 三、1:1复现核心规格

### 3.1 颜色系统
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

### 3.2 字体系统
```rust
// 字体家族
DISPLAY: "Segoe UI Variable Display"
BODY: "Segoe UI Variable Text"
MONO: "Cascadia Code"

// 字体大小
DISPLAY: 28px, TITLE: 20px, SUBTITLE: 16px
BODY: 14px, CAPTION: 12px, MONO: 13px
```

### 3.3 间距系统 (4px网格)
```rust
SPACE_1: 4px, SPACE_2: 8px, SPACE_3: 12px
SPACE_4: 16px, SPACE_5: 20px, SPACE_6: 24px, SPACE_8: 32px
```

### 3.4 组件规格
```rust
// 按钮
HEIGHT: 32px, PADDING: 8px 16px, RADIUS: 4px

// 输入框
HEIGHT: 32px, PADDING: 0 12px, RADIUS: 4px

// 文件列表
ROW_HEIGHT: 36px
COLUMNS: 图标32px, 名称1fr, 类型120px, 大小80px, 时间140px
```

### 3.5 布局结构
```
┌─────────────────────────────────────────────────────────────────┐
│  工具栏: [←后退] [↑上级] [⟲刷新] [面包屑] [🔍搜索] [☰视图] [⊞侧栏] [◧左|◨右] [⚙设置] │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┬──────────────┐                                │
│  │  面板 1      │  面板 2      │                                │
│  │  ┌────────┐ │  ┌────────┐ │                                │
│  │  │面包屑   │ │  │面包屑   │ │                                │
│  │  ├────────┤ │  ├────────┤ │                                │
│  │  │ 标签页  │ │  │ 标签页  │ │                                │
│  │  ├────────┤ │  ├────────┤ │                                │
│  │  │ 文件列表│ │  │ 文件列表│ │                                │
│  │  └────────┘ │  └────────┘ │                                │
│  └──────────────┴──────────────┘                                │
├─────────────────────────────────────────────────────────────────┤
│  状态栏: [多面板][级联] | [🏠默认][💼Work][💻Dev][+] | 4个面板·1个选中 | D:\...\ | [1][2][3][4] │
└─────────────────────────────────────────────────────────────────┘
```

---

## 四、开发计划概览

### Phase 0: 核心框架 (8周)
- Week 1: 项目初始化
- Week 2-3: GPU渲染管线
- Week 4: 文件系统抽象
- Week 5-6: 基础UI框架
- Week 7-8: 主题系统 + 单面板

### Phase 1: 核心功能 (14周)
- Week 9-10: 多面板布局
- Week 11-12: 标签页+面包屑
- Week 13-15: 文件列表+操作
- Week 16-17: 拖拽+地址栏
- Week 18-20: 侧边栏
- Week 21-22: 状态栏+主题+快捷键

### Phase 2: 增强功能 (12周)
- Week 23-24: 文件预览
- Week 25-26: 全局搜索
- Week 27-28: 文件标签
- Week 29-30: 批量重命名
- Week 31-32: Vim模式

### Phase 3: 高级功能 (16周)
- Week 33-34: 分栏视图
- Week 35-36: 画廊视图
- Week 37-38: 文件比较
- Week 39-41: 远程协议
- Week 42-44: 插件系统

---

## 五、关键里程碑

| 里程碑 | 时间 | 交付物 |
|-------|------|--------|
| **M1** | 第8周 | Windows可运行，GPU渲染正常，单面板+标签页 |
| **M2** | 第16周 | 多面板布局正常，文件操作完整 |
| **M3** | 第22周 | 侧边栏完整，**1:1还原设计原型** |
| **M4** | 第32周 | 文件预览+搜索+标签完整 |
| **M5** | 第44周 | 分栏/画廊+远程+插件完整 |

---

## 六、性能指标

| 指标 | 目标值 |
|-----|--------|
| 启动时间 | < 200ms |
| 10万文件响应 | < 100ms |
| 内存占用 | < 100MB |
| 帧率 | > 60fps |

---

## 七、下一步行动

### 立即行动 (本周)
1. **确认设计原型**：打开 `wireframes/design-preview.html` 确认视觉效果
2. **启动Phase 0 Week 1**：按照 `weekly-execution-plan.md` 执行
3. **代码仓库准备**：确保Git仓库干净，准备开始开发

### 开发环境准备
```bash
# 确保Rust工具链最新
rustup update

# 确保依赖安装
cargo install cargo-watch
cargo install cargo-expand

# 开始开发
cargo watch -x run
```

---

## 八、参考资源

### 设计文档
- `DESIGN.md` - 设计系统文档
- `docs/ui-design.md` - UI设计文档
- `docs/ux-interaction.md` - UX交互文档
- `wireframes/design-preview.html` - 设计预览页面

### 开发文档
- `docs/implementation-plan.md` - 竞品分析与开发计划
- `docs/weekly-execution-plan.md` - 每周执行计划
- `docs/design-spec-quickref.md` - 设计规范速查表
- `docs/development-plan.md` - 原始开发计划

### 竞品参考
- `competitive_analysis.md` - 竞品分析
- `product_analysis.md` - 产品分析

---

*文档版本: v1.0*  
*最后更新: 2026-09-01*

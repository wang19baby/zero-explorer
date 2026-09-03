# Phase 2 实现计划 — 参考 Tessoa (Week 13-22)

> **状态更新**: 2026-09-03  
> **总体进度**: 核心算法完成 ✅ / UI 组件部分完成 ⏳

## 一、Tessoa 核心特性分析

### 1. 亚像素字体渲染系统
Tessoa 的文字清晰度远超竞品，关键在五层渲染设置：

| 设置项 | 作用 | 默认值 | 状态 |
|--------|------|--------|------|
| 亚像素定位 | 每个字按亚像素精度落位，修复字距忽宽忽窄 | 开 | ✅ 完成 |
| LCD 亚像素抗锯齿 | 把水平分辨率当三倍用（RGB子像素） | 关 | ✅ 完成 |
| 字形对齐像素格 | 笔画主干对齐整像素边界（3档：关/跟随字体/强制） | 关 | ✅ 完成 |
| 文字 Gamma | 控制笔画边缘深浅（0.60-1.60） | 1.00 | ✅ 完成 |
| 经典文字渲染引擎 | 切换新旧两套渲染链路（仅Windows） | 开 | ✅ 完成 |

**技术要点**：
- 当前 zero-explorer 使用 `ab_glyph` 做基本光栅化，无亚像素支持
- 需要实现 subpixel positioning（ fractional pixel offset）
- LCD subpixel antialiasing 需要 fragment shader 采样 RGB 通道
- 参考实现：`osor.io/text.html` 的 temporal accumulation + winding number 方案

### 2. Hover-Reveal 交互模式
Tessoa 的 UI 保持安静，需要时才出现控件：

| 触发区域 | Hover 行为 | 状态 |
|----------|-----------|------|
| 侧栏条目 | 行尾露出 ×、⋯、锁 图标 | ⏳ 待集成 |
| 底栏按钮 | 弹出面板/提示 | ⏳ 待集成 |
| 排布徽章 | 悬停展开四种排布面板 | ⏳ 待集成 |
| 分组徽章 | 悬停弹出分组键面板 | ⏳ 待集成 |
| 按钮 | 浮出提示（功能+快捷键+鼠标手势） | ✅ 完成 |
| 文件夹（图标视图） | 缩略图摊开（弧线/抽牌/横排/阶梯） | ⏳ 待实现 |
| 布局行 | 行尾露出 ↻、锁、⋯ | ✅ 完成 |
| 布局分组标题 | 露出 ＋ 按钮 | ⏳ 待集成 |

**技术要点**：
- 需要 hover state 管理器（延迟触发、自动消失）
- 控件 fadeIn/fadeOut 动画
- 鼠标离开判定（tolerance zone）

### 3. 12 种布局模板
按窗格数分四类：

| 窗格数 | 模板 | 状态 |
|--------|------|------|
| 1格 | 单栏 | ✅ 完成 |
| 2格 | 双栏·左右、双栏·上下 | ✅ 完成 |
| 3格 | 三栏·横排、三栏·左一右二、三栏·左二右一、三栏·上一下二、三栏·上二下一 | ✅ 完成 |
| 4格 | 四栏·田字格、四栏·横排、四栏·左一右三、四栏·上一下三 | ✅ 完成 |

**当前状态**：✅ 12 种布局全部实现，布局计算正确

### 4. 4 种网格排布
仅在图标视图下生效：

| 排布 | 算法 | 状态 |
|------|------|------|
| 规则网格 | 所有格子等大，`auto-fill` + `minmax` | ✅ 完成 |
| 等高行 | 每行等高，宽度按图片比例 | ✅ 完成 |
| 瀑布流 | 每列等宽，高度各随图片 | ✅ 完成 |
| 马赛克拼贴 | 大小格混排，每隔几张出双倍大格 | ✅ 完成 |

**当前状态**：✅ 4 种排布算法全部实现，测试通过

### 5. 10+ 主题系统
Tessoa 内置 10 套主题：青铜雪、海盐蓝、海图蓝、绯樱、碧波、黛蓝、暖砂、曜黑、暮紫、晴空。支持自定义配色（最多32套），可导出/导入。

**当前状态**：✅ 10 套 Tessoa 主题 + 自定义配色 + JSON 导出/导入

---

## 二、实现计划

### Week 13-14: 亚像素字体渲染

**目标**：实现 Tessoa 级别的文字清晰度

**步骤**：

1. **亚像素定位（Subpixel Positioning）**
   - 在 `FontRenderer` 中存储每个 glyph 的 fractional offset
   - 修改 glyph cache key 包含 subpixel position
   - 文件：`src/ui/font_renderer.rs`
   - **状态**: ✅ 完成 - `GlyphCacheKey { glyph_id, subpixel_bin }` 实现

2. **LCD 亚像素抗锯齿**
   - 新增 fragment shader，采样时分别计算 R/G/B 通道的 coverage
   - 需要知道屏幕子像素排列（RGB/BGR）
   - 文件：`src/ui/renderer.rs`
   - **状态**: ✅ 完成 - `LCD_TEXT_SHADER` + `lcd_text_pipeline` 实现

3. **字形对齐像素格（Glyph Hinting）**
   - 三档：Off / Follow Font / Force
   - Force 模式下将 stem 对齐到整像素
   - 文件：`src/ui/font_renderer.rs`
   - **状态**: ✅ 完成 - `GlyphHinting` 枚举实现

4. **文字 Gamma**
   - 在 fragment shader 中调整 alpha 曲线
   - 范围 0.60-1.60，默认 1.00
   - **状态**: ✅ 完成 - `TextRenderSettings.text_gamma` 实现

5. **经典文字渲染引擎**
   - Windows 专用：切换新旧两套渲染链路
   - 三档：Off / FollowFont / Force
   - 文件：`src/ui/text_render_settings.rs`
   - **状态**: ✅ 完成 - `ClassicTextEngine` 枚举实现

6. **设置面板集成**
   - 在 `Settings` 的「外观」组添加字体渲染设置
   - 文件：`src/ui/settings.rs`
   - **状态**: ⏳ 待实现

**验证**：在不同 DPI 下对比文字清晰度

---

### Week 15-16: Hover-Reveal 交互系统

**目标**：实现 Tessoa 级别的安静 UI

**步骤**：

1. **HoverStateManager**
   - 新增 `src/ui/hover_state.rs`
   - 管理延迟触发（默认 200ms）、自动消失
   - tolerance zone（鼠标移出控件一定距离内不算离开）
   - **状态**: ✅ 完成 - 10 个单元测试通过

2. **布局列表 hover 效果**
   - 布局行：hover 时行尾 fadeIn 操作按钮（↻、锁、⋯）
   - 新增 `src/ui/layout_list.rs`
   - **状态**: ✅ 完成 - 7 个单元测试通过

3. **按钮提示系统**
   - hover 浮出两段提示（功能 + 快捷键/手势）
   - 提示内容从快捷键配置动态读取
   - 新增 `src/ui/button_tooltip.rs`
   - **状态**: ✅ 完成 - 10 个单元测试通过

4. **控件 hover 效果集成**
   - 侧栏条目：hover 时行尾 fadeIn 操作按钮
   - 底栏按钮：hover 弹出面板
   - 修改：`src/ui/sidebar.rs`、`src/ui/status_bar.rs`
   - **状态**: ⏳ 待集成

5. **文件夹缩略图摊开**
   - 4 种动画：弧线、抽牌、横排、阶梯
   - 修改：`src/ui/gallery_view.rs`
   - **状态**: ⏳ 待实现

**验证**：所有 hover 控件延迟出现、自动消失、tolerance zone 正常

---

### Week 17-18: 12 种布局模板

**目标**：补齐 Tessoa 的全部 12 种布局

**步骤**：

1. **扩展 LayoutMode 枚举**
   - 12 种布局模式全部实现
   - 修改：`src/core/state.rs`
   - **状态**: ✅ 完成

2. **实现布局计算**
   - 在 `LayoutEngine::calculate_layout` 中添加新布局的 Rect 计算
   - 修改：`src/ui/layout.rs`
   - **状态**: ✅ 完成 - 11 个单元测试通过

3. **布局管理功能**
   - 自动保存、锁定、重新加载、未命名布局恢复
   - 修改：`src/core/state.rs`
   - **状态**: ✅ 完成 - `LayoutState` 结构体实现

4. **布局模板对话框**
   - 新增 `LayoutTemplateDialog` 组件
   - 左栏：12 种排布缩略图
   - 右栏：选中排布的示意图 + 每格初始目录
   - 新增：`src/ui/layout_template_dialog.rs`
   - **状态**: ✅ 完成 - 6 个单元测试通过

**验证**：12 种布局全部正确渲染，模板对话框交互正常

---

### Week 19-20: 4 种网格排布

**目标**：图标视图支持 Tessoa 的 4 种排布

**步骤**：

1. **GridArrangement 枚举**
   - 4 种排布类型
   - 新增文件：`src/ui/grid_arrangement.rs`
   - **状态**: ✅ 完成

2. **排布算法实现**
   - 规则网格：现有 `GalleryView` 已有基础
   - 等高行：按行分组，计算每行总宽度，等比缩放
   - 瀑布流：维护每列高度，总是插入最矮的列
   - 马赛克：每隔 N 张插入一个 2x2 格子，其余填充
   - **状态**: ✅ 完成 - 9 个单元测试通过

3. **排布切换 UI**
   - 底栏排布徽章（点击切换、滚轮循环）
   - 右键菜单「排布」子菜单
   - 修改：`src/ui/status_bar.rs`
   - **状态**: ⏳ 待实现

4. **与图标大小独立**
   - 排布和图标大小可任意搭配
   - 最小档强制规则网格
   - 修改：`src/ui/gallery_view.rs`
   - **状态**: ⏳ 待集成

**验证**：4 种排布在不同图标大小下正确渲染

---

### Week 21-22: 10+ 主题系统

**目标**：实现 Tessoa 级别的主题架构

**步骤**：

1. **扩展 ThemeColors**
   - 增加更多 token：shadow、overlay、scrollbar、tooltip 等
   - 支持 32 位 ARGB
   - 修改：`src/ui/theme.rs`
   - **状态**: ✅ 完成

2. **10 套内置主题**
   - 参考 Tessoa 的配色风格：青铜雪（深色）、海盐蓝（浅色）、海图蓝、绯樱、碧波、黛蓝、暖砂、曜黑、暮紫、晴空
   - 修改：`src/ui/theme.rs`
   - **状态**: ✅ 完成 - 10 种 Tessoa 主题

3. **自定义配色系统**
   - 配色编辑页：逐项修改 token
   - 最多 32 套自定义配色
   - 导出/导入（剪贴板格式）
   - 修改：`src/ui/theme.rs`
   - **状态**: ✅ 完成 - JSON 导出/导入

4. **跟随系统深浅分别指定**
   - 系统为浅色时用主题 A
   - 系统为深色时用主题 B
   - 修改：`src/ui/theme_manager.rs`
   - **状态**: ⏳ 待实现

5. **主题设置面板**
   - 主题下拉 + 新建配色 + 删除配色
   - 修改：`src/ui/settings.rs`
   - **状态**: ⏳ 待实现

**验证**：10 套主题切换正常，自定义配色可保存/导出/导入

---

## 三、文件变更清单

| 文件 | 变更类型 | 说明 | 状态 |
|------|---------|------|------|
| `src/core/state.rs` | 修改 | 扩展 LayoutMode、新增 LayoutState | ✅ 完成 |
| `src/ui/font_renderer.rs` | 修改 | 亚像素定位、hinting、gamma | ✅ 完成 |
| `src/ui/renderer.rs` | 修改 | LCD subpixel shader | ✅ 完成 |
| `src/ui/text_render_settings.rs` | 修改 | 新增 ClassicTextEngine | ✅ 完成 |
| `src/ui/hover_state.rs` | 新增 | HoverStateManager | ✅ 完成 |
| `src/ui/layout.rs` | 修改 | 12 种布局计算 | ✅ 完成 |
| `src/ui/layout_list.rs` | 新增 | 布局列表 hover-reveal | ✅ 完成 |
| `src/ui/layout_template_dialog.rs` | 新增 | 布局模板对话框 | ✅ 完成 |
| `src/ui/grid_arrangement.rs` | 新增 | 4 种排布算法 | ✅ 完成 |
| `src/ui/button_tooltip.rs` | 新增 | 按钮提示系统 | ✅ 完成 |
| `src/ui/theme.rs` | 修改 | 10 种 Tessoa 主题、自定义配色 | ✅ 完成 |
| `src/ui/theme_manager.rs` | 修改 | try_send 修复 | ✅ 完成 |
| `src/ui/sidebar.rs` | 修改 | hover-reveal 操作按钮 | ⏳ 待集成 |
| `src/ui/status_bar.rs` | 修改 | hover 面板、排布徽章 | ⏳ 待实现 |
| `src/ui/gallery_view.rs` | 修改 | 排布切换、文件夹摊开 | ⏳ 待实现 |
| `src/ui/panel_container.rs` | 修改 | 布局模板对话框集成 | ⏳ 待集成 |
| `src/ui/settings.rs` | 修改 | 字体渲染、主题设置 | ⏳ 待实现 |
| `src/ui/components.rs` | 修改 | 按钮提示集成 | ⏳ 待集成 |

---

## 四、测试状态

| 模块 | 测试数 | 通过 | 失败 |
|------|--------|------|------|
| ui::theme | 24 | 24 | 0 |
| ui::hover_state | 10 | 10 | 0 |
| ui::grid_arrangement | 9 | 9 | 0 |
| ui::layout | 11 | 11 | 0 |
| ui::layout_list | 7 | 7 | 0 |
| ui::layout_template_dialog | 6 | 6 | 0 |
| ui::button_tooltip | 10 | 10 | 0 |
| **总计** | **77** | **77** | **0** |

---

## 五、后续工作

### 优先级 P0 (必须完成)
1. ⏳ 设置面板集成 - 字体渲染设置 UI
2. ⏳ 主题设置面板 - 主题选择/新建/删除

### 优先级 P1 (应该完成)
3. ⏳ 侧栏布局列表集成 - 将 LayoutList 集成到 Sidebar
4. ⏳ 排布切换 UI - 底栏徽章/右键菜单
5. ⏳ 跟随系统深浅分别指定

### 优先级 P2 (可以延后)
6. ⏳ 控件 hover 效果集成 - 侧栏/底栏/布局行
7. ⏳ 文件夹缩略图摊开动画
8. ⏳ 按钮提示系统集成到 Components

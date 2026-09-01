# Zero Explorer - TDD 测试设计

**日期**: 2026-09-01  
**目标**: 端到端功能验证，确保每个模块可独立测试

---

## 测试架构

```
tests/
├── unit/                    # 单元测试（模块内）
│   ├── ui/
│   │   ├── components_test.rs
│   │   ├── layout_test.rs
│   │   ├── panel_container_test.rs
│   │   ├── tab_bar_test.rs
│   │   ├── breadcrumb_test.rs
│   │   ├── file_list_test.rs
│   │   ├── address_bar_test.rs
│   │   ├── sidebar_test.rs
│   │   ├── status_bar_test.rs
│   │   └── theme_test.rs
│   ├── core/
│   │   ├── state_test.rs
│   │   ├── event_test.rs
│   │   └── shortcuts_test.rs
│   └── fs/
│       ├── file_system_test.rs
│       ├── file_operations_test.rs
│       └── path_utils_test.rs
├── integration/             # 集成测试（模块间交互）
│   ├── panel_layout_test.rs
│   ├── tab_navigation_test.rs
│   ├── file_operations_test.rs
│   └── theme_switch_test.rs
└── e2e/                     # 端到端测试（完整流程）
    ├── open_folder_test.rs
    ├── multi_panel_test.rs
    └── keyboard_shortcuts_test.rs
```

---

## 测试原则

### 1. 模块独立性
每个模块必须能独立测试，不依赖 GPU/窗口系统

### 2. Mock 策略
```rust
// Mock GPU 渲染器
struct MockRenderer {
    rendered_pixels: Vec<u32>,
}

// Mock 文件系统
struct MockFileSystem {
    files: HashMap<PathBuf, Vec<u8>>,
}
```

### 3. 测试命名规范
```rust
#[test]
fn test_panel_container_add_panel() { }

#[test]
fn test_tab_bar_close_last_tab_should_fail() { }

#[test]
fn test_file_list_select_multiple_with_ctrl() { }
```

---

## 核心模块测试用例

### 1. UI Components (`components_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains_point_inside() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert!(rect.contains(50.0, 25.0));
    }

    #[test]
    fn test_rect_contains_point_outside() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert!(!rect.contains(150.0, 25.0));
    }

    #[test]
    fn test_rect_intersection_overlapping() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection.x, 50.0);
        assert_eq!(intersection.y, 50.0);
        assert_eq!(intersection.width, 50.0);
        assert_eq!(intersection.height, 50.0);
    }

    #[test]
    fn test_rect_intersection_no_overlap() {
        let r1 = Rect::new(0.0, 0.0, 50.0, 50.0);
        let r2 = Rect::new(100.0, 100.0, 50.0, 50.0);
        assert!(r1.intersection(&r2).is_none());
    }

    #[test]
    fn test_button_state_transitions() {
        let mut button = Button::new("test", Rect::default());
        assert_eq!(*button.state(), ComponentState::Normal);
        
        button.handle_mouse_move(0.0, 0.0);
        assert_eq!(*button.state(), ComponentState::Hovered);
        
        button.handle_mouse_button_down(0.0, 0.0);
        assert_eq!(*button.state(), ComponentState::Pressed);
        
        button.handle_mouse_button_up(0.0, 0.0);
        assert_eq!(*button.state(), ComponentState::Hovered);
    }
}
```

### 2. Panel Container (`panel_container_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_container_new_single() {
        let container = PanelContainer::new();
        assert_eq!(container.panel_count(), 1);
    }

    #[test]
    fn test_panel_container_add_panel() {
        let mut container = PanelContainer::new();
        container.add_panel();
        assert_eq!(container.panel_count(), 2);
    }

    #[test]
    fn test_panel_container_max_panels() {
        let mut container = PanelContainer::new();
        for _ in 0..5 {
            container.add_panel();
        }
        assert_eq!(container.panel_count(), 4); // 最多4个
    }

    #[test]
    fn test_panel_container_remove_panel() {
        let mut container = PanelContainer::new();
        container.add_panel();
        container.remove_panel(0);
        assert_eq!(container.panel_count(), 1);
    }

    #[test]
    fn test_panel_container_remove_last_panel_should_fail() {
        let mut container = PanelContainer::new();
        let result = container.remove_panel(0);
        assert!(result.is_err()); // 至少保留1个
    }

    #[test]
    fn test_panel_divider_drag() {
        let mut container = PanelContainer::new();
        container.add_panel();
        
        let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
        container.set_bounds(bounds);
        
        // 拖拽分割线
        container.start_divider_drag(400.0, 300.0);
        container.update_divider_drag(450.0);
        container.end_divider_drag();
        
        let panels = container.panel_bounds();
        assert!(panels[0].width > 200.0); // 最小宽度
    }
}
```

### 3. Tab Bar (`tab_bar_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_new() {
        let tab_bar = TabBar::new();
        assert_eq!(tab_bar.tab_count(), 1); // 默认1个标签
    }

    #[test]
    fn test_tab_bar_add_tab() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab();
        assert_eq!(tab_bar.tab_count(), 2);
    }

    #[test]
    fn test_tab_bar_close_tab() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab();
        tab_bar.close_tab(0);
        assert_eq!(tab_bar.tab_count(), 1);
    }

    #[test]
    fn test_tab_bar_close_last_tab_should_fail() {
        let mut tab_bar = TabBar::new();
        let result = tab_bar.close_tab(0);
        assert!(result.is_err()); // 至少保留1个
    }

    #[test]
    fn test_tab_bar_switch_tab() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab();
        tab_bar.switch_tab(1);
        assert_eq!(tab_bar.active_tab(), Some(1));
    }

    #[test]
    fn test_tab_bar_drag_reorder() {
        let mut tab_bar = TabBar::new();
        tab_bar.add_tab();
        tab_bar.add_tab();
        
        tab_bar.start_drag(0, 100.0);
        tab_bar.update_drag(200.0);
        tab_bar.end_drag();
        
        // 验证顺序变化
    }
}
```

### 4. Breadcrumb (`breadcrumb_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breadcrumb_parse_path() {
        let mut breadcrumb = Breadcrumb::new();
        breadcrumb.set_path(PathBuf::from("C:/Users/test/Documents"));
        
        let segments = breadcrumb.segments();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].name, "C:");
        assert_eq!(segments[1].name, "Users");
        assert_eq!(segments[2].name, "Documents");
    }

    #[test]
    fn test_breadcrumb_click_segment() {
        let mut breadcrumb = Breadcrumb::new();
        breadcrumb.set_path(PathBuf::from("C:/Users/test/Documents"));
        
        let result = breadcrumb.click_segment(1); // 点击 "Users"
        assert_eq!(result, Some(PathBuf::from("C:/Users")));
    }

    #[test]
    fn test_breadcrumb_switch_to_input() {
        let mut breadcrumb = Breadcrumb::new();
        breadcrumb.set_path(PathBuf::from("C:/Users/test"));
        
        breadcrumb.switch_to_input();
        assert_eq!(breadcrumb.mode(), BreadcrumbMode::Input);
        assert_eq!(breadcrumb.input_value(), "C:\\Users\\test");
    }

    #[test]
    fn test_breadcrumb_confirm_input() {
        let mut breadcrumb = Breadcrumb::new();
        breadcrumb.set_path(PathBuf::from("C:/Users/test"));
        
        breadcrumb.switch_to_input();
        breadcrumb.set_input_value("D:/NewPath");
        let result = breadcrumb.confirm_input();
        
        assert!(result.is_ok());
        assert_eq!(breadcrumb.path(), PathBuf::from("D:/NewPath"));
    }
}
```

### 5. File List (`file_list_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_list_set_items() {
        let mut file_list = FileList::new();
        let items = vec![
            FileItem::new("file1.txt", FileType::File),
            FileItem::new("folder1", FileType::Directory),
        ];
        file_list.set_items(items);
        
        assert_eq!(file_list.item_count(), 2);
    }

    #[test]
    fn test_file_list_select_single() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::new("file1.txt", FileType::File),
            FileItem::new("file2.txt", FileType::File),
        ]);
        
        file_list.select(0, false);
        assert_eq!(file_list.selected_indices(), &[0]);
    }

    #[test]
    fn test_file_list_select_multiple_with_ctrl() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::new("file1.txt", FileType::File),
            FileItem::new("file2.txt", FileType::File),
            FileItem::new("file3.txt", FileType::File),
        ]);
        
        file_list.select(0, false);
        file_list.select(2, true); // Ctrl+Click
        assert_eq!(file_list.selected_indices(), &[0, 2]);
    }

    #[test]
    fn test_file_list_select_range_with_shift() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::new("file1.txt", FileType::File),
            FileItem::new("file2.txt", FileType::File),
            FileItem::new("file3.txt", FileType::File),
            FileItem::new("file4.txt", FileType::File),
        ]);
        
        file_list.select(0, false);
        file_list.select(3, true); // Shift+Click
        assert_eq!(file_list.selected_indices(), &[0, 1, 2, 3]);
    }

    #[test]
    fn test_file_list_sort_by_name() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::new("Charlie.txt", FileType::File),
            FileItem::new("Alpha.txt", FileType::File),
            FileItem::new("Bravo.txt", FileType::File),
        ]);
        
        file_list.sort_by(Column::Name, SortOrder::Ascending);
        let names: Vec<_> = file_list.items().iter().map(|i| i.name()).collect();
        assert_eq!(names, vec!["Alpha.txt", "Bravo.txt", "Charlie.txt"]);
    }

    #[test]
    fn test_file_list_sort_by_size() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::with_size("small.txt", FileType::File, 100),
            FileItem::with_size("large.txt", FileType::File, 1000),
            FileItem::with_size("medium.txt", FileType::File, 500),
        ]);
        
        file_list.sort_by(Column::Size, SortOrder::Descending);
        let sizes: Vec<_> = file_list.items().iter().map(|i| i.size()).collect();
        assert_eq!(sizes, vec![1000, 500, 100]);
    }

    #[test]
    fn test_file_list_keyboard_navigation() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::new("file1.txt", FileType::File),
            FileItem::new("file2.txt", FileType::File),
            FileItem::new("file3.txt", FileType::File),
        ]);
        
        file_list.handle_key_down(40); // Down arrow
        assert_eq!(file_list.selected_indices(), &[0]);
        
        file_list.handle_key_down(40); // Down arrow
        assert_eq!(file_list.selected_indices(), &[1]);
        
        file_list.handle_key_down(38); // Up arrow
        assert_eq!(file_list.selected_indices(), &[0]);
    }

    #[test]
    fn test_file_list_select_all() {
        let mut file_list = FileList::new();
        file_list.set_items(vec![
            FileItem::new("file1.txt", FileType::File),
            FileItem::new("file2.txt", FileType::File),
            FileItem::new("file3.txt", FileType::File),
        ]);
        
        file_list.handle_key_down(65); // Ctrl+A
        assert_eq!(file_list.selected_indices(), &[0, 1, 2]);
    }
}
```

### 6. Address Bar (`address_bar_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_bar_focus() {
        let mut address_bar = AddressBar::new();
        address_bar.set_path(PathBuf::from("C:/Users/test"));
        
        address_bar.focus();
        assert_eq!(address_bar.mode(), AddressBarMode::Input);
        assert_eq!(address_bar.input_value(), "C:\\Users\\test");
    }

    #[test]
    fn test_address_bar_blur() {
        let mut address_bar = AddressBar::new();
        address_bar.focus();
        address_bar.blur();
        
        assert_eq!(address_bar.mode(), AddressBarMode::Breadcrumb);
    }

    #[test]
    fn test_address_bar_go_back() {
        let mut address_bar = AddressBar::new();
        address_bar.set_path(PathBuf::from("C:/Users"));
        address_bar.set_path(PathBuf::from("C:/Users/test"));
        
        let prev = address_bar.go_back();
        assert_eq!(prev, Some(PathBuf::from("C:/Users")));
    }

    #[test]
    fn test_address_bar_go_forward() {
        let mut address_bar = AddressBar::new();
        address_bar.set_path(PathBuf::from("C:/Users"));
        address_bar.set_path(PathBuf::from("C:/Users/test"));
        address_bar.go_back();
        
        let next = address_bar.go_forward();
        assert_eq!(next, Some(PathBuf::from("C:/Users/test")));
    }

    #[test]
    fn test_address_bar_go_up() {
        let mut address_bar = AddressBar::new();
        address_bar.set_path(PathBuf::from("C:/Users/test"));
        
        let parent = address_bar.go_up();
        assert_eq!(parent, Some(PathBuf::from("C:/Users")));
    }

    #[test]
    fn test_address_bar_confirm_valid_path() {
        let mut address_bar = AddressBar::new();
        address_bar.set_path(PathBuf::from("C:/Users/test"));
        
        address_bar.focus();
        address_bar.handle_char_input('D');
        address_bar.handle_char_input(':');
        // ... 输入完整路径
        
        // 注意：需要 mock 文件系统来验证路径存在
    }

    #[test]
    fn test_address_bar_esc_cancel() {
        let mut address_bar = AddressBar::new();
        address_bar.set_path(PathBuf::from("C:/Users/test"));
        
        address_bar.focus();
        address_bar.handle_key_down(27); // Esc
        
        assert_eq!(address_bar.mode(), AddressBarMode::Breadcrumb);
        assert_eq!(address_bar.current_path(), &PathBuf::from("C:/Users/test"));
    }
}
```

### 7. Sidebar (`sidebar_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_toggle() {
        let mut sidebar = Sidebar::new();
        assert!(sidebar.is_visible());
        
        sidebar.toggle();
        assert!(!sidebar.is_visible());
        
        sidebar.toggle();
        assert!(sidebar.is_visible());
    }

    #[test]
    fn test_sidebar_resize() {
        let mut sidebar = Sidebar::new();
        let initial_width = sidebar.width();
        
        sidebar.set_width(initial_width + 50.0);
        assert!(sidebar.width() > initial_width);
    }

    #[test]
    fn test_sidebar_min_width() {
        let mut sidebar = Sidebar::new();
        sidebar.set_width(50.0); // 低于最小值
        assert!(sidebar.width() >= 150.0); // 最小150px
    }

    #[test]
    fn test_sidebar_max_width() {
        let mut sidebar = Sidebar::new();
        sidebar.set_width(500.0); // 超过最大值
        assert!(sidebar.width() <= 400.0); // 最大400px
    }

    #[test]
    fn test_sidebar_drag_resize() {
        let mut sidebar = Sidebar::new();
        let bounds = Rect::new(0.0, 0.0, 200.0, 600.0);
        sidebar.set_bounds(bounds);
        
        sidebar.start_drag(200.0);
        sidebar.update_drag(250.0);
        sidebar.end_drag();
        
        assert!(sidebar.width() > 200.0);
    }

    #[test]
    fn test_sidebar_select_item() {
        let mut sidebar = Sidebar::new();
        sidebar.select(0); // This PC
        assert_eq!(sidebar.selected_index(), Some(0));
    }
}
```

### 8. Status Bar (`status_bar_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_set_panel_count() {
        let mut status_bar = StatusBar::new();
        status_bar.set_panel_count(2);
        
        let text = status_bar.get_status_text();
        assert!(text.contains("2 panel(s)"));
    }

    #[test]
    fn test_status_bar_set_selected_count() {
        let mut status_bar = StatusBar::new();
        status_bar.set_selected_count(5);
        
        let text = status_bar.get_status_text();
        assert!(text.contains("5 selected"));
    }

    #[test]
    fn test_status_bar_cycle_layout() {
        let mut status_bar = StatusBar::new();
        assert_eq!(status_bar.layout(), &StatusBarLayout::SinglePanel);
        
        status_bar.cycle_layout();
        assert_eq!(status_bar.layout(), &StatusBarLayout::DualPanel);
        
        status_bar.cycle_layout();
        assert_eq!(status_bar.layout(), &StatusBarLayout::TriplePanel);
        
        status_bar.cycle_layout();
        assert_eq!(status_bar.layout(), &StatusBarLayout::QuadPanel);
        
        status_bar.cycle_layout();
        assert_eq!(status_bar.layout(), &StatusBarLayout::SinglePanel);
    }
}
```

### 9. Theme (`theme_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_dark() {
        let theme = Theme::dark();
        assert!(theme.colors.background.r < 0.5); // 深色背景
    }

    #[test]
    fn test_theme_light() {
        let theme = Theme::light();
        assert!(theme.colors.background.r > 0.5); // 浅色背景
    }

    #[test]
    fn test_theme_manager_toggle() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.mode(), ThemeMode::System);
        
        manager.toggle();
        assert_eq!(manager.mode(), ThemeMode::Light);
        
        manager.toggle();
        assert_eq!(manager.mode(), ThemeMode::Dark);
    }

    #[test]
    fn test_theme_manager_set_mode() {
        let mut manager = ThemeManager::new();
        
        manager.set_mode(ThemeMode::Light);
        assert_eq!(manager.theme().colors.background.r, Theme::light().colors.background.r);
        
        manager.set_mode(ThemeMode::Dark);
        assert_eq!(manager.theme().colors.background.r, Theme::dark().colors.background.r);
    }

    #[test]
    fn test_color_to_u32() {
        let color = Color::rgb(1.0, 0.0, 0.0); // 红色
        let u32_color = color.to_u32();
        assert_eq!(u32_color, 0xFFFF0000);
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex(0x00FF00); // 绿色
        assert!(color.r < 0.01);
        assert!(color.g > 0.99);
        assert!(color.b < 0.01);
    }
}
```

### 10. Shortcuts (`shortcuts_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_display() {
        let shortcut = Shortcut::ctrl(67); // Ctrl+C
        assert_eq!(shortcut.display(), "Ctrl+C");
    }

    #[test]
    fn test_shortcut_ctrl_shift() {
        let shortcut = Shortcut::ctrl_shift(78); // Ctrl+Shift+N
        assert_eq!(shortcut.display(), "Ctrl+Shift+N");
    }

    #[test]
    fn test_shortcut_manager_register() {
        let mut manager = ShortcutManager::new();
        let shortcut = Shortcut::ctrl(80); // Ctrl+P
        
        manager.register(shortcut, ShortcutAction::Custom("print".to_string()));
        
        let action = manager.get_action(&shortcut);
        assert!(action.is_some());
    }

    #[test]
    fn test_shortcut_manager_match() {
        let manager = ShortcutManager::new();
        
        let action = manager.matches(true, false, false, 67); // Ctrl+C
        assert_eq!(action, Some(&ShortcutAction::Copy));
    }

    #[test]
    fn test_shortcut_manager_unregister() {
        let mut manager = ShortcutManager::new();
        let shortcut = Shortcut::ctrl(67);
        
        manager.unregister(&shortcut);
        
        let action = manager.get_action(&shortcut);
        assert!(action.is_none());
    }
}
```

---

## 集成测试用例

### 1. Panel Layout Integration (`panel_layout_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_container_with_layout_engine() {
        let mut container = PanelContainer::new();
        container.add_panel();
        
        let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
        let layout_constraint = LayoutConstraint::default();
        
        let panel_bounds = LayoutEngine::calculate_layout(
            LayoutMode::DualVertical,
            &bounds,
            &layout_constraint,
            false,
            0.0,
        );
        
        assert_eq!(panel_bounds.len(), 2);
        assert!(panel_bounds[0].width > 200.0);
        assert!(panel_bounds[1].width > 200.0);
    }

    #[test]
    fn test_panel_with_sidebar() {
        let mut container = PanelContainer::new();
        let mut sidebar = Sidebar::new();
        
        sidebar.set_width(250.0);
        
        let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
        let available_width = bounds.width - sidebar.width() - 1.0;
        
        let panel_bounds = LayoutEngine::calculate_layout(
            LayoutMode::Single,
            &Rect::new(0.0, 0.0, available_width, bounds.height),
            &LayoutConstraint::default(),
            true,
            sidebar.width(),
        );
        
        assert_eq!(panel_bounds.len(), 1);
        assert!(panel_bounds[0].width > 400.0);
    }
}
```

### 2. File Operations Integration (`file_operations_test.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_paste_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");
        
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::create_dir(&dst_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        ops.copy(vec![src_dir.join("test.txt")]);
        let result = ops.paste(&dst_dir);
        
        assert!(result.success);
        assert!(dst_dir.join("test.txt").exists());
    }

    #[test]
    fn test_move_files() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");
        
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::create_dir(&dst_dir).unwrap();
        std::fs::write(src_dir.join("test.txt"), "hello").unwrap();
        
        let mut ops = FileOperations::new();
        ops.cut(vec![src_dir.join("test.txt")]);
        let result = ops.paste(&dst_dir);
        
        assert!(result.success);
        assert!(!src_dir.join("test.txt").exists());
        assert!(dst_dir.join("test.txt").exists());
    }

    #[test]
    fn test_delete_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();
        
        let ops = FileOperations::new();
        let result = ops.delete(&[file_path.clone()]);
        
        assert!(result.success);
        assert!(!file_path.exists());
    }

    #[test]
    fn test_rename_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("old_name.txt");
        std::fs::write(&file_path, "hello").unwrap();
        
        let ops = FileOperations::new();
        let result = ops.rename(&file_path, "new_name.txt");
        
        assert!(result.success);
        assert!(!file_path.exists());
        assert!(temp_dir.path().join("new_name.txt").exists());
    }

    #[test]
    fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_folder");
        
        let ops = FileOperations::new();
        let result = ops.create_dir(temp_dir.path(), "new_folder");
        
        assert!(result.success);
        assert!(new_dir.exists());
    }
}
```

---

## 测试运行命令

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test ui::components_test
cargo test ui::panel_container_test

# 运行集成测试
cargo test --test integration

# 运行端到端测试
cargo test --test e2e

# 显示测试输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_panel_container_add_panel
```

---

## 测试覆盖率目标

| 模块 | 目标覆盖率 | 优先级 |
|------|-----------|--------|
| UI Components | 90% | 高 |
| Panel Container | 95% | 高 |
| Tab Bar | 90% | 高 |
| Breadcrumb | 85% | 中 |
| File List | 95% | 高 |
| Address Bar | 85% | 中 |
| Sidebar | 80% | 中 |
| Status Bar | 80% | 中 |
| Theme | 90% | 高 |
| Shortcuts | 95% | 高 |
| File Operations | 95% | 高 |

---

## 下一步

1. 为现有模块添加单元测试
2. 创建 mock 模块（MockRenderer, MockFileSystem）
3. 编写集成测试
4. 配置 CI 自动运行测试

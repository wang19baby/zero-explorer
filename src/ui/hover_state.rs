use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 悬浮状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverState {
    /// 未悬浮
    Idle,
    /// 等待延迟（鼠标已进入，但还没到触发时间）
    Waiting,
    /// 已激活（延迟已过，控件可见）
    Active,
    /// 退出中（鼠标已离开，等待 tolerance 判定）
    Leaving,
}

/// 悬浮条目
#[derive(Debug, Clone)]
struct HoverEntry {
    state: HoverState,
    entered_at: Instant,
    left_at: Option<Instant>,
    /// 自定义数据（如按钮ID、区域ID等）
    data: u64,
}

/// 悬浮管理器 - 管理 UI 元素的 hover-reveal 行为
/// 参考 Tessoa 的 hover-reveal 交互模式
pub struct HoverStateManager {
    /// 默认延迟时间
    default_delay: Duration,
    /// tolerance 时间（鼠标离开后多久内算"还在"）
    tolerance: Duration,
    /// 各元素的悬浮状态
    entries: HashMap<u64, HoverEntry>,
}

impl HoverStateManager {
    /// 创建悬浮管理器
    pub fn new() -> Self {
        Self {
            default_delay: Duration::from_millis(200),
            tolerance: Duration::from_millis(100),
            entries: HashMap::new(),
        }
    }

    /// 创建带自定义延迟的悬浮管理器
    pub fn with_delay(delay_ms: u64) -> Self {
        Self {
            default_delay: Duration::from_millis(delay_ms),
            tolerance: Duration::from_millis(100),
            entries: HashMap::new(),
        }
    }

    /// 鼠标进入某个元素
    pub fn enter(&mut self, id: u64) {
        let now = Instant::now();
        
        if let Some(entry) = self.entries.get_mut(&id) {
            // 如果正在离开状态，取消离开
            if entry.state == HoverState::Leaving {
                entry.state = HoverState::Active;
                entry.left_at = None;
                return;
            }
            // 如果已经是 Waiting 或 Active，不重复处理
            if entry.state == HoverState::Waiting || entry.state == HoverState::Active {
                return;
            }
        }

        // 新建或重置条目
        self.entries.insert(id, HoverEntry {
            state: HoverState::Waiting,
            entered_at: now,
            left_at: None,
            data: 0,
        });
    }

    /// 鼠标离开某个元素
    pub fn leave(&mut self, id: u64) {
        let now = Instant::now();
        
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.state = HoverState::Leaving;
            entry.left_at = Some(now);
        }
    }

    /// 检查某个元素是否应该显示（已激活）
    pub fn is_active(&self, id: u64) -> bool {
        if let Some(entry) = self.entries.get(&id) {
            entry.state == HoverState::Active
        } else {
            false
        }
    }

    /// 获取某个元素的当前状态
    pub fn state(&self, id: u64) -> HoverState {
        self.entries.get(&id)
            .map(|e| e.state)
            .unwrap_or(HoverState::Idle)
    }

    /// 更新状态（每帧调用）
    /// 返回需要通知的元素列表（状态从 Waiting->Active 或 Active->Idle 的变化）
    pub fn update(&mut self) -> Vec<(u64, HoverState)> {
        let now = Instant::now();
        let mut notifications = Vec::new();

        for (id, entry) in self.entries.iter_mut() {
            match entry.state {
                HoverState::Waiting => {
                    // 检查延迟是否已过
                    if now.duration_since(entry.entered_at) >= self.default_delay {
                        entry.state = HoverState::Active;
                        notifications.push((*id, HoverState::Active));
                    }
                }
                HoverState::Leaving => {
                    // 检查 tolerance 时间是否已过
                    if let Some(left_at) = entry.left_at {
                        if now.duration_since(left_at) >= self.tolerance {
                            entry.state = HoverState::Idle;
                            notifications.push((*id, HoverState::Idle));
                        }
                    }
                }
                _ => {}
            }
        }

        // 清理 Idle 状态的条目
        self.entries.retain(|_, entry| entry.state != HoverState::Idle);

        notifications
    }

    /// 强制激活某个元素（跳过延迟）
    pub fn force_activate(&mut self, id: u64) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.state = HoverState::Active;
        } else {
            self.entries.insert(id, HoverEntry {
                state: HoverState::Active,
                entered_at: Instant::now(),
                left_at: None,
                data: 0,
            });
        }
    }

    /// 强制隐藏某个元素
    pub fn force_hide(&mut self, id: u64) {
        self.entries.remove(&id);
    }

    /// 清除所有状态
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 设置自定义数据
    pub fn set_data(&mut self, id: u64, data: u64) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.data = data;
        }
    }

    /// 获取自定义数据
    pub fn data(&self, id: u64) -> Option<u64> {
        self.entries.get(&id).map(|e| e.data)
    }
}

impl Default for HoverStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 悬浮区域 - 用于检测鼠标是否在某个矩形区域内
#[derive(Debug, Clone, Copy)]
pub struct HoverRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl HoverRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// 检查点是否在区域内
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width
            && py >= self.y && py <= self.y + self.height
    }

    /// 检查点是否在区域内（带 tolerance）
    pub fn contains_with_tolerance(&self, px: f32, py: f32, tolerance: f32) -> bool {
        px >= self.x - tolerance && px <= self.x + self.width + tolerance
            && py >= self.y - tolerance && py <= self.y + self.height + tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_hover_state_manager_new() {
        let manager = HoverStateManager::new();
        assert_eq!(manager.entries.len(), 0);
    }

    #[test]
    fn test_hover_state_manager_with_delay() {
        let manager = HoverStateManager::with_delay(300);
        assert_eq!(manager.default_delay, Duration::from_millis(300));
    }

    #[test]
    fn test_hover_enter_and_active() {
        let mut manager = HoverStateManager::with_delay(50);
        
        manager.enter(1);
        assert_eq!(manager.state(1), HoverState::Waiting);
        assert!(!manager.is_active(1));
        
        // 等待延迟
        thread::sleep(Duration::from_millis(60));
        manager.update();
        
        assert_eq!(manager.state(1), HoverState::Active);
        assert!(manager.is_active(1));
    }

    #[test]
    fn test_hover_leave_and_tolerance() {
        let mut manager = HoverStateManager::with_delay(50);
        
        manager.enter(1);
        thread::sleep(Duration::from_millis(60));
        manager.update();
        assert!(manager.is_active(1));
        
        // 离开
        manager.leave(1);
        assert_eq!(manager.state(1), HoverState::Leaving);
        
        // tolerance 时间内重新进入
        manager.enter(1);
        assert_eq!(manager.state(1), HoverState::Active);
    }

    #[test]
    fn test_hover_force_activate() {
        let mut manager = HoverStateManager::new();
        
        manager.force_activate(1);
        assert!(manager.is_active(1));
        assert_eq!(manager.state(1), HoverState::Active);
    }

    #[test]
    fn test_hover_force_hide() {
        let mut manager = HoverStateManager::new();
        
        manager.force_activate(1);
        assert!(manager.is_active(1));
        
        manager.force_hide(1);
        assert!(!manager.is_active(1));
        assert_eq!(manager.state(1), HoverState::Idle);
    }

    #[test]
    fn test_hover_clear() {
        let mut manager = HoverStateManager::new();
        
        manager.force_activate(1);
        manager.force_activate(2);
        assert_eq!(manager.entries.len(), 2);
        
        manager.clear();
        assert_eq!(manager.entries.len(), 0);
    }

    #[test]
    fn test_hover_rect_contains() {
        let rect = HoverRect::new(10.0, 10.0, 100.0, 50.0);
        
        assert!(rect.contains(50.0, 30.0)); // 内部
        assert!(rect.contains(10.0, 10.0)); // 左上角
        assert!(rect.contains(110.0, 60.0)); // 右下角
        assert!(!rect.contains(5.0, 5.0)); // 外部
        assert!(!rect.contains(115.0, 65.0)); // 外部
    }

    #[test]
    fn test_hover_rect_contains_with_tolerance() {
        let rect = HoverRect::new(10.0, 10.0, 100.0, 50.0);
        
        // tolerance 为 5 时，边缘外 5 像素内也算
        assert!(rect.contains_with_tolerance(5.0, 30.0, 5.0));
        assert!(rect.contains_with_tolerance(115.0, 30.0, 5.0));
        assert!(!rect.contains_with_tolerance(0.0, 30.0, 5.0)); // 超出 tolerance
    }

    #[test]
    fn test_hover_set_data() {
        let mut manager = HoverStateManager::new();
        
        manager.force_activate(1);
        manager.set_data(1, 42);
        
        assert_eq!(manager.data(1), Some(42));
    }
}

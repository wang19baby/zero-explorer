use crossbeam_channel::{bounded, Receiver, Sender};

/// 主题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    Light,
    Dark,
    System,
}

/// 主题颜色
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // 背景色
    pub bg_primary: [f32; 4],      // 主背景
    pub bg_secondary: [f32; 4],    // 次背景
    pub bg_tertiary: [f32; 4],     // 第三背景
    pub bg_hover: [f32; 4],        // 悬停背景
    pub bg_selected: [f32; 4],     // 选中背景
    pub bg_active: [f32; 4],       // 活跃背景

    // 前景色
    pub fg_primary: [f32; 4],      // 主前景
    pub fg_secondary: [f32; 4],    // 次前景
    pub fg_disabled: [f32; 4],     // 禁用前景

    // 边框色
    pub border: [f32; 4],          // 边框
    pub border_focus: [f32; 4],    // 聚焦边框

    // 强调色
    pub accent: [f32; 4],          // 品牌色
    pub accent_hover: [f32; 4],    // 品牌色悬停

    // 状态色
    pub success: [f32; 4],         // 成功 (绿色)
    pub warning: [f32; 4],         // 警告 (黄色)
    pub error: [f32; 4],           // 错误 (红色)
    pub info: [f32; 4],            // 信息 (蓝色)

    // 特殊色
    pub shadow: [f32; 4],          // 阴影
    pub overlay: [f32; 4],         // 遮罩
}

impl ThemeColors {
    /// 深色主题
    pub fn dark() -> Self {
        Self {
            bg_primary: [0.102, 0.102, 0.102, 1.0],      // #1A1A1A
            bg_secondary: [0.133, 0.133, 0.133, 1.0],    // #222222
            bg_tertiary: [0.165, 0.165, 0.165, 1.0],     // #2A2A2A
            bg_hover: [0.196, 0.196, 0.196, 1.0],        // #333333
            bg_selected: [0.200, 0.200, 0.220, 1.0],     // #333338 (hover - subtle)
            bg_active: [0.310, 0.510, 0.749, 1.0],       // #4F82BF (selection - blue)

            fg_primary: [0.937, 0.937, 0.937, 1.0],      // #EFEFEF
            fg_secondary: [0.690, 0.690, 0.690, 1.0],    // #B0B0B0
            fg_disabled: [0.400, 0.400, 0.400, 1.0],     // #666666

            border: [0.251, 0.251, 0.251, 1.0],          // #404040
            border_focus: [0.310, 0.510, 0.749, 1.0],    // #4F82BF

            accent: [0.310, 0.510, 0.749, 1.0],          // #4F82BF
            accent_hover: [0.376, 0.576, 0.816, 1.0],    // #6093D0

            success: [0.298, 0.686, 0.384, 1.0],         // #4CB062
            warning: [0.784, 0.620, 0.176, 1.0],         // #C89E2D
            error: [0.784, 0.224, 0.224, 1.0],           // #C83939
            info: [0.251, 0.565, 0.784, 1.0],            // #4090C8

            shadow: [0.0, 0.0, 0.0, 0.3],
            overlay: [0.0, 0.0, 0.0, 0.5],
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            bg_primary: [0.961, 0.961, 0.961, 1.0],      // #F5F5F5
            bg_secondary: [0.937, 0.937, 0.937, 1.0],    // #EFEFEF
            bg_tertiary: [0.914, 0.914, 0.914, 1.0],     // #E9E9E9
            bg_hover: [0.890, 0.890, 0.890, 1.0],        // #E3E3E3
            bg_selected: [0.880, 0.920, 0.960, 1.0],     // #E0EBF5 (hover - subtle)
            bg_active: [0.204, 0.482, 0.706, 1.0],       // #347BB4 (selection - blue)

            fg_primary: [0.102, 0.102, 0.102, 1.0],      // #1A1A1A
            fg_secondary: [0.400, 0.400, 0.400, 1.0],    // #666666
            fg_disabled: [0.690, 0.690, 0.690, 1.0],     // #B0B0B0

            border: [0.827, 0.827, 0.827, 1.0],          // #D3D3D3
            border_focus: [0.251, 0.565, 0.784, 1.0],    // #4090C8

            accent: [0.251, 0.565, 0.784, 1.0],          // #4090C8
            accent_hover: [0.204, 0.482, 0.706, 1.0],    // #347BB4

            success: [0.224, 0.565, 0.298, 1.0],         // #39904B
            warning: [0.686, 0.537, 0.137, 1.0],         // #AE8923
            error: [0.686, 0.169, 0.169, 1.0],           // #AE2C2C
            info: [0.176, 0.478, 0.686, 1.0],            // #2D7AAE

            shadow: [0.0, 0.0, 0.0, 0.1],
            overlay: [0.0, 0.0, 0.0, 0.3],
        }
    }
}

/// 主题管理器
pub struct ThemeManager {
    /// 当前主题类型
    theme_type: ThemeType,
    /// 当前颜色
    colors: ThemeColors,
    /// 颜色变更通知通道
    sender: Sender<ThemeColors>,
    receiver: Receiver<ThemeColors>,
}

impl ThemeManager {
    pub fn new(initial: ThemeType) -> Self {
        let (sender, receiver) = bounded(1);
        let colors = match initial {
            ThemeType::Light => ThemeColors::light(),
            ThemeType::Dark => ThemeColors::dark(),
            ThemeType::System => {
                if crate::ui::gpu_backend::detect_system_theme() {
                    ThemeColors::light()
                } else {
                    ThemeColors::dark()
                }
            }
        };

        Self {
            theme_type: initial,
            colors,
            sender,
            receiver,
        }
    }

    /// 获取当前颜色
    pub fn colors(&self) -> &ThemeColors {
        &self.colors
    }

    /// 获取当前主题类型
    pub fn theme_type(&self) -> ThemeType {
        self.theme_type
    }

    /// 设置主题
    pub fn set_theme(&mut self, theme_type: ThemeType) {
        self.theme_type = theme_type;
        self.update_colors();
    }

    /// 切换深色/浅色
    pub fn toggle(&mut self) {
        self.theme_type = match self.theme_type {
            ThemeType::Light => ThemeType::Dark,
            ThemeType::Dark => ThemeType::Light,
            ThemeType::System => {
                if crate::ui::gpu_backend::detect_system_theme() {
                    ThemeType::Dark
                } else {
                    ThemeType::Light
                }
            }
        };
        self.update_colors();
    }

    /// 检查并应用系统主题变更
    pub fn check_system_theme(&mut self) {
        if self.theme_type == ThemeType::System {
            self.update_colors();
        }
    }

    /// 尝试接收主题变更 (非阻塞)
    pub fn try_receive(&self) -> Option<ThemeColors> {
        self.receiver.try_recv().ok()
    }

    /// 更新颜色
    fn update_colors(&mut self) {
        self.colors = match self.theme_type {
            ThemeType::Light => ThemeColors::light(),
            ThemeType::Dark => ThemeColors::dark(),
            ThemeType::System => {
                if crate::ui::gpu_backend::detect_system_theme() {
                    ThemeColors::light()
                } else {
                    ThemeColors::dark()
                }
            }
        };

        // 通知监听者 (非阻塞)
        let _ = self.sender.try_send(self.colors.clone());
    }

    /// 将颜色转换为线性空间 (用于GPU渲染)
    pub fn to_linear(&self, color: [f32; 4]) -> [f32; 4] {
        // sRGB → Linear
        [
            Self::srgb_to_linear(color[0]),
            Self::srgb_to_linear(color[1]),
            Self::srgb_to_linear(color[2]),
            color[3], // Alpha不需要转换
        ]
    }

    /// sRGB → Linear
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new(ThemeType::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        let dark = ThemeColors::dark();
        let light = ThemeColors::light();

        // 深色背景应该比浅色暗
        assert!(dark.bg_primary[0] < light.bg_primary[0]);
    }

    #[test]
    fn test_theme_toggle() {
        let mut manager = ThemeManager::new(ThemeType::Dark);
        assert_eq!(manager.theme_type(), ThemeType::Dark);

        manager.toggle();
        assert_eq!(manager.theme_type(), ThemeType::Light);

        manager.toggle();
        assert_eq!(manager.theme_type(), ThemeType::Dark);
    }

    #[test]
    fn test_srgb_to_linear() {
        // 0.0 → 0.0
        assert_eq!(ThemeManager::srgb_to_linear(0.0), 0.0);
        // 1.0 → 1.0
        assert!((ThemeManager::srgb_to_linear(1.0) - 1.0).abs() < 0.001);
    }
}

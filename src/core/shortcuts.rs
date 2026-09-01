use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: u32,
}

impl Shortcut {
    pub const fn new(key: u32) -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            key,
        }
    }

    pub const fn ctrl(key: u32) -> Self {
        Self {
            ctrl: true,
            shift: false,
            alt: false,
            key,
        }
    }

    pub const fn shift(key: u32) -> Self {
        Self {
            ctrl: false,
            shift: true,
            alt: false,
            key,
        }
    }

    pub const fn alt(key: u32) -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: true,
            key,
        }
    }

    pub const fn ctrl_shift(key: u32) -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
            key,
        }
    }

    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        parts.push(self.key_to_string());
        parts.join("+")
    }

    fn key_to_string(&self) -> String {
        match self.key {
            8 => "Backspace".to_string(),
            9 => "Tab".to_string(),
            13 => "Enter".to_string(),
            16 => "Shift".to_string(),
            17 => "Ctrl".to_string(),
            18 => "Alt".to_string(),
            19 => "Pause".to_string(),
            20 => "CapsLock".to_string(),
            27 => "Esc".to_string(),
            32 => "Space".to_string(),
            33 => "PageUp".to_string(),
            34 => "PageDown".to_string(),
            35 => "End".to_string(),
            36 => "Home".to_string(),
            37 => "Left".to_string(),
            38 => "Up".to_string(),
            39 => "Right".to_string(),
            40 => "Down".to_string(),
            45 => "Insert".to_string(),
            46 => "Delete".to_string(),
            48..=57 => format!("{}", self.key - 48),
            65..=90 => char::from_u32(self.key).unwrap_or('?').to_string(),
            112..=123 => format!("F{}", self.key - 111),
            _ => format!("Key{}", self.key),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShortcutAction {
    NavigateBack,
    NavigateForward,
    NavigateUp,
    Refresh,
    Copy,
    Cut,
    Paste,
    Delete,
    Rename,
    NewFolder,
    SelectAll,
    ToggleSidebar,
    ToggleHidden,
    ActivateAddressBar,
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    LayoutSingle,
    LayoutDualVertical,
    LayoutDualHorizontal,
    LayoutTripleLeft,
    LayoutTripleRight,
    LayoutQuad,
    LayoutCascade,
    Custom(String),
}

pub struct ShortcutManager {
    shortcuts: HashMap<Shortcut, ShortcutAction>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self {
            shortcuts: HashMap::new(),
        };
        manager.register_defaults();
        manager
    }

    fn register_defaults(&mut self) {
        self.register(Shortcut::alt(37), ShortcutAction::NavigateBack);
        self.register(Shortcut::alt(39), ShortcutAction::NavigateForward);
        self.register(Shortcut::alt(38), ShortcutAction::NavigateUp);
        self.register(Shortcut::new(116), ShortcutAction::Refresh);
        self.register(Shortcut::ctrl(67), ShortcutAction::Copy);
        self.register(Shortcut::ctrl(88), ShortcutAction::Cut);
        self.register(Shortcut::ctrl(86), ShortcutAction::Paste);
        self.register(Shortcut::new(46), ShortcutAction::Delete);
        self.register(Shortcut::new(113), ShortcutAction::Rename);
        self.register(Shortcut::ctrl_shift(78), ShortcutAction::NewFolder);
        self.register(Shortcut::ctrl(65), ShortcutAction::SelectAll);
        self.register(Shortcut::ctrl_shift(66), ShortcutAction::ToggleSidebar);
        self.register(Shortcut::ctrl(76), ShortcutAction::ActivateAddressBar);
        self.register(Shortcut::ctrl(84), ShortcutAction::NewTab);
        self.register(Shortcut::ctrl(87), ShortcutAction::CloseTab);
        self.register(Shortcut::ctrl(9), ShortcutAction::NextTab);
        self.register(Shortcut::ctrl_shift(9), ShortcutAction::PreviousTab);
    }

    pub fn register(&mut self, shortcut: Shortcut, action: ShortcutAction) {
        self.shortcuts.insert(shortcut, action);
    }

    pub fn unregister(&mut self, shortcut: &Shortcut) -> Option<ShortcutAction> {
        self.shortcuts.remove(shortcut)
    }

    pub fn get_action(&self, shortcut: &Shortcut) -> Option<&ShortcutAction> {
        self.shortcuts.get(shortcut)
    }

    pub fn get_shortcut(&self, action: &ShortcutAction) -> Option<&Shortcut> {
        self.shortcuts.iter().find(|(_, a)| *a == action).map(|(s, _)| s)
    }

    pub fn all_shortcuts(&self) -> &HashMap<Shortcut, ShortcutAction> {
        &self.shortcuts
    }

    pub fn matches(
        &self,
        ctrl: bool,
        shift: bool,
        alt: bool,
        key: u32,
    ) -> Option<&ShortcutAction> {
        let shortcut = Shortcut {
            ctrl,
            shift,
            alt,
            key,
        };
        self.get_action(&shortcut)
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl VimMode {
    pub fn name(&self) -> &str {
        match self {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Visual => "VISUAL",
            VimMode::Command => "COMMAND",
        }
    }
}

impl Default for VimMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone)]
pub struct VimState {
    pub mode: VimMode,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_offset: usize,
    pub repeat_count: u32,
    pub command_buffer: String,
    pub last_command: Option<String>,
    pub visual_start: Option<(usize, usize)>,
}

impl VimState {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            repeat_count: 0,
            command_buffer: String::new(),
            last_command: None,
            visual_start: None,
        }
    }

    pub fn reset(&mut self) {
        self.mode = VimMode::Normal;
        self.repeat_count = 0;
        self.command_buffer.clear();
        self.visual_start = None;
    }

    pub fn enter_insert(&mut self) {
        self.mode = VimMode::Insert;
    }

    pub fn enter_visual(&mut self) {
        self.mode = VimMode::Visual;
        self.visual_start = Some((self.cursor_x, self.cursor_y));
    }

    pub fn enter_command(&mut self) {
        self.mode = VimMode::Command;
        self.command_buffer.clear();
    }

    pub fn move_up(&mut self, count: usize) {
        self.cursor_y = self.cursor_y.saturating_sub(count);
    }

    pub fn move_down(&mut self, count: usize, max_y: usize) {
        self.cursor_y = (self.cursor_y + count).min(max_y);
    }

    pub fn move_left(&mut self, count: usize) {
        self.cursor_x = self.cursor_x.saturating_sub(count);
    }

    pub fn move_right(&mut self, count: usize, max_x: usize) {
        self.cursor_x = (self.cursor_x + count).min(max_x);
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor_x = 0;
    }

    pub fn move_to_line_end(&mut self, max_x: usize) {
        self.cursor_x = max_x;
    }

    pub fn move_to_top(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn move_to_bottom(&mut self, max_y: usize) {
        self.cursor_y = max_y;
    }
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct VimEngine {
    state: VimState,
    file_count: usize,
}

impl VimEngine {
    pub fn new() -> Self {
        Self {
            state: VimState::new(),
            file_count: 0,
        }
    }

    pub fn state(&self) -> &VimState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut VimState {
        &mut self.state
    }

    pub fn set_file_count(&mut self, count: usize) {
        self.file_count = count;
    }

    pub fn handle_key(&mut self, key: char, ctrl: bool, shift: bool) -> Option<VimAction> {
        match self.state.mode {
            VimMode::Normal => self.handle_normal_key(key, ctrl, shift),
            VimMode::Insert => self.handle_insert_key(key, ctrl, shift),
            VimMode::Visual => self.handle_visual_key(key, ctrl, shift),
            VimMode::Command => self.handle_command_key(key, ctrl, shift),
        }
    }

    fn handle_normal_key(&mut self, key: char, ctrl: bool, _shift: bool) -> Option<VimAction> {
        if ctrl {
            return match key {
                'f' => Some(VimAction::Search),
                'd' => Some(VimAction::HalfPageDown),
                'u' => Some(VimAction::HalfPageUp),
                _ => None,
            };
        }

        // Handle repeat count
        if key.is_ascii_digit() && key != '0' {
            self.state.repeat_count = self.state.repeat_count * 10 + (key as u32 - '0' as u32);
            return None;
        }

        let count = if self.state.repeat_count > 0 {
            self.state.repeat_count as usize
        } else {
            1
        };

        let action = match key {
            'h' | '←' => {
                self.state.move_left(count);
                Some(VimAction::MoveLeft(count))
            }
            'j' | '↓' => {
                self.state.move_down(count, usize::MAX);
                Some(VimAction::MoveDown(count))
            }
            'k' | '↑' => {
                self.state.move_up(count);
                Some(VimAction::MoveUp(count))
            }
            'l' | '→' => {
                self.state.move_right(count, usize::MAX);
                Some(VimAction::MoveRight(count))
            }
            '0' => {
                self.state.move_to_line_start();
                Some(VimAction::None)
            }
            '$' => {
                self.state.move_to_line_end(usize::MAX);
                Some(VimAction::MoveToLineEnd)
            }
            'g' => {
                if self.state.repeat_count == 0 {
                    //等待下一个g
                    return None;
                }
                self.state.move_to_top();
                Some(VimAction::MoveToTop)
            }
            'G' => {
                self.state.move_to_bottom(usize::MAX);
                Some(VimAction::MoveToBottom)
            }
            'i' => {
                self.state.enter_insert();
                Some(VimAction::EnterInsert)
            }
            'I' => {
                self.state.enter_insert();
                self.state.move_to_line_start();
                Some(VimAction::EnterInsert)
            }
            'a' => {
                self.state.enter_insert();
                self.state.move_right(1, usize::MAX);
                Some(VimAction::EnterInsert)
            }
            'A' => {
                self.state.enter_insert();
                Some(VimAction::MoveToLineEnd)
            }
            'o' => {
                self.state.enter_insert();
                Some(VimAction::OpenLineBelow)
            }
            'O' => {
                self.state.enter_insert();
                Some(VimAction::OpenLineAbove)
            }
            'v' => {
                self.state.enter_visual();
                Some(VimAction::EnterVisual)
            }
            ':' => {
                self.state.enter_command();
                Some(VimAction::EnterCommand)
            }
            'x' => Some(VimAction::Delete),
            'd' => Some(VimAction::Delete),
            'y' => Some(VimAction::Yank),
            'p' => Some(VimAction::Paste),
            'u' => Some(VimAction::Undo),
            'r' => Some(VimAction::Redo),
            '/' => Some(VimAction::Search),
            'n' => Some(VimAction::SearchNext),
            'N' => Some(VimAction::SearchPrev),
            'q' => Some(VimAction::Quit),
            _ => None,
        };

        self.state.repeat_count = 0;
        action
    }

    fn handle_insert_key(&mut self, key: char, ctrl: bool, _shift: bool) -> Option<VimAction> {
        if ctrl {
            return match key {
                'c' | '[' => {
                    self.state.reset();
                    Some(VimAction::ExitInsert)
                }
                _ => None,
            };
        }

        match key {
            '\x1b' => {
                self.state.reset();
                Some(VimAction::ExitInsert)
            }
            '\r' | '\n' => Some(VimAction::Confirm),
            _ => None,
        }
    }

    fn handle_visual_key(&mut self, key: char, ctrl: bool, _shift: bool) -> Option<VimAction> {
        if ctrl {
            return None;
        }

        match key {
            '\x1b' => {
                self.state.reset();
                Some(VimAction::ExitVisual)
            }
            'h' | '←' => Some(VimAction::MoveLeft(1)),
            'j' | '↓' => Some(VimAction::MoveDown(1)),
            'k' | '↑' => Some(VimAction::MoveUp(1)),
            'l' | '→' => Some(VimAction::MoveRight(1)),
            'd' | 'x' => Some(VimAction::DeleteSelection),
            'y' => Some(VimAction::YankSelection),
            _ => None,
        }
    }

    fn handle_command_key(&mut self, key: char, ctrl: bool, _shift: bool) -> Option<VimAction> {
        if ctrl {
            return match key {
                'c' | '[' => {
                    self.state.reset();
                    Some(VimAction::ExitCommand)
                }
                _ => None,
            };
        }

        match key {
            '\x1b' => {
                self.state.reset();
                Some(VimAction::ExitCommand)
            }
            '\r' | '\n' => {
                let cmd = self.state.command_buffer.clone();
                self.state.reset();
                Some(VimAction::ExecuteCommand(cmd))
            }
            '\x08' => {
                // Backspace
                self.state.command_buffer.pop();
                None
            }
            _ => {
                self.state.command_buffer.push(key);
                None
            }
        }
    }
}

impl Default for VimEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VimAction {
    None,
    MoveLeft(usize),
    MoveRight(usize),
    MoveUp(usize),
    MoveDown(usize),
    MoveToLineStart,
    MoveToLineEnd,
    MoveToTop,
    MoveToBottom,
    EnterInsert,
    EnterVisual,
    EnterCommand,
    ExitInsert,
    ExitVisual,
    ExitCommand,
    Delete,
    DeleteSelection,
    Yank,
    YankSelection,
    Paste,
    Undo,
    Redo,
    Search,
    SearchNext,
    SearchPrev,
    HalfPageUp,
    HalfPageDown,
    OpenLineAbove,
    OpenLineBelow,
    Confirm,
    ExecuteCommand(String),
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_mode_default() {
        assert_eq!(VimMode::default(), VimMode::Normal);
    }

    #[test]
    fn test_vim_mode_name() {
        assert_eq!(VimMode::Normal.name(), "NORMAL");
        assert_eq!(VimMode::Insert.name(), "INSERT");
        assert_eq!(VimMode::Visual.name(), "VISUAL");
        assert_eq!(VimMode::Command.name(), "COMMAND");
    }

    #[test]
    fn test_vim_state_new() {
        let state = VimState::new();
        assert_eq!(state.mode, VimMode::Normal);
        assert_eq!(state.cursor_x, 0);
        assert_eq!(state.cursor_y, 0);
    }

    #[test]
    fn test_vim_state_move() {
        let mut state = VimState::new();
        
        state.move_down(5, 10);
        assert_eq!(state.cursor_y, 5);
        
        state.move_right(3, 20);
        assert_eq!(state.cursor_x, 3);
        
        state.move_up(2);
        assert_eq!(state.cursor_y, 3);
        
        state.move_left(1);
        assert_eq!(state.cursor_x, 2);
    }

    #[test]
    fn test_vim_state_move_bounds() {
        let mut state = VimState::new();
        
        state.move_up(5);
        assert_eq!(state.cursor_y, 0); // Can't go below 0
        
        state.move_left(5);
        assert_eq!(state.cursor_x, 0); // Can't go below 0
        
        state.move_down(100, 10);
        assert_eq!(state.cursor_y, 10); // Capped at max
        
        state.move_right(100, 20);
        assert_eq!(state.cursor_x, 20); // Capped at max
    }

    #[test]
    fn test_vim_engine_normal_mode() {
        let mut engine = VimEngine::new();
        
        // j moves down
        let action = engine.handle_key('j', false, false);
        assert_eq!(action, Some(VimAction::MoveDown(1)));
        assert_eq!(engine.state().cursor_y, 1);
        
        // k moves up
        let action = engine.handle_key('k', false, false);
        assert_eq!(action, Some(VimAction::MoveUp(1)));
        assert_eq!(engine.state().cursor_y, 0);
    }

    #[test]
    fn test_vim_engine_enter_insert() {
        let mut engine = VimEngine::new();
        
        let action = engine.handle_key('i', false, false);
        assert_eq!(action, Some(VimAction::EnterInsert));
        assert_eq!(engine.state().mode, VimMode::Insert);
    }

    #[test]
    fn test_vim_engine_exit_insert() {
        let mut engine = VimEngine::new();
        engine.state_mut().enter_insert();
        
        let action = engine.handle_key('\x1b', false, false);
        assert_eq!(action, Some(VimAction::ExitInsert));
        assert_eq!(engine.state().mode, VimMode::Normal);
    }

    #[test]
    fn test_vim_engine_repeat_count() {
        let mut engine = VimEngine::new();
        
        engine.handle_key('3', false, false);
        let action = engine.handle_key('j', false, false);
        assert_eq!(action, Some(VimAction::MoveDown(3)));
        assert_eq!(engine.state().cursor_y, 3);
    }

    #[test]
    fn test_vim_engine_command_mode() {
        let mut engine = VimEngine::new();
        
        let action = engine.handle_key(':', false, false);
        assert_eq!(action, Some(VimAction::EnterCommand));
        assert_eq!(engine.state().mode, VimMode::Command);
        
        engine.handle_key('w', false, false);
        engine.handle_key('q', false, false);
        
        let action = engine.handle_key('\r', false, false);
        assert_eq!(action, Some(VimAction::ExecuteCommand("wq".to_string())));
    }
}

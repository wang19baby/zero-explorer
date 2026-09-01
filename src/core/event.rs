#[derive(Debug, Clone)]
pub enum AppEvent {
    WindowResized(u32, u32),
    WindowMoved(i32, i32),
    WindowCloseRequested,
    WindowFocused(bool),
    KeyPressed(u32),
    KeyReleased(u32),
    CharInput(char),
    MouseClicked(u32, f64, f64),
    MouseMoved(f64, f64),
    MouseWheel(f64),
    MouseButtonPressed(u32),
    MouseButtonReleased(u32),
    FileCreated(String),
    FileDeleted(String),
    FileModified(String),
    DirectoryChanged(String),
    TabCreated(usize),
    TabClosed(usize),
    TabSelected(usize),
    PanelResized(usize, f32),
    SidebarToggled,
    LayoutChanged(usize),
    Custom(String),
}

pub struct EventDispatcher {
    handlers: Vec<Box<dyn Fn(&AppEvent)>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register<F: Fn(&AppEvent) + 'static>(&mut self, handler: F) {
        self.handlers.push(Box::new(handler));
    }

    pub fn dispatch(&self, event: &AppEvent) {
        for handler in &self.handlers {
            handler(event);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

pub trait Component {
    fn id(&self) -> &str;
    fn bounds(&self) -> &Rect;
    fn bounds_mut(&mut self) -> &mut Rect;
    fn state(&self) -> &ComponentState;
    fn set_state(&mut self, state: ComponentState);

    fn update(&mut self, _dt: f32) {}
    fn render(&self) {}

    fn is_visible(&self) -> bool {
        true
    }

    fn set_visible(&mut self, _visible: bool) {}
}

#[derive(Debug)]
pub struct Button {
    id: String,
    bounds: Rect,
    state: ComponentState,
    text: String,
    visible: bool,
}

impl Button {
    pub fn new(id: &str, text: &str, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            text: text.to_string(),
            visible: true,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl Component for Button {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
    }

    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn set_state(&mut self, state: ComponentState) {
        self.state = state;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

#[derive(Debug)]
pub struct Panel {
    id: String,
    bounds: Rect,
    state: ComponentState,
    visible: bool,
}

impl Panel {
    pub fn new(id: &str, bounds: Rect) -> Self {
        Self {
            id: id.to_string(),
            bounds,
            state: ComponentState::Normal,
            visible: true,
        }
    }
}

impl Component for Panel {
    fn id(&self) -> &str {
        &self.id
    }

    fn bounds(&self) -> &Rect {
        &self.bounds
    }

    fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.bounds
    }

    fn state(&self) -> &ComponentState {
        &self.state
    }

    fn set_state(&mut self, state: ComponentState) {
        self.state = state;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

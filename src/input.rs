#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Debug, Default)]
pub struct UiInput {
    pub mouse_pos: (f32, f32),
    pub mouse_held: Vec<MouseButton>,
    pub mouse_pressed: Vec<MouseButton>,
    pub mouse_released: Vec<MouseButton>,
    pub scroll_y: f32,
    pub text: String,
    pub keys: Vec<NamedKey>,
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub dt: f32,
}

impl UiInput {
    pub fn left_just_pressed(&self) -> bool {
        self.mouse_pressed.contains(&MouseButton::Left)
    }

    pub fn left_just_released(&self) -> bool {
        self.mouse_released.contains(&MouseButton::Left)
    }

    pub fn left_held(&self) -> bool {
        self.mouse_held.contains(&MouseButton::Left)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedKey {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Enter,
    Tab,
    Escape,
}
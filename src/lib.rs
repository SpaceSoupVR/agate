pub mod input;
pub mod layout;
pub mod text_editor;
pub mod theme;
pub mod widgets;

pub use input::{MouseButton as AMouseButton, UiInput};
pub use layout::{rect_from, Flow, OverlapGuard, Region};
pub use text_editor::TextEditor;
pub use theme::Theme;
pub use widgets::{in_rect, Rect, Ui, WidgetId};

pub use space_soup::ui2d::{Align, Area, Color, Font, Item};

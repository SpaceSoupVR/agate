use space_soup::ui2d::Color;

const fn c(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color(r, g, b, a)
}

pub const BG: Color = c(0x09, 0x17, 0x2A, 255);
pub const SURFACE: Color = c(0x0D, 0x1E, 0x35, 255);
pub const SURFACE_RAISED: Color = c(0x11, 0x27, 0x3F, 255);
pub const BORDER: Color = c(0x1E, 0x3A, 0x55, 255);
pub const SEPARATOR: Color = c(0x00, 0x00, 0x00, 100);

pub const CONTROL_BG: Color = c(0x12, 0x2A, 0x45, 255);
pub const CONTROL_HOVER: Color = c(0x19, 0x35, 0x53, 255);
pub const CONTROL_ACTIVE: Color = c(0x0A, 0x1F, 0x36, 255);
pub const CONTROL_BORDER: Color = c(0x22, 0x45, 0x65, 255);

pub const FIELD_BG: Color = c(0x07, 0x12, 0x22, 255);
pub const FIELD_BORDER: Color = c(0x1C, 0x38, 0x54, 255);
pub const FIELD_FOCUS: Color = c(0x00, 0xB5, 0xFF, 255);
pub const CARET: Color = c(0xFF, 0xD7, 0x00, 255);
pub const SELECTION_BG: Color = c(0x00, 0x4D, 0x99, 200);

pub const ACCENT: Color = c(0x00, 0xB5, 0xFF, 255);
pub const ACCENT_HI: Color = c(0x6D, 0xD6, 0xFF, 255);
pub const ACCENT_DIM: Color = c(0x00, 0xB5, 0xFF, 50);
pub const SUCCESS: Color = c(0x3A, 0xD9, 0x00, 255);
pub const WARNING: Color = c(0xFF, 0xD7, 0x00, 255);
pub const DANGER: Color = c(0xFF, 0x43, 0x4D, 255);
pub const DANGER_BG: Color = c(0xFF, 0x43, 0x4D, 35);

pub const TEXT_PRIMARY: Color = c(0xE8, 0xF0, 0xFF, 255);
pub const TEXT_SECONDARY: Color = c(0x72, 0x8F, 0xB0, 255);
pub const TEXT_DISABLED: Color = c(0x35, 0x4E, 0x65, 255);
pub const TEXT_ON_ACCENT: Color = c(0x00, 0x10, 0x26, 255);
pub const TEXT_LINK: Color = c(0x6D, 0xD6, 0xFF, 255);

pub const EDITOR_BG: Color = c(0x09, 0x16, 0x28, 255);
pub const GUTTER_BG: Color = c(0x07, 0x12, 0x22, 255);
pub const CURRENT_LINE: Color = c(0x00, 0x4A, 0x8C, 40);
pub const LINE_NUMBER: Color = c(0x2B, 0x4D, 0x70, 255);
pub const LINE_NUMBER_CUR: Color = c(0xFF, 0xD7, 0x00, 255);
pub const SCROLLBAR: Color = c(0x1C, 0x5A, 0x99, 130);

pub const TITLEBAR_BG: Color = c(0x06, 0x0F, 0x1C, 255);
pub const TOOLBAR_BG: Color = c(0x08, 0x14, 0x25, 255);
pub const SIDEBAR_BG: Color = c(0x07, 0x12, 0x20, 255);
pub const STATUSBAR_BG: Color = c(0x00, 0x0D, 0x1A, 255);

pub const SYN_PLAIN: Color = c(0xE8, 0xF0, 0xFF, 255);
pub const SYN_STRING: Color = c(0xFF, 0x97, 0x1F, 255);
pub const SYN_NUMBER: Color = c(0xFF, 0xD7, 0x00, 255);
pub const SYN_KEYWORD: Color = c(0xFF, 0x2C, 0x9C, 255);
pub const SYN_KEY: Color = c(0x00, 0xE5, 0xFF, 255);
pub const SYN_PUNCT: Color = c(0x5B, 0x8A, 0xB0, 255);
pub const SYN_COMMENT: Color = c(0x3A, 0x5F, 0x80, 255);

pub const PT_DISPLAY: f32 = 28.0;
pub const PT_TITLE: f32 = 20.0;
pub const PT_HEADING: f32 = 16.0;
pub const PT_BODY: f32 = 13.0;
pub const PT_SMALL: f32 = 11.0;
pub const PT_EDITOR: f32 = 12.0;
pub const PT_EDITOR_LH: f32 = 17.0;

pub const CORNER: f32 = 8.0;
pub const CORNER_SM: f32 = 5.0;
pub const CORNER_LG: f32 = 12.0;
pub const PAD: f32 = 10.0;
pub const PAD_SM: f32 = 6.0;
pub const ROW_H: f32 = 32.0;
pub const BTN_H: f32 = 32.0;
pub const FIELD_H: f32 = 28.0;
pub const SLIDER_TRACK: f32 = 4.0;
pub const SLIDER_THUMB: f32 = 16.0;
pub const TOGGLE_W: f32 = 40.0;
pub const TOGGLE_H: f32 = 22.0;
pub const CHECK_BOX: f32 = 18.0;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub scale: f32,
}

impl Theme {
    pub fn new(scale: f32) -> Self {
        Self {
            scale: scale.max(0.25),
        }
    }

    #[inline]
    pub fn px(&self, points: f32) -> f32 {
        points * self.scale
    }
    #[inline]
    pub fn font(&self, points: f32) -> f32 {
        points * self.scale
    }
    #[inline]
    pub fn body(&self) -> f32 {
        self.font(PT_BODY)
    }
    #[inline]
    pub fn small(&self) -> f32 {
        self.font(PT_SMALL)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(1.0)
    }
}

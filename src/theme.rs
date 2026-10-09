use space_soup::ui2d::Color;

const fn c(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color(r, g, b, a)
}

// Apple-style dark "glass" palette. The editor window is transparent with a
// macOS vibrancy blur behind it, so chrome colors are deliberately translucent:
// panels are dark washes, controls are white washes, and the compositor does
// the frosted-glass work. Popups (tooltips, dropdowns, dialogs) stay
// near-opaque so they read over anything.

pub const BG: Color = c(20, 21, 25, 255);
pub const SURFACE: Color = c(255, 255, 255, 14);
pub const SURFACE_RAISED: Color = c(44, 44, 48, 244);
pub const BORDER: Color = c(255, 255, 255, 32);
pub const SEPARATOR: Color = c(255, 255, 255, 22);

pub const CONTROL_BG: Color = c(255, 255, 255, 26);
pub const CONTROL_HOVER: Color = c(255, 255, 255, 44);
pub const CONTROL_ACTIVE: Color = c(0, 0, 0, 60);
pub const CONTROL_BORDER: Color = c(255, 255, 255, 40);

pub const FIELD_BG: Color = c(0, 0, 0, 72);
pub const FIELD_BORDER: Color = c(255, 255, 255, 28);
pub const FIELD_FOCUS: Color = c(0x0A, 0x84, 0xFF, 255);
pub const CARET: Color = c(0x0A, 0x84, 0xFF, 255);
pub const SELECTION_BG: Color = c(0x0A, 0x84, 0xFF, 96);

// macOS dark-mode system colors.
pub const ACCENT: Color = c(0x0A, 0x84, 0xFF, 255); // systemBlue
pub const ACCENT_HI: Color = c(0x6C, 0xB4, 0xFF, 255);
pub const ACCENT_DIM: Color = c(0x0A, 0x84, 0xFF, 56);
pub const SUCCESS: Color = c(0x30, 0xD1, 0x58, 255); // systemGreen
pub const WARNING: Color = c(0xFF, 0xD6, 0x0A, 255); // systemYellow
pub const DANGER: Color = c(0xFF, 0x45, 0x3A, 255); // systemRed
pub const DANGER_BG: Color = c(0xFF, 0x45, 0x3A, 42);

pub const TEXT_PRIMARY: Color = c(255, 255, 255, 245);
pub const TEXT_SECONDARY: Color = c(235, 235, 245, 153);
pub const TEXT_DISABLED: Color = c(235, 235, 245, 78);
pub const TEXT_ON_ACCENT: Color = c(255, 255, 255, 255);
pub const TEXT_LINK: Color = c(0x6C, 0xB4, 0xFF, 255);

// The code editor keeps a darker, barely-translucent backing so text stays
// readable over whatever the desktop shows through.
pub const EDITOR_BG: Color = c(20, 21, 25, 255);
pub const GUTTER_BG: Color = c(20, 21, 24, 255);
pub const CURRENT_LINE: Color = c(255, 255, 255, 14);
pub const LINE_NUMBER: Color = c(235, 235, 245, 70);
pub const LINE_NUMBER_CUR: Color = c(235, 235, 245, 190);
pub const SCROLLBAR: Color = c(255, 255, 255, 72);

pub const TITLEBAR_BG: Color = c(30, 31, 36, 255);
pub const TOOLBAR_BG: Color = c(30, 31, 36, 255);
pub const SIDEBAR_BG: Color = c(24, 25, 29, 255);
pub const STATUSBAR_BG: Color = c(26, 27, 31, 255);

// Xcode-dark-inspired syntax colors.
pub const SYN_PLAIN: Color = c(255, 255, 255, 245);
pub const SYN_STRING: Color = c(0xFC, 0x6A, 0x5D, 255);
pub const SYN_NUMBER: Color = c(0xD0, 0xBF, 0x69, 255);
pub const SYN_KEYWORD: Color = c(0xFC, 0x5F, 0xA3, 255);
pub const SYN_KEY: Color = c(0x5D, 0xD8, 0xFF, 255);
pub const SYN_PUNCT: Color = c(235, 235, 245, 140);
pub const SYN_COMMENT: Color = c(0x7F, 0x8C, 0x98, 255);

pub const PT_DISPLAY: f32 = 28.0;
pub const PT_TITLE: f32 = 20.0;
pub const PT_HEADING: f32 = 16.0;
pub const PT_BODY: f32 = 13.0;
pub const PT_SMALL: f32 = 11.0;
pub const PT_EDITOR: f32 = 12.0;
pub const PT_EDITOR_LH: f32 = 17.0;

pub const CORNER: f32 = 9.0;
pub const CORNER_SM: f32 = 6.0;
pub const CORNER_LG: f32 = 14.0;
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

// theme.rs — Camera design tokens (GPUI)
// Colors are 0xRRGGBBAA: gpui::rgba(color::CONTROL)

pub mod color {
    pub const LETTERBOX: u32           = 0x000000FF;
    pub const APP_BG: u32              = 0x0B0B0CFF;
    pub const CONTROL: u32             = 0x1414168F;
    pub const CONTROL_HOVER: u32       = 0x242428B8;
    pub const CONTROL_ON: u32          = 0xFFFFFF38;
    pub const SELECTED: u32            = 0xFFFFFFEB;
    pub const OVERLAY: u32             = 0x18181BF5;
    pub const ITEM_HOVER: u32          = 0xFFFFFF14;
    pub const KEYCAP: u32              = 0xFFFFFF1A;
    pub const KEYCAP_BORDER: u32       = 0xFFFFFF29;
    pub const BORDER_SUBTLE: u32       = 0xFFFFFF1A;
    pub const BORDER_STRONG: u32       = 0xFFFFFF3D;
    pub const BORDER_STRONG_HOVER: u32 = 0xFFFFFF7A;
    pub const ICON_TILE: u32           = 0xFFFFFF0F;
    pub const TEXT_PRIMARY: u32        = 0xFFFFFFF5;
    pub const TEXT_SECONDARY: u32      = 0xFFFFFFB8;
    pub const TEXT_TERTIARY: u32       = 0xFFFFFF80;
    pub const TEXT_DISABLED: u32       = 0xFFFFFF52;
    pub const TEXT_ON_SELECTED: u32    = 0x111113FF;

    pub const SHUTTER_RING: u32         = 0xFFFFFFEB;
    pub const SHUTTER_FILL: u32         = 0xFFFFFFFF;
    pub const SHUTTER_FILL_HOVER: u32   = 0xFFFFFFDB;
    pub const SHUTTER_FILL_PRESSED: u32 = 0xFFFFFFB3;
    pub const RECORD: u32               = 0xFF453AFF;
    pub const RECORD_HOVER: u32         = 0xFF5F55FF;
    pub const RECORD_PRESSED: u32       = 0xE0352BFF;

    // Gradients: color at the edge, fading to transparent
    pub const SCRIM_TOP: u32      = 0x00000066; // 96 px, top down
    pub const SCRIM_BOTTOM: u32   = 0x0000008C; // 200 px, bottom up
    pub const DIM: u32            = 0x00000040; // countdown
    pub const FLASH: u32          = 0xFFFFFFCC; // peak
    pub const THUMB_BADGE: u32    = 0x00000099;
    pub const FOCUS_RING: u32     = 0xFFFFFFE6;
    pub const SHADOW_CONTROL: u32 = 0x0000003D; // y 2, blur 8
    pub const SHADOW_OVERLAY: u32 = 0x00000066; // y 8, blur 24
}

// Lengths in logical px: gpui::px(size::SHUTTER)
pub mod radius {
    pub const XS: f32 = 4.0;  pub const STOP: f32 = 6.0;
    pub const SM: f32 = 8.0;  pub const MD: f32 = 10.0;
    pub const LG: f32 = 14.0; pub const FULL: f32 = 9999.0;
}
pub mod space {
    pub const S2: f32 = 2.0;   pub const S4: f32 = 4.0;
    pub const S8: f32 = 8.0;   pub const S12: f32 = 12.0;
    pub const S16: f32 = 16.0; pub const S24: f32 = 24.0;
    pub const S40: f32 = 40.0;
}
pub mod size {
    pub const CONTROL: f32 = 40.0;  pub const MIRROR: f32 = 48.0;
    pub const THUMB: f32 = 48.0;    pub const SHUTTER: f32 = 72.0;
    pub const SHUTTER_RING: f32 = 4.0;
    pub const SHUTTER_FILL: f32 = 56.0;
    pub const STOP: f32 = 26.0;     pub const PILL: f32 = 36.0;
    pub const SEGMENT: f32 = 32.0;  pub const MENU_ROW: f32 = 32.0;
    pub const TOOLTIP: f32 = 28.0;  pub const KEYCAP: f32 = 20.0;
    pub const STATUS: f32 = 32.0;   pub const REC_DOT: f32 = 8.0;
    pub const ICON_SM: f32 = 14.0;  pub const ICON_MD: f32 = 16.0;
    pub const ICON_LG: f32 = 20.0;  pub const ICON_XL: f32 = 28.0;
}
pub mod font {
    // Weights: FontWeight::LIGHT / NORMAL / MEDIUM / SEMIBOLD
    pub const FAMILY: &str = "Inter";
    pub const COUNTDOWN: f32 = 144.0; // LIGHT, tabular
    pub const TITLE: f32 = 18.0;      // SEMIBOLD
    pub const BODY: f32 = 13.0;       // NORMAL, line 20
    pub const LABEL: f32 = 13.0;      // MEDIUM / SEMIBOLD
    pub const SMALL: f32 = 12.0;      // MEDIUM
    pub const CAPTION: f32 = 11.0;    // SEMIBOLD
}
pub mod motion {
    use std::time::Duration;
    const fn ms(v: u64) -> Duration { Duration::from_millis(v) }
    pub const HOVER: Duration = ms(100);
    pub const MODE_SWITCH: Duration = ms(150);
    pub const CONTROLS_IN: Duration = ms(150);
    pub const CONTROLS_OUT: Duration = ms(300);
    pub const IDLE_TIMEOUT: Duration = ms(2500);
    pub const MENU: Duration = ms(120);
    pub const TOOLTIP_DELAY: Duration = ms(500);
    pub const FLASH_IN: Duration = ms(40);
    pub const FLASH_OUT: Duration = ms(260);
    pub const TICK_FADE: Duration = ms(150);
    pub const REC_PULSE: Duration = ms(1000);
    pub const TOAST_HOLD: Duration = ms(2000);
}

use crate::{Font, Language, LocalizableString, Text, TextPainter};
use fontdue::layout::{HorizontalAlign, LayoutSettings, VerticalAlign};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};

pub const DEBUG_TEXT: Color = Color::RGB(0xFF, 0xFF, 0x88);
pub const WINDOW_BACKGROUND: Color = Color::RGB(0x33, 0x33, 0x33);
pub const HUD_BACKGROUND_TRANSPARENT: Color = Color::RGBA(0x44, 0x44, 0x44, 0xAA);
pub const HUD_BACKGROUND_OPAQUE: Color = Color::RGB(0x44, 0x44, 0x44);
pub const HUD_BORDER: Color = Color::RGB(0x77, 0x88, 0x88);
pub const HUD_BUTTON_BACKGROUND: Color = Color::RGB(0x55, 0x55, 0x55);
pub const HUD_BUTTON_BACKGROUND_DISABLED: Color = Color::RGB(0x4A, 0x4A, 0x4A);
pub const HUD_BUTTON_BACKGROUND_HIGHLIGHT: Color = Color::RGB(0x66, 0x66, 0x66);
pub const HUD_BUTTON_BACKGROUND_PRESSED: Color = Color::RGB(0x5D, 0x5D, 0x5D);
pub const HEALTH_BORDER: Color = Color::RGBA(0x33, 0x33, 0x33, 0x44);
pub const HEALTH_EMPTY: Color = Color::RGBA(0xAA, 0xAA, 0xAA, 0xAA);
pub const HEALTH_LOW: Color = Color::RGB(0xCC, 0x33, 0x22);
pub const HEALTH_MEDIUM: Color = Color::RGB(0xEE, 0xAA, 0x22);
pub const HEALTH_HIGH: Color = Color::RGB(0x66, 0xCC, 0x33);
pub const SCREEN_FADE_COLOR: Color = Color::RGBA(0x33, 0x33, 0x33, 0xBB);
pub const HOTKEY_TIP: Color = Color::RGBA(0xDD, 0xDD, 0xDD, 0xFF);
pub const ROW_BACKGROUND: Color = Color::RGB(0x44, 0x44, 0x44);
pub const ROW_BACKGROUND_ALT: Color = Color::RGB(0x3A, 0x3A, 0x3A);
pub const ROW_BACKGROUND_HIGHLIGHT: Color = Color::RGB(0x3A, 0x55, 0x3A);
pub const SCROLL_BACKGROUND: Color = Color::RGB(0x3A, 0x3A, 0x3A);
pub const SCROLL_HANDLE: Color = Color::RGB(0x55, 0x55, 0x55);

pub struct UserInterface {
    pub mouse_position: Point,
    pub mouse_left_pressed: bool,
    pub mouse_left_released: bool,
    pub mouse_right_pressed: bool,
    pub mouse_right_released: bool,
    pub hovering: bool,
    pub button_count: usize,
    pub released_buttons: [bool; 9],
    pub scroll: i32,
    pub text_input: Option<String>,
}

impl UserInterface {
    pub fn new() -> UserInterface {
        UserInterface {
            mouse_position: Point::new(0, 0),
            mouse_left_pressed: false,
            mouse_left_released: false,
            mouse_right_pressed: false,
            mouse_right_released: false,
            hovering: false,
            button_count: 0,
            released_buttons: [false; 9],
            scroll: 0,
            text_input: None,
        }
    }

    pub fn reset_for_new_frame(&mut self) {
        self.mouse_left_released = false;
        self.mouse_right_released = false;
        self.hovering = false;
        self.button_count = 0;
        self.released_buttons = [false; 9];
        self.scroll = 0;
        self.text_input = None;
    }

    pub fn button<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        text_painter: &mut TextPainter,
        text: &LocalizableString,
        rect: Rect,
        enabled: bool,
    ) -> bool {
        let hovering = rect.contains_point(self.mouse_position);
        if enabled {
            if hovering {
                self.hovering = true;
                if self.mouse_left_pressed {
                    canvas.set_draw_color(HUD_BUTTON_BACKGROUND_PRESSED);
                } else {
                    canvas.set_draw_color(HUD_BUTTON_BACKGROUND_HIGHLIGHT);
                }
            } else {
                canvas.set_draw_color(HUD_BUTTON_BACKGROUND);
            }
        } else {
            canvas.set_draw_color(HUD_BUTTON_BACKGROUND_DISABLED);
        }
        let _ = canvas.fill_rect(rect);
        canvas.set_draw_color(HUD_BORDER);
        let _ = canvas.draw_rect(rect);

        let layout = LayoutSettings {
            x: (rect.x + 4) as f32,
            y: (rect.y + 4) as f32,
            max_width: Some((rect.width() - 8 - 12) as f32),
            max_height: Some((rect.height() - 8) as f32),
            vertical_align: VerticalAlign::Middle,
            horizontal_align: HorizontalAlign::Center,
            ..LayoutSettings::default()
        };
        let mut texts = text.localize(Language::English);

        self.button_count += 1;
        let hotkey_pressed = if self.button_count < 10 {
            let hotkey_tip = Text(Font::RegularUi, 14.0, HOTKEY_TIP, format!("[{}] ", self.button_count));
            texts.insert(0, hotkey_tip);
            self.released_buttons[self.button_count - 1]
        } else {
            false
        };

        if !enabled {
            for text in &mut texts {
                text.2 = Color::RGB(text.2.r / 2, text.2.g / 2, text.2.b / 2);
            }
        }

        canvas.set_clip_rect(rect);
        text_painter.draw_text(canvas, &layout, &texts);
        canvas.set_clip_rect(None);

        enabled && (hotkey_pressed || (hovering && self.mouse_left_released))
    }

    pub fn text_box<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        text_painter: &mut TextPainter,
        text: &LocalizableString,
        rect: Rect,
        opaque: bool,
    ) {
        canvas.set_draw_color(if opaque {
            HUD_BACKGROUND_OPAQUE
        } else {
            HUD_BACKGROUND_TRANSPARENT
        });
        let _ = canvas.fill_rect(rect);

        let layout = LayoutSettings {
            x: (rect.x + 8) as f32,
            y: (rect.y + 8) as f32,
            max_width: Some((rect.width() - 16) as f32),
            max_height: Some((rect.height() - 16) as f32),
            ..LayoutSettings::default()
        };
        canvas.set_clip_rect(rect);
        text_painter.draw_text(canvas, &layout, &text.localize(Language::English));
        canvas.set_clip_rect(None);

        canvas.set_draw_color(HUD_BORDER);
        let _ = canvas.draw_rect(rect);
    }

    pub fn text<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        text_painter: &mut TextPainter,
        text: &LocalizableString,
        x: i32,
        y: i32,
    ) {
        let layout = LayoutSettings {
            x: x as f32,
            y: y as f32,
            ..LayoutSettings::default()
        };
        text_painter.draw_text(canvas, &layout, &text.localize(Language::English));
    }
}

use crate::{interface, Font, Language, LocalizableString, Text, TextPainter};
use fontdue::layout::{LayoutSettings, VerticalAlign};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};

/// The log visible to the player in-game, as opposed to internal
/// debugging logs better suited to the `log` crate and such.
#[derive(Clone, PartialEq, Debug)]
pub struct GameLog {
    messages: Vec<(u64, LocalizableString)>,
}

impl GameLog {
    pub fn new() -> GameLog {
        GameLog { messages: Vec::new() }
    }

    pub fn combat(&mut self, round: u64, message: LocalizableString) {
        self.messages.push((round, message));
    }

    pub fn lockpicking(&mut self, round: u64, message: LocalizableString) {
        self.messages.push((round, message));
    }

    pub fn draw_messages<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, text_painter: &mut TextPainter) {
        let (width, height) = canvas.output_size().map(|(a, b)| (a as i32, b as i32)).unwrap();
        let margin = 10;
        let log_width = width - margin * 2;
        let log_height = 16 * 12;
        // TODO: Hide the log after no activity for N rounds
        let background_rect = Rect::new(
            width - (log_width + margin),
            height - (log_height + margin),
            log_width as u32,
            log_height as u32,
        );
        let text_margin = 4;
        let layout = LayoutSettings {
            x: (background_rect.x() + text_margin) as f32,
            y: (background_rect.y() + text_margin) as f32,
            max_width: Some((background_rect.width() as i32 - text_margin * 2) as f32),
            max_height: Some((background_rect.height() as i32 - text_margin * 2) as f32),
            vertical_align: VerticalAlign::Bottom,
            ..LayoutSettings::default()
        };

        let mut localized_texts: Vec<Text> = Vec::new();
        for (round, message) in &self.messages {
            // TODO: Add language option, pass it to GameLog
            localized_texts.push(Text(Font::RegularUi, 14.0, Color::WHITE, String::from("\n")));
            localized_texts.push(Text(
                Font::RegularUi,
                14.0,
                Color::WHITE,
                format!(
                    " ::: 21XX-03-{d:x} T {h:02}:{m:02}:{s:02} :::\n",
                    d = 0x14 + round / 60 / 60 / 24,
                    h = (5 + round / 60 / 60) % 24,
                    m = (31 + round / 60) % 60,
                    s = round % 60
                ),
            ));
            localized_texts.extend(message.localize(Language::English).into_iter());
        }

        canvas.set_draw_color(interface::HUD_BACKGROUND_TRANSPARENT);
        let _ = canvas.fill_rect(background_rect);

        canvas.set_clip_rect(background_rect);
        text_painter.draw_text(canvas, &layout, &localized_texts);
        canvas.set_clip_rect(None);

        canvas.set_draw_color(interface::HUD_BORDER);
        let _ = canvas.draw_rect(background_rect);
    }
}

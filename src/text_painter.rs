use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::{Font as FontdueFont, FontSettings};
use fontdue_sdl2::FontTexture;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget, TextureCreator};

#[derive(Clone, Copy, Debug)]
pub enum Font {
    RegularUi,
    #[doc(hidden)]
    Count,
}

pub struct TextPainter<'r> {
    font_texture: FontTexture<'r>,
    fonts: [FontdueFont; Font::Count as usize],
    layout: Layout<Color>,
}

impl TextPainter<'_> {
    pub fn new<'r, T>(texture_creator: &'r TextureCreator<T>) -> Result<TextPainter<'r>, String> {
        let font_texture = FontTexture::new(&texture_creator)?;
        let font = include_bytes!("fonts/iceland/Iceland-Regular.ttf") as &[u8];
        let regular_ui = FontdueFont::from_bytes(font, FontSettings::default()).unwrap();
        let fonts = [regular_ui];
        let layout = Layout::new(CoordinateSystem::PositiveYDown);

        Ok(TextPainter {
            font_texture,
            fonts,
            layout,
        })
    }

    pub fn draw_text<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        text_parts: &[(Font, f32, Color, &str)],
    ) {
        self.layout.reset(&LayoutSettings::default());
        for (font_enum, font_size, color, text) in text_parts {
            self.layout.append(
                &self.fonts,
                &TextStyle::with_user_data(text, *font_size, *font_enum as usize, *color),
            );
        }
        let _ = self
            .font_texture
            .draw_text(canvas, &self.fonts, self.layout.glyphs());
    }
}

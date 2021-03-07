use png::{BitDepth, ColorType};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{
    BlendMode, Canvas, RenderTarget, Texture, TextureCreator, TextureValueError, UpdateTextureError,
};

pub const TILE_STRIDE: i32 = 64;
const TILE_COLUMNS: i32 = 256 / TILE_STRIDE;
const TILE_WIDTH: u32 = TILE_STRIDE as u32;
const TILE_HEIGHT: u32 = TILE_STRIDE as u32;

#[derive(Clone, Copy)]
pub enum TileGraphic {
    Ground,
    Wall,
    WallBackground,
    Player,
}

#[derive(Debug)]
pub enum ImageLoadingError {
    Png(png::DecodingError),
    TextureCreation(TextureValueError),
    TextureUpload(UpdateTextureError),
    UnsupportedFormat,
}

impl From<png::DecodingError> for ImageLoadingError {
    fn from(err: png::DecodingError) -> ImageLoadingError {
        ImageLoadingError::Png(err)
    }
}

impl From<TextureValueError> for ImageLoadingError {
    fn from(err: TextureValueError) -> ImageLoadingError {
        ImageLoadingError::TextureCreation(err)
    }
}

impl From<UpdateTextureError> for ImageLoadingError {
    fn from(err: UpdateTextureError) -> ImageLoadingError {
        ImageLoadingError::TextureUpload(err)
    }
}

pub struct TilePainter<'r> {
    tileset: Texture<'r>,
}

impl TilePainter<'_> {
    pub fn new<'r, T>(
        texture_creator: &'r TextureCreator<T>,
    ) -> Result<TilePainter<'r>, ImageLoadingError> {
        let bytes: &[u8] = include_bytes!("graphics/tileset.png");
        let decoder = png::Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;

        let format = match (info.color_type, info.bit_depth) {
            (ColorType::RGBA, BitDepth::Eight) => PixelFormatEnum::RGBA32,
            _ => return Err(ImageLoadingError::UnsupportedFormat),
        };
        let mut tileset = texture_creator.create_texture_static(format, info.width, info.height)?;
        let pitch = info.width as usize * format.byte_size_per_pixel();
        tileset.update(None, &buf, pitch)?;
        tileset.set_blend_mode(BlendMode::Blend);
        Ok(TilePainter { tileset })
    }

    pub fn draw_tile<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        tile: TileGraphic,
        x: i32,
        y: i32,
    ) {
        let tile_x = tile as usize as i32 % TILE_COLUMNS;
        let tile_y = tile as usize as i32 / TILE_COLUMNS;
        let _ = canvas.copy(
            &self.tileset,
            Rect::new(
                tile_x * TILE_STRIDE,
                tile_y * TILE_STRIDE,
                TILE_WIDTH,
                TILE_HEIGHT,
            ),
            Rect::new(x, y, TILE_WIDTH, TILE_HEIGHT),
        );
    }
}

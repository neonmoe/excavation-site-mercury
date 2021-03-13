use png::{BitDepth, ColorType};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, RenderTarget, Texture, TextureCreator, TextureValueError, UpdateTextureError};

pub const TILE_STRIDE: i32 = 64;
const TILE_COLUMNS: i32 = 512 / TILE_STRIDE;
const TILE_WIDTH: u32 = TILE_STRIDE as u32;
const TILE_HEIGHT: u32 = TILE_STRIDE as u32;

#[derive(Clone, Copy, PartialEq)]
pub enum TileLayer {
    BelowFighters,
    AboveFighters,
    AboveAll,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TileGraphic {
    Ground,
    WallTop,
    WallSide,
    Player,
    ShadowLeft,
    ShadowBottom,
    ShadowBottomLeft,
    ShadowTopLeft,
    CornerShadowTopLeft,
    TileHighlight,
    DoorClosed,
    DoorOpening,
    DoorOpen,
    SideDoorClosed,
    SideDoorOpening,
    SideDoorOpen,
    Slime,
    DeadSlime,
    Roach,
    DeadRoach,
    Rockman,
    DeadRockman,
    SentientMetal,
    DeadSentientMetal,
    LevelExit,
    MineralCounter,
    MineralsScattered,
    FinalTreasureMinerals,
    LockedDoor,
    HotGround,
    HotWallTop,
    HotWallSide,
    LaserBeam,
    AttackMiss,
    AttackHit,
}

impl TileGraphic {
    pub const fn layer(self) -> TileLayer {
        match self {
            TileGraphic::WallTop | TileGraphic::HotWallTop => TileLayer::AboveAll,
            TileGraphic::DoorClosed | TileGraphic::LockedDoor => TileLayer::AboveFighters,
            _ => TileLayer::BelowFighters,
        }
    }

    pub const fn dead(self) -> TileGraphic {
        match self {
            TileGraphic::Slime => TileGraphic::DeadSlime,
            TileGraphic::Roach => TileGraphic::DeadRoach,
            TileGraphic::Rockman => TileGraphic::DeadRockman,
            TileGraphic::SentientMetal => TileGraphic::DeadSentientMetal,
            x => x,
        }
    }
}

pub struct TilePainter<'r> {
    pub tileset: Texture<'r>,
    shadow_tileset: Texture<'r>,
}

impl TilePainter<'_> {
    pub fn new<'r, T>(texture_creator: &'r TextureCreator<T>) -> Result<TilePainter<'r>, ImageLoadingError> {
        let bytes: &[u8] = include_bytes!("graphics/tileset-quantized.png");
        let decoder = png::Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;

        let format = match (info.color_type, info.bit_depth) {
            (ColorType::RGBA, BitDepth::Eight) => PixelFormatEnum::RGBA32,
            _ => return Err(ImageLoadingError::UnsupportedFormat),
        };
        let pitch = info.width as usize * format.byte_size_per_pixel();

        let mut tileset = texture_creator.create_texture_static(format, info.width, info.height)?;
        tileset.update(None, &buf, pitch)?;
        tileset.set_blend_mode(BlendMode::Blend);

        let mut shadow_tileset = texture_creator.create_texture_static(format, info.width, info.height)?;
        for pixel in buf.chunks_mut(4) {
            pixel[0] = 0x44;
            pixel[1] = 0x44;
            pixel[2] = 0x44;
            pixel[3] /= 2;
        }
        shadow_tileset.update(None, &buf, pitch)?;
        shadow_tileset.set_blend_mode(BlendMode::Blend);

        Ok(TilePainter {
            tileset,
            shadow_tileset,
        })
    }

    pub fn draw_tile_shadowed_ex<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        tile: TileGraphic,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        flip_h: bool,
        flip_v: bool,
    ) {
        let tile_x = tile as usize as i32 % TILE_COLUMNS;
        let tile_y = tile as usize as i32 / TILE_COLUMNS;
        let src_rect = Rect::new(tile_x * TILE_STRIDE, tile_y * TILE_STRIDE, TILE_WIDTH, TILE_HEIGHT);
        let dst_rect = Rect::new(x + 4, y - 2, width, height);
        let _ = canvas.copy_ex(&self.shadow_tileset, src_rect, dst_rect, 0.0, None, flip_h, flip_v);
        let dst_rect = Rect::new(x - 1, y, width, height);
        let _ = canvas.copy_ex(&self.shadow_tileset, src_rect, dst_rect, 0.0, None, flip_h, flip_v);
        let dst_rect = Rect::new(x, y + 1, width, height);
        let _ = canvas.copy_ex(&self.shadow_tileset, src_rect, dst_rect, 0.0, None, flip_h, flip_v);
        let dst_rect = Rect::new(x, y, width, height);
        let _ = canvas.copy_ex(&self.tileset, src_rect, dst_rect, 0.0, None, flip_h, flip_v);
    }

    pub fn draw_tile_shadowed<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        tile: TileGraphic,
        x: i32,
        y: i32,
        flip_h: bool,
        flip_v: bool,
    ) {
        self.draw_tile_shadowed_ex(canvas, tile, x, y, TILE_WIDTH, TILE_HEIGHT, flip_h, flip_v);
    }

    pub fn draw_tile_rotated<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        tile: TileGraphic,
        x: i32,
        y: i32,
        angle: f64,
        around: Point,
    ) {
        let tile_x = tile as usize as i32 % TILE_COLUMNS;
        let tile_y = tile as usize as i32 / TILE_COLUMNS;
        let src_rect = Rect::new(tile_x * TILE_STRIDE, tile_y * TILE_STRIDE, TILE_WIDTH, TILE_HEIGHT);
        let dst_rect = Rect::new(x, y, TILE_WIDTH, TILE_HEIGHT);
        let _ = canvas.copy_ex(&self.tileset, src_rect, dst_rect, angle, Some(around), false, false);
    }

    pub fn draw_tile<RT: RenderTarget>(
        &mut self,
        canvas: &mut Canvas<RT>,
        tile: TileGraphic,
        x: i32,
        y: i32,
        flip_h: bool,
        flip_v: bool,
    ) {
        let tile_x = tile as usize as i32 % TILE_COLUMNS;
        let tile_y = tile as usize as i32 / TILE_COLUMNS;
        let src_rect = Rect::new(tile_x * TILE_STRIDE, tile_y * TILE_STRIDE, TILE_WIDTH, TILE_HEIGHT);
        let dst_rect = Rect::new(x, y, TILE_WIDTH, TILE_HEIGHT);
        let _ = canvas.copy_ex(&self.tileset, src_rect, dst_rect, 0.0, None, flip_h, flip_v);
    }
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

use crate::{TileGraphic, TilePainter, TILE_STRIDE};
use sdl2::render::{Canvas, RenderTarget};

pub struct Dungeon {}

impl Dungeon {
    pub fn new() -> Dungeon {
        Dungeon {}
    }

    pub fn draw<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, tile_painter: &mut TilePainter) {
        for y in 3..=8 {
            for x in 3..=8 {
                let tile = if x == 3 || x == 8 || y == 3 || y == 8 {
                    TileGraphic::WallTop
                } else if x > 3 && x < 8 && y == 4 {
                    TileGraphic::WallSide
                } else {
                    TileGraphic::Ground
                };
                tile_painter.draw_tile_shadowed(
                    canvas,
                    tile,
                    x * TILE_STRIDE,
                    y * TILE_STRIDE,
                    false,
                    false,
                );
            }
        }
        let tile = TileGraphic::CornerShadowTopLeft;
        tile_painter.draw_tile(canvas, tile, 4 * TILE_STRIDE, 5 * TILE_STRIDE, false, false);
        tile_painter.draw_tile(canvas, tile, 7 * TILE_STRIDE, 5 * TILE_STRIDE, true, false);
        tile_painter.draw_tile(canvas, tile, 4 * TILE_STRIDE, 7 * TILE_STRIDE, false, true);
        tile_painter.draw_tile(canvas, tile, 7 * TILE_STRIDE, 7 * TILE_STRIDE, true, true);
    }

    pub fn draw_shadows<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        tile_painter: &mut TilePainter,
    ) {
        for y in 5..=6 {
            let x = 4;
            let tile = TileGraphic::ShadowLeft;
            tile_painter.draw_tile(canvas, tile, x * TILE_STRIDE, y * TILE_STRIDE, false, false);
        }
        for x in 5..=7 {
            let y = 7;
            let tile = TileGraphic::ShadowBottom;
            tile_painter.draw_tile(canvas, tile, x * TILE_STRIDE, y * TILE_STRIDE, false, false);
        }
        let tile = TileGraphic::ShadowTopLeft;
        tile_painter.draw_tile(canvas, tile, 4 * TILE_STRIDE, 4 * TILE_STRIDE, false, false);
        let tile = TileGraphic::ShadowBottomLeft;
        tile_painter.draw_tile(canvas, tile, 4 * TILE_STRIDE, 7 * TILE_STRIDE, false, false);
    }
}

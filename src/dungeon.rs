use crate::{TileGraphic, TilePainter, TILE_STRIDE};
use sdl2::render::{Canvas, RenderTarget};

pub struct Dungeon {}

impl Dungeon {
    pub fn new() -> Dungeon {
        Dungeon {}
    }

    pub fn draw<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, tile_painter: &TilePainter) {
        for y in 0..20 {
            for x in 0..20 {
                let tile = if x >= 3 && x <= 8 && y >= 3 && y <= 8 {
                    if x == 3 || x == 8 || y == 3 || y == 8 {
                        TileGraphic::Wall
                    } else {
                        TileGraphic::Ground
                    }
                } else {
                    TileGraphic::WallBackground
                };
                tile_painter.draw_tile(canvas, tile, x * TILE_STRIDE, y * TILE_STRIDE);
            }
        }
    }
}

use crate::{Level, Terrain, TileGraphic, TilePainter, TILE_STRIDE};
use sdl2::render::{Canvas, RenderTarget};

#[derive(Clone, PartialEq, Debug)]
pub struct Fighter {
    pub tile: Option<TileGraphic>,
    pub x: i32,
    pub y: i32,
    pub flip_h: bool,
}

impl Fighter {
    pub const fn new(tile: TileGraphic, x: i32, y: i32) -> Fighter {
        Fighter {
            tile: Some(tile),
            x,
            y,
            flip_h: false,
        }
    }

    pub const fn dummy() -> Fighter {
        Fighter {
            tile: None,
            x: 0,
            y: 0,
            flip_h: false,
        }
    }

    pub fn step(&mut self, dx: i32, dy: i32, fighters: &mut [Fighter], level: &mut Level) {
        let (new_x, new_y) = (self.x + dx, self.y + dy);
        let mut hit_something = false;

        for hit_fighter in fighters
            .iter_mut()
            .filter(|fighter| fighter.x == new_x && fighter.y == new_y)
        {
            println!("Hit someone: {:?}", hit_fighter);
            hit_something = true;
        }

        let hit_terrain = level.get_terrain(new_x, new_y);
        if hit_terrain.unwalkable() {
            hit_something = true;
        }
        if hit_terrain == Terrain::Door {
            level.open_door(new_x, new_y);
        }

        if !hit_something {
            self.x = new_x;
            self.y = new_y;
        }
    }

    pub fn draw<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, tile_painter: &mut TilePainter) {
        if let Some(tile) = self.tile {
            tile_painter.draw_tile_shadowed(
                canvas,
                tile,
                self.x * TILE_STRIDE,
                self.y * TILE_STRIDE - TILE_STRIDE / 2,
                self.flip_h,
                false,
            );
        }
    }
}

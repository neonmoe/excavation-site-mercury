use crate::{Camera, Level, Terrain, TileGraphic, TilePainter, TILE_STRIDE};
use sdl2::render::{Canvas, RenderTarget};
use std::cell::RefCell;
use std::f32::consts::PI;

#[derive(Clone, Debug, Default)]
struct Animation {
    // Values applied to the sprite.
    flip_h: bool,
    offset_x: i32,
    offset_y: i32,
    width_inc: i32,
    height_inc: i32,

    // Animation data.
    move_from_x: i32,
    move_from_y: i32,
    move_progress: f32,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub tile: Option<TileGraphic>,
    pub x: i32,
    pub y: i32,
    animation: RefCell<Animation>,
}

impl PartialEq for Fighter {
    fn eq(&self, other: &Self) -> bool {
        self.tile == other.tile && self.x == other.x && self.y == other.y
    }
}

impl Fighter {
    pub fn new(tile: TileGraphic, x: i32, y: i32) -> Fighter {
        Fighter {
            tile: Some(tile),
            x,
            y,
            animation: RefCell::new(Animation::default()),
        }
    }

    pub fn dummy() -> Fighter {
        Fighter {
            tile: None,
            x: 0,
            y: 0,
            animation: RefCell::new(Animation::default()),
        }
    }

    pub fn animate(&self, delta_time: f32) {
        const STEP_ANIMATION_DURATION: f32 = 0.15;

        let mut animation = self.animation.borrow_mut();
        if animation.move_progress > 0.0 {
            animation.move_progress = (animation.move_progress - delta_time / STEP_ANIMATION_DURATION).max(0.0);

            // This function goes up a little bit, then down a bit more, then up a little bit again.
            // Kinda like the shape of M, except the middle dip is deeper than the two peaks.
            let f = |x: f32| 1.0 + ((2.7 * PI * x - 1.85 * PI).sin() - 0.5) * 0.025;

            let move_squish_width_ratio = f(animation.move_progress);
            let dx = animation.move_from_x - self.x;
            let dy = animation.move_from_y - self.y;
            animation.width_inc = (TILE_STRIDE as f32 * move_squish_width_ratio) as i32 - TILE_STRIDE;
            animation.height_inc = (TILE_STRIDE as f32 / move_squish_width_ratio) as i32 - TILE_STRIDE;
            animation.offset_x =
                ((dx as f32 * animation.move_progress) * TILE_STRIDE as f32) as i32 - (animation.width_inc) / 2;
            animation.offset_y = ((dy as f32 * animation.move_progress) * TILE_STRIDE as f32) as i32
                - (animation.height_inc) / 2
                - ((1.0 - move_squish_width_ratio).max(0.0) * 2.0 * TILE_STRIDE as f32) as i32;
        } else {
            animation.move_from_x = self.x;
            animation.move_from_y = self.y;
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

        let mut animation = self.animation.borrow_mut();
        animation.move_from_x = self.x;
        animation.move_from_y = self.y;
        animation.move_progress = 1.0;
        if !hit_something {
            self.x = new_x;
            self.y = new_y;
            if dx < 0 {
                animation.flip_h = true;
            } else if dx > 0 {
                animation.flip_h = false;
            }
        }
    }

    pub fn draw<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, tile_painter: &mut TilePainter, camera: &Camera) {
        if let Some(tile) = self.tile {
            let animation = self.animation.borrow();
            tile_painter.draw_tile_shadowed_ex(
                canvas,
                tile,
                self.x * TILE_STRIDE + animation.offset_x - camera.x,
                self.y * TILE_STRIDE - TILE_STRIDE / 2 + animation.offset_y - camera.y,
                (TILE_STRIDE + animation.width_inc) as u32,
                (TILE_STRIDE + animation.height_inc) as u32,
                animation.flip_h,
                false,
            );
        }
    }
}

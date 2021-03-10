use crate::{
    stats, Camera, GameLog, Level, LocalizableString, Name, Stats, Terrain, TileGraphic, TilePainter, TILE_STRIDE,
};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, RenderTarget};
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
    flying_time: f32,
    descent_progress: f32,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub name: Name,
    pub tile: Option<TileGraphic>,
    pub x: i32,
    pub y: i32,
    pub stats: Stats,
    animation: RefCell<Animation>,
}

impl PartialEq for Fighter {
    fn eq(&self, other: &Self) -> bool {
        self.tile == other.tile && self.x == other.x && self.y == other.y
    }
}

impl Fighter {
    pub fn new(name: Name, tile: TileGraphic, x: i32, y: i32, stats: Stats) -> Fighter {
        Fighter {
            name,
            tile: Some(tile),
            x,
            y,
            stats,
            animation: RefCell::new(Animation::default()),
        }
    }

    pub fn dummy() -> Fighter {
        Fighter {
            name: Name::Dummy,
            tile: None,
            x: 0,
            y: 0,
            stats: stats::DUMMY,
            animation: RefCell::new(Animation::default()),
        }
    }

    pub fn animate(&self, delta_time: f32) {
        let mut animation = self.animation.borrow_mut();
        if animation.move_progress > 0.0 {
            let duration = if self.stats.flying { 0.2 } else { 0.15 };
            animation.move_progress = (animation.move_progress - delta_time / duration).max(0.0);

            let dx = animation.move_from_x - self.x;
            let dy = animation.move_from_y - self.y;
            animation.offset_x = ((dx as f32 * animation.move_progress) * TILE_STRIDE as f32) as i32;
            animation.offset_y = ((dy as f32 * animation.move_progress) * TILE_STRIDE as f32) as i32;

            if !self.stats.flying {
                // This function goes up a little bit, then down a bit more, then up a little bit again.
                // Kinda like the shape of M, except the middle dip is deeper than the two peaks.
                let f = |x: f32| 1.0 + ((2.7 * PI * x - 1.85 * PI).sin() - 0.5) * 0.025;

                let move_squish_width_ratio = f(animation.move_progress);
                animation.width_inc = (TILE_STRIDE as f32 * move_squish_width_ratio) as i32 - TILE_STRIDE;
                animation.height_inc = (TILE_STRIDE as f32 / move_squish_width_ratio) as i32 - TILE_STRIDE;
                animation.offset_x -= (animation.width_inc) / 2;
                animation.offset_y -= (animation.height_inc) / 2
                    + ((1.0 - move_squish_width_ratio).max(0.0) * 2.0 * TILE_STRIDE as f32) as i32;
            }
        } else {
            animation.move_from_x = self.x;
            animation.move_from_y = self.y;
            animation.offset_x = 0;
            animation.offset_y = 0;
            animation.width_inc = 0;
            animation.height_inc = 0;
        }

        if self.stats.flying && self.stats.health > 0 {
            animation.flying_time += delta_time;
        } else if animation.descent_progress < 1.0 {
            animation.descent_progress = (animation.descent_progress + delta_time * 2.0).min(1.0);
        }
        animation.offset_y +=
            (((animation.flying_time * 4.0).sin() - 1.0) * 8.0 * (1.0 - animation.descent_progress)) as i32;
    }

    pub fn step(
        &mut self,
        dx: i32,
        dy: i32,
        fighters: &mut [Fighter],
        level: &mut Level,
        rng: &mut Pcg32,
        log: &mut GameLog,
        round: u64,
    ) {
        let (new_x, new_y) = (self.x + dx, self.y + dy);
        let mut hit_something = false;

        for hit_fighter in fighters
            .iter_mut()
            .filter(|fighter| fighter.x == new_x && fighter.y == new_y && fighter.stats.health > 0)
        {
            hit_something = !hit_fighter.walkable();
            hit_fighter.take_damage(&self, rng, log, round);
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
        if dx < 0 {
            animation.flip_h = true;
        } else if dx > 0 {
            animation.flip_h = false;
        }

        if !hit_something {
            self.x = new_x;
            self.y = new_y;
        }
    }

    fn take_damage(&mut self, from: &Fighter, rng: &mut Pcg32, log: &mut GameLog, round: u64) {
        let hit_roll = (rng.next_u32() % 6) as i32 + 1;
        let modifier = from.stats.arm - self.stats.leg;
        let hit_value = hit_roll + modifier;
        if hit_value > 0 {
            let damage = 1 + hit_value / 6;
            self.stats.health = (self.stats.health - damage).max(0);
            log.combat(
                round,
                LocalizableString::SomeoneAttackedSomeone {
                    attacker: from.name.clone(),
                    defender: self.name.clone(),
                    damage,
                    roll: hit_roll,
                    attacker_arm: from.stats.arm,
                    defender_leg: self.stats.leg,
                },
            );
            if self.stats.health == 0 {}
        }
    }

    fn walkable(&self) -> bool {
        self.stats.health == 0
    }

    pub fn draw<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        tile_painter: &mut TilePainter,
        camera: &Camera,
        dead_layer: bool,
        show_debug: bool,
    ) {
        if let Some(tile) = self.tile {
            let is_dead = self.stats.health == 0;
            if is_dead != dead_layer {
                return;
            }

            if show_debug {
                if is_dead {
                    canvas.set_draw_color(Color::RGB(0x11, 0x55, 0x11));
                } else {
                    canvas.set_draw_color(Color::RGB(0x44, 0xCC, 0x11));
                }
                let _ = canvas.draw_rect(Rect::new(
                    self.x * TILE_STRIDE - camera.x,
                    self.y * TILE_STRIDE - camera.y,
                    TILE_STRIDE as u32,
                    TILE_STRIDE as u32,
                ));
            }

            let animation = self.animation.borrow();
            let x = self.x * TILE_STRIDE - camera.x + animation.offset_x;
            let mut y = self.y * TILE_STRIDE - camera.y + animation.offset_y;
            if is_dead {
                tile_painter.draw_tile(canvas, tile.dead(), x, y, false, false);
            } else {
                y -= TILE_STRIDE / 4;
                tile_painter.draw_tile_shadowed_ex(
                    canvas,
                    tile,
                    x,
                    y,
                    (TILE_STRIDE + animation.width_inc) as u32,
                    (TILE_STRIDE + animation.height_inc) as u32,
                    animation.flip_h,
                    false,
                );
            }

            let gap = (4 - self.stats.max_health / 3).max(1);
            let health_area_width = TILE_STRIDE - 20 + self.stats.max_health * 3;
            let health_rect_width = health_area_width / self.stats.max_health;
            canvas.set_blend_mode(BlendMode::Blend);
            for i in 0..self.stats.max_health {
                if i >= self.stats.health {
                    canvas.set_draw_color(Color::RGBA(0xAA, 0xAA, 0xAA, 0xAA));
                } else if self.stats.health <= self.stats.max_health / 3 {
                    canvas.set_draw_color(Color::RGB(0xCC, 0x33, 0x22));
                } else if self.stats.health <= self.stats.max_health / 2 {
                    canvas.set_draw_color(Color::RGB(0xEE, 0xAA, 0x22));
                } else {
                    canvas.set_draw_color(Color::RGB(0x66, 0xCC, 0x33));
                }
                let health_rect_offset =
                    health_rect_width * i + (TILE_STRIDE - self.stats.max_health * health_rect_width) / 2;
                let health_rect = Rect::new(
                    x + health_rect_offset + gap / 2,
                    y - TILE_STRIDE / 8 - 2,
                    (health_rect_width - gap) as u32,
                    (TILE_STRIDE / 8) as u32,
                );
                let _ = canvas.fill_rect(health_rect);
            }
        }
    }
}

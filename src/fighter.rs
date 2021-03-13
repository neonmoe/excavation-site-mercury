use crate::{
    interface, stats, Camera, GameLog, Level, LocalizableString, Name, StatIncrease, Stats, Terrain, TileGraphic,
    TilePainter, TILE_STRIDE,
};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, RenderTarget};
use std::cell::RefCell;

#[derive(Clone, Debug)]
struct ParticleEffect {
    x: i32,
    y: i32,
    angle: f64,
    tile: TileGraphic,
    duration: f32,
    opacity: f32,
    shadowed: bool,
}

impl ParticleEffect {
    pub fn new(x: i32, y: i32, angle: f64, tile: TileGraphic, duration: f32, shadowed: bool) -> ParticleEffect {
        ParticleEffect {
            x,
            y,
            angle,
            tile,
            duration,
            opacity: 1.0,
            shadowed,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Animation {
    // Values applied to the sprite.
    flip_h: bool,
    offset_x: i32,
    offset_y: i32,
    width_inc: i32,
    height_inc: i32,
    particles: Vec<ParticleEffect>,

    // Animation data.
    move_from_x: i32,
    move_from_y: i32,
    move_progress: f32,
    flying_time: f32,
    descent_progress: f32,
}

#[derive(Clone, Debug, Default)]
struct Experience {
    arm: f32,
    leg: f32,
    finger: f32,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub id: usize,
    pub name: Name,
    pub tile: Option<TileGraphic>,
    pub x: i32,
    pub y: i32,
    pub stats: Stats,
    pub previously_hit_from: Option<(i32, i32)>,
    animation: RefCell<Animation>,
    experience: Option<Experience>,
}

impl PartialEq for Fighter {
    fn eq(&self, other: &Self) -> bool {
        self.tile == other.tile && self.x == other.x && self.y == other.y
    }
}

impl Fighter {
    pub fn new(id: usize, name: Name, tile: TileGraphic, x: i32, y: i32, stats: Stats, levels_up: bool) -> Fighter {
        Fighter {
            id,
            name,
            tile: Some(tile),
            x,
            y,
            stats,
            previously_hit_from: None,
            animation: RefCell::new(Animation::default()),
            experience: if levels_up { Some(Experience::default()) } else { None },
        }
    }

    pub fn dummy() -> Fighter {
        Fighter {
            id: 0,
            name: Name::Dummy,
            tile: None,
            x: 0,
            y: 0,
            stats: stats::DUMMY,
            previously_hit_from: None,
            animation: RefCell::new(Animation::default()),
            experience: None,
        }
    }

    pub fn position(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn is_animating(&self) -> bool {
        self.animation.borrow().move_progress > 0.0
    }

    pub fn animate(&self, delta_time: f32, level: &Level) {
        let exit_animation = level.get_terrain(self.x, self.y) == Terrain::Exit;
        let mut animation = self.animation.borrow_mut();

        for particle in &mut animation.particles {
            particle.opacity -= delta_time / particle.duration;
        }
        animation.particles.retain(|particle| particle.opacity > 0.0);

        if animation.move_progress > 0.0 {
            let duration = if exit_animation {
                0.35
            } else if self.stats.flying {
                0.3
            } else {
                0.15
            };
            animation.move_progress = (animation.move_progress - delta_time / duration).max(0.0);

            let dx = animation.move_from_x - self.x;
            let dy = animation.move_from_y - self.y;
            animation.offset_x = ((dx as f32 * animation.move_progress.min(1.0)) * TILE_STRIDE as f32) as i32;
            animation.offset_y = ((dy as f32 * animation.move_progress.min(1.0)) * TILE_STRIDE as f32) as i32;

            if !self.stats.flying {
                let f = |x: f32| 1.0 + (x * (4.0 - 4.0 * x)) * 0.05;
                let move_squish_width_ratio = f(animation.move_progress.min(1.0));
                animation.width_inc = (TILE_STRIDE as f32 * move_squish_width_ratio) as i32 - TILE_STRIDE;
                animation.height_inc = (TILE_STRIDE as f32 / move_squish_width_ratio) as i32 - TILE_STRIDE;
                animation.offset_x -= (animation.width_inc) / 2;
                animation.offset_y -=
                    (animation.height_inc) / 2 + ((move_squish_width_ratio - 1.0) * 6.0 * TILE_STRIDE as f32) as i32;
            }
        } else {
            animation.move_from_x = self.x;
            animation.move_from_y = self.y;
            animation.offset_x = 0;
            animation.offset_y = 0;
            animation.width_inc = 0;
            animation.height_inc = 0;
        }

        if self.stats.health > 0 {
            animation.offset_y -= TILE_STRIDE / 4;
        }

        if self.stats.flying && self.stats.health > 0 {
            animation.flying_time += delta_time;
        } else if animation.descent_progress < 1.0 {
            animation.descent_progress = (animation.descent_progress + delta_time * 2.0).min(1.0);
        }
        animation.offset_y +=
            (((animation.flying_time * 4.0).cos() - 1.0) * 8.0 * (1.0 - animation.descent_progress)) as i32;

        let scale = if exit_animation {
            animation.move_progress.min(1.0).sqrt()
        } else {
            1.0
        };
        let new_width_inc = (TILE_STRIDE as f32 + animation.width_inc as f32) * scale - TILE_STRIDE as f32;
        animation.offset_x += ((animation.width_inc as f32 - new_width_inc) / 2.0) as i32;
        animation.width_inc = new_width_inc as i32;
        let new_height_inc = (TILE_STRIDE as f32 + animation.height_inc as f32) * scale - TILE_STRIDE as f32;
        animation.offset_y += ((animation.height_inc as f32 - new_height_inc) / 2.0) as i32;
        animation.height_inc = new_height_inc as i32;
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
            hit_fighter.take_damage(&self, level, rng, log, round);
            hit_fighter.previously_hit_from = Some((-dx, -dy));

            if let Some(exp) = &mut self.experience {
                exp.arm += 1.0 / (10.0 + (self.stats.arm - 10) as f32 * 5.0);
                while exp.arm >= 1.0 {
                    exp.arm -= 1.0;
                    self.stats.arm += 1;
                    log.level_up(
                        round,
                        LocalizableString::StatIncreaseByTraining {
                            stat: StatIncrease::Arm,
                            name: self.name.clone(),
                        },
                    );
                }
            }
        }

        let hit_terrain = level.get_terrain(new_x, new_y);
        if hit_terrain.unwalkable() {
            hit_something = true;
        }
        if hit_terrain == Terrain::Door {
            level.open_door(new_x, new_y);
        } else if let Terrain::LockedDoor { roll_threshold } = hit_terrain {
            let roll = 1 + (rng.next_u32() % 6) as i32;
            let finger = self.stats.finger;
            if finger + roll >= roll_threshold {
                level.open_door(new_x, new_y);
                log.lockpicking(
                    round,
                    LocalizableString::DoorUnlocked {
                        roll_threshold,
                        roll,
                        finger,
                    },
                );

                if let Some(exp) = &mut self.experience {
                    exp.finger += 1.0 / (2.0 + (self.stats.finger - 10) as f32 * 2.0);
                    while exp.finger >= 1.0 {
                        exp.finger -= 1.0;
                        self.stats.finger += 1;
                        log.level_up(
                            round,
                            LocalizableString::StatIncreaseByTraining {
                                stat: StatIncrease::Finger,
                                name: self.name.clone(),
                            },
                        );
                    }
                }
            } else {
                log.lockpicking(
                    round,
                    LocalizableString::DoorUnlockingFailed {
                        roll_threshold,
                        roll,
                        finger,
                    },
                );
            }
        }

        let nth_fighter = fighters.iter().position(|f| f.stats == stats::DUMMY).unwrap_or(0);
        let anim_offset = nth_fighter as f32 / fighters.len() as f32;

        {
            let mut animation = self.animation.borrow_mut();
            animation.move_from_x = self.x;
            animation.move_from_y = self.y;
            animation.move_progress = 1.0 + anim_offset;
            if dx < 0 {
                animation.flip_h = true;
            } else if dx > 0 {
                animation.flip_h = false;
            }
        }

        if !hit_something {
            self.x = new_x;
            self.y = new_y;

            if let Some(exp) = &mut self.experience {
                exp.leg += 1.0 / (50.0 + (self.stats.leg - 10) as f32 * 50.0);
                while exp.leg >= 1.0 {
                    exp.leg -= 1.0;
                    self.stats.leg += 1;
                    log.level_up(
                        round,
                        LocalizableString::StatIncreaseByTraining {
                            stat: StatIncrease::Leg,
                            name: self.name.clone(),
                        },
                    );
                }
            }
        }
    }

    fn take_damage(&mut self, from: &Fighter, level: &mut Level, rng: &mut Pcg32, log: &mut GameLog, round: u64) {
        let hit_roll = (rng.next_u32() % 6) as i32 + 1;
        let modifier = from.stats.arm - self.stats.leg;
        let damage = if hit_roll >= -modifier {
            let damage = 1 + (hit_roll + modifier) / 6;
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

            if self.stats.health == 0 {
                log.combat(round, LocalizableString::SomeoneWasIncapacitated(self.name.clone()));
                if self.stats.treasure > 0 {
                    level.put_treasure(self.x, self.y, self.stats.treasure);
                }
            }

            damage
        } else {
            log.combat(
                round,
                LocalizableString::AttackMissed {
                    attacker: from.name.clone(),
                    defender: self.name.clone(),
                    roll: hit_roll,
                    attacker_arm: from.stats.arm,
                    defender_leg: self.stats.leg,
                },
            );

            0
        };

        self.spawn_hit_particles(damage);
    }

    fn spawn_hit_particles(&self, damage: i32) {
        let mut animation = self.animation.borrow_mut();
        for i in 0..damage.max(1) {
            animation.particles.push(ParticleEffect::new(
                0,
                TILE_STRIDE / 4 + i * 20 - damage.max(1) * 10,
                0.0,
                if damage > 0 {
                    TileGraphic::AttackHit
                } else {
                    TileGraphic::AttackMiss
                },
                0.75,
                true,
            ));
        }
    }

    pub fn cast_laser_cross(
        &self,
        rng: &mut Pcg32,
        fighters: &mut [Fighter],
        level: &mut Level,
        log: &mut GameLog,
        round: u64,
    ) {
        let (x0, y0, x1, y1) = self.spawn_laser_cross_particles(level);
        for fighter in fighters {
            if fighter.x == self.x && fighter.y >= y0 && fighter.y <= y1 {
                fighter.take_damage(&self, level, rng, log, round);
            } else if fighter.y == self.y && fighter.x >= x0 && fighter.x <= x1 {
                fighter.take_damage(&self, level, rng, log, round);
            }
        }
    }

    fn spawn_laser_cross_particles(&self, level: &Level) -> (i32, i32, i32, i32) {
        let mut animation = self.animation.borrow_mut();
        let (mut x0, mut y0, mut x1, mut y1) = (0, 0, 0, 0);
        while !level.get_terrain(self.x + x0, self.y + y0).unwalkable() {
            x0 -= 1;
        }
        while !level.get_terrain(self.x + x1, self.y + y0).unwalkable() {
            x1 += 1;
        }
        while !level.get_terrain(self.x + (x0 + x1) / 2, self.y + y0).unwalkable() {
            y0 -= 1;
        }
        while !level.get_terrain(self.x + (x0 + x1) / 2, self.y + y1).unwalkable() {
            y1 += 1;
        }

        for x in x0..=x1 {
            let (x, y) = (x * TILE_STRIDE, 0);
            animation
                .particles
                .push(ParticleEffect::new(x, y, 0.0, TileGraphic::LaserBeam, 0.2, false));
        }

        for y in y0..=y1 {
            let (x, y) = (0, y * TILE_STRIDE);
            animation
                .particles
                .push(ParticleEffect::new(x, y, 90.0, TileGraphic::LaserBeam, 0.2, false));
        }

        (self.x + x0, self.y + y0, self.x + x1, self.y + y1)
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
        selected: bool,
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

            if selected {
                let x = self.x * TILE_STRIDE - camera.x;
                let y = self.y * TILE_STRIDE - camera.y;
                tile_painter.draw_tile(canvas, TileGraphic::TileHighlight, x, y, false, false);
            }

            let animation = self.animation.borrow();
            let x = self.x * TILE_STRIDE - camera.x + animation.offset_x;
            let y = self.y * TILE_STRIDE - camera.y + animation.offset_y;
            if is_dead {
                tile_painter.draw_tile(canvas, tile.dead(), x, y, animation.flip_h, false);
            } else {
                let w = (TILE_STRIDE + animation.width_inc) as u32;
                let h = (TILE_STRIDE + animation.height_inc) as u32;
                tile_painter.draw_tile_shadowed_ex(canvas, tile, x, y, w, h, animation.flip_h, false);
            }
        }
    }

    pub fn draw_health<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, camera: &Camera) {
        let animation = self.animation.borrow();
        let x = self.x * TILE_STRIDE - camera.x + animation.offset_x;
        let y = self.y * TILE_STRIDE - camera.y + animation.offset_y;

        let gap = (4 - self.stats.max_health / 3).max(1);
        let health_area_width = TILE_STRIDE - 20 + self.stats.max_health * 3;
        let health_rect_width = health_area_width / self.stats.max_health;
        canvas.set_blend_mode(BlendMode::Blend);
        for i in 0..self.stats.max_health {
            if i >= self.stats.health {
                canvas.set_draw_color(interface::HEALTH_EMPTY);
            } else if self.stats.health <= self.stats.max_health / 3 {
                canvas.set_draw_color(interface::HEALTH_LOW);
            } else if self.stats.health <= self.stats.max_health * 2 / 3 {
                canvas.set_draw_color(interface::HEALTH_MEDIUM);
            } else {
                canvas.set_draw_color(interface::HEALTH_HIGH);
            }

            let health_rect_offset =
                health_rect_width * i + (TILE_STRIDE - self.stats.max_health * health_rect_width) / 2;
            let mut health_rect = Rect::new(
                x + health_rect_offset + gap / 2,
                y - TILE_STRIDE / 8 - 2,
                (health_rect_width - gap) as u32,
                (TILE_STRIDE / 8) as u32,
            );
            let _ = canvas.fill_rect(health_rect);

            canvas.set_draw_color(interface::HEALTH_BORDER);
            health_rect.offset(-1, -1);
            health_rect.resize(health_rect.width() + 2, health_rect.height() + 2);
            let _ = canvas.draw_rect(health_rect);
        }
    }

    pub fn draw_particles<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        tile_painter: &mut TilePainter,
        camera: &Camera,
    ) {
        let animation = self.animation.borrow();
        for particle in &animation.particles {
            // Note: setting alpha mod for each draw probably causes a
            // draw call per particle, which is bad, but hopefully it
            // won't become a real issue.
            tile_painter
                .tileset
                .set_alpha_mod((0xFF as f32 * particle.opacity) as u8);
            let x = self.x * TILE_STRIDE + particle.x - camera.x + animation.offset_x;
            let y = self.y * TILE_STRIDE + particle.y - camera.y + animation.offset_y;
            let center = Point::new(TILE_STRIDE / 2, TILE_STRIDE / 2);
            if particle.shadowed {
                // FIXME: Shadowed particles ignore angle, currently
                tile_painter
                    .shadow_tileset
                    .set_alpha_mod((0xFF as f32 * particle.opacity) as u8);
                tile_painter.draw_tile_shadowed(canvas, particle.tile, x, y, false, false);
            } else {
                tile_painter.draw_tile_rotated(canvas, particle.tile, x, y, particle.angle, center);
            }
        }
        tile_painter.tileset.set_alpha_mod(0xFF);
    }

    pub fn mouse_over(&self, camera: &Camera, mouse: Point) -> bool {
        let animation = self.animation.borrow();
        let x = self.x * TILE_STRIDE - camera.x + animation.offset_x;
        let y = self.y * TILE_STRIDE - camera.y + animation.offset_y;
        let width = (TILE_STRIDE + animation.width_inc) as u32;
        let height = (TILE_STRIDE + animation.height_inc) as u32;
        Rect::new(x, y, width, height).contains_point(mouse)
    }
}

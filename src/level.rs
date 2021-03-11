use crate::{enemy_ai, stats, Camera, EnemyAi, Name, Stats, TileGraphic, TilePainter, TILE_STRIDE};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget};
use std::cell::RefCell;
use std::collections::HashMap;

const LEVEL_WIDTH: usize = 128;
const LEVEL_HEIGHT: usize = 128;

pub const SPAWN_PLAYER: FighterSpawn = FighterSpawn {
    name: Name::Astronaut,
    tile: TileGraphic::Player,
    stats: stats::PLAYER,
    ai: None,
    x: 0,
    y: 0,
};

pub const SPAWN_SLIME: FighterSpawn = FighterSpawn {
    name: Name::Slime,
    tile: TileGraphic::Slime,
    stats: stats::SLIME,
    ai: Some(enemy_ai::SLIME),
    x: 0,
    y: 0,
};

pub const SPAWN_ROACH: FighterSpawn = FighterSpawn {
    name: Name::Roach,
    tile: TileGraphic::Roach,
    stats: stats::ROACH,
    ai: Some(enemy_ai::ROACH),
    x: 0,
    y: 0,
};

pub const SPAWN_ROCKMAN: FighterSpawn = FighterSpawn {
    name: Name::Rockman,
    tile: TileGraphic::Rockman,
    stats: stats::ROCKMAN,
    ai: Some(enemy_ai::ROCKMAN),
    x: 0,
    y: 0,
};

pub const SPAWN_SENTIENT_METAL: FighterSpawn = FighterSpawn {
    name: Name::SentientMetal,
    tile: TileGraphic::SentientMetal,
    stats: stats::SENTIENT_METAL,
    ai: Some(enemy_ai::SENTIENT_METAL),
    x: 0,
    y: 0,
};

#[derive(Clone, Debug)]
pub struct FighterSpawn {
    pub name: Name,
    pub tile: TileGraphic,
    pub stats: Stats,
    pub ai: Option<EnemyAi>,
    pub x: i32,
    pub y: i32,
}

impl FighterSpawn {
    const fn at_position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Terrain {
    Empty,
    Floor,
    Wall,
    Door,
    DoorOpen,
    Exit,
}

impl Terrain {
    pub const fn unwalkable(self) -> bool {
        match self {
            Terrain::Wall => true,
            Terrain::Door => true,
            _ => false,
        }
    }

    pub const fn enemies_avoid(self) -> bool {
        match self {
            Terrain::Door | Terrain::DoorOpen | Terrain::Empty | Terrain::Exit => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone, Debug)]
struct LevelAnimation {
    door_openings: HashMap<(i32, i32), f32>,
}

#[derive(Clone, Debug)]
pub struct Level {
    pub spawns: Vec<FighterSpawn>,
    pub line_of_sight_x: i32,
    pub line_of_sight_y: i32,
    terrain: [Terrain; LEVEL_WIDTH * LEVEL_HEIGHT],
    rooms: Vec<Rect>,

    /// Intended to only be used in the drawing functions, mutated by
    /// `.animate()`. In a RefCell, because this is "stateful" per
    /// say. If the game is loaded, this state wont persist.
    animation_state: RefCell<LevelAnimation>,
}

impl PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        self.terrain == other.terrain
    }
}

impl Level {
    pub fn new(rng: &mut Pcg32, difficulty: u32) -> Level {
        let mut terrain = [Terrain::Empty; LEVEL_WIDTH * LEVEL_HEIGHT];
        let mut rooms = Vec::new();

        fn terrain_mut(
            terrain: &mut [Terrain; LEVEL_WIDTH * LEVEL_HEIGHT],
            x: i32,
            y: i32,
        ) -> Result<&mut Terrain, ()> {
            if x >= 0 && x < LEVEL_WIDTH as i32 && y >= 0 && y < LEVEL_HEIGHT as i32 {
                Ok(&mut terrain[x as usize + y as usize * LEVEL_WIDTH])
            } else {
                Err(())
            }
        }

        fn put_room(terrain: &mut [Terrain; LEVEL_WIDTH * LEVEL_HEIGHT], room: Rect) -> Result<(), ()> {
            let terrain_rect = Rect::new(0, 0, LEVEL_WIDTH as u32, LEVEL_HEIGHT as u32);
            if terrain_rect.contains_rect(room) {
                // Ensure the floor space is empty
                for y in room.top()..room.bottom() {
                    for x in room.left()..room.right() {
                        if *terrain_mut(terrain, x, y)? != Terrain::Empty {
                            return Err(());
                        }
                    }
                }

                // Ensure there aren't walls that would result in double-wide walls
                let mut consecutive_walls = 0;
                for y in &[room.top() - 2, room.bottom() + 1] {
                    for x in room.left() - 1..=room.right() {
                        if let Ok(&mut Terrain::Wall) = terrain_mut(terrain, x, *y) {
                            consecutive_walls += 1;
                            if consecutive_walls >= 2 {
                                return Err(());
                            }
                        } else {
                            consecutive_walls = 0;
                        }
                    }
                }
                consecutive_walls = 0;
                for x in &[room.left() - 2, room.right() + 1] {
                    for y in room.top() - 1..=room.bottom() {
                        if let Ok(&mut Terrain::Wall) = terrain_mut(terrain, *x, y) {
                            consecutive_walls += 1;
                            if consecutive_walls >= 2 {
                                return Err(());
                            }
                        } else {
                            consecutive_walls = 0;
                        }
                    }
                }

                // Add the room tiles
                for y in room.top() - 1..=room.bottom() {
                    for x in room.left() - 1..=room.right() {
                        if x == room.left() - 1 || x == room.right() || y == room.top() - 1 || y == room.bottom() {
                            *terrain_mut(terrain, x, y)? = Terrain::Wall;
                        } else {
                            *terrain_mut(terrain, x, y)? = Terrain::Floor;
                        }
                    }
                }

                Ok(())
            } else {
                Err(())
            }
        }

        fn add_doors(
            rng: &mut Pcg32,
            terrain: &mut [Terrain; LEVEL_WIDTH * LEVEL_HEIGHT],
            rooms: &[Rect],
            room: Rect,
            dry_run: bool,
        ) -> Result<(), ()> {
            for neighbor in rooms {
                let shared_top = neighbor.top().max(room.top()) + 1;
                let shared_bottom = neighbor.bottom().min(room.bottom()) - 2;
                if shared_top < shared_bottom {
                    let y = (rng.next_u32() % (shared_bottom - shared_top) as u32) as i32 + shared_top;
                    if neighbor.right() == room.left() - 1 {
                        if dry_run {
                            return Ok(());
                        } else {
                            terrain[neighbor.right() as usize + y as usize * LEVEL_WIDTH] = Terrain::Door;
                        }
                    } else if neighbor.left() - 1 == room.right() {
                        if dry_run {
                            return Ok(());
                        } else {
                            terrain[room.right() as usize + y as usize * LEVEL_WIDTH] = Terrain::Door;
                        }
                    }
                }

                let shared_left = neighbor.left().max(room.left()) + 1;
                let shared_right = neighbor.right().min(room.right()) - 1;
                if shared_left < shared_right {
                    let x = (rng.next_u32() % (shared_right - shared_left) as u32) as i32 + shared_left;
                    if neighbor.bottom() == room.top() - 1 {
                        if dry_run {
                            return Ok(());
                        } else {
                            terrain[x as usize + neighbor.bottom() as usize * LEVEL_WIDTH] = Terrain::Door;
                        }
                    } else if neighbor.top() - 1 == room.bottom() {
                        if dry_run {
                            return Ok(());
                        } else {
                            terrain[x as usize + room.bottom() as usize * LEVEL_WIDTH] = Terrain::Door;
                        }
                    }
                }
            }

            if dry_run {
                Err(())
            } else {
                Ok(())
            }
        }

        let start_room_width = 8;
        let start_room_height = 5;
        let start_room_x = (LEVEL_WIDTH as u32 - start_room_width) as i32 / 2;
        let start_room_y = (LEVEL_HEIGHT as u32 - start_room_height) as i32 / 2;
        let start_room = Rect::new(start_room_x, start_room_y, start_room_width, start_room_height);
        put_room(&mut terrain, start_room).unwrap();
        rooms.push(start_room);

        let mut iterations = 0;
        while rooms.len() < 8 && iterations < 10_000 {
            iterations += 1;

            let originating_room = rooms[rng.next_u32() as usize % rooms.len()];
            let new_room_width = 4 + (rng.next_u32() % 5);
            let new_room_height = 4 + (rng.next_u32() % 2);
            let (dx, dy) = match rng.next_u32() % 4 {
                0 => (1, 0),
                1 => (-1, 0),
                2 => (0, 1),
                3 => (0, -1),
                _ => unreachable!(),
            };

            let new_room_x = if dx < 0 {
                originating_room.left() - new_room_width as i32 - 1
            } else if dx > 0 {
                originating_room.right() + 1
            } else {
                originating_room.left() + (rng.next_u32() % (originating_room.width() + new_room_width - 2)) as i32
                    - new_room_width as i32
                    + 1
            };

            let new_room_y = if dy < 0 {
                originating_room.top() - new_room_height as i32 - 1
            } else if dy > 0 {
                originating_room.bottom() + 1
            } else {
                originating_room.top() + (rng.next_u32() % (originating_room.height() + new_room_height - 2)) as i32
                    - new_room_height as i32
                    + 1
            };

            let new_room = Rect::new(new_room_x, new_room_y, new_room_width, new_room_height);
            let door_spots_available = add_doors(rng, &mut terrain, &rooms, new_room, true).is_ok();
            if door_spots_available && put_room(&mut terrain, new_room).is_ok() {
                let _ = add_doors(rng, &mut terrain, &rooms, new_room, false);
                rooms.push(new_room);
            }
        }

        let mut spawns = Vec::new();
        spawns.push(SPAWN_PLAYER.at_position(
            start_room.x + start_room.width() as i32 / 2,
            start_room.y + start_room.height() as i32 / 2,
        ));

        for room in rooms.iter().skip(1) {
            let mut occupied_spots = Vec::new();
            let spawned_enemies = room.width() / 3 + rng.next_u32() % (3 + difficulty / 2);
            'spawn_loop: for _ in 0..spawned_enemies {
                let x = (rng.next_u32() % room.width()) as i32 + room.x;
                let y = (rng.next_u32() % (room.height() - 1)) as i32 + room.y;

                for (x_, y_) in &occupied_spots {
                    if x == *x_ && y == *y_ {
                        continue 'spawn_loop;
                    }
                }

                let spawn = match rng.next_u32() % 10 + difficulty * 3 {
                    0..=7 => SPAWN_SLIME,
                    8..=12 => SPAWN_ROACH,
                    13..=15 => SPAWN_ROCKMAN,
                    16..=17 => SPAWN_SENTIENT_METAL,
                    _ => SPAWN_ROCKMAN,
                };
                spawns.push(spawn.at_position(x, y));
                occupied_spots.push((x, y));
            }
        }

        let start_room_center_x = start_room_x + start_room_width as i32 / 2;
        let start_room_center_y = start_room_y + start_room_height as i32 / 2;
        rooms.sort_unstable_by_key(|room| {
            let dx = room.x + room.width() as i32 / 2 - start_room_center_x;
            let dy = room.y + room.height() as i32 / 2 - start_room_center_y;
            dx * dx + dy * dy
        });
        let furthest_room = rooms.iter().nth_back(0).unwrap();
        let exit_x = furthest_room.x as usize + 1 + (rng.next_u32() % (furthest_room.width() - 2)) as usize;
        let exit_y = furthest_room.y as usize + 1 + (rng.next_u32() % (furthest_room.height() - 3)) as usize;
        terrain[exit_x + exit_y * LEVEL_WIDTH] = Terrain::Exit;

        let line_of_sight_x = spawns[0].x;
        let line_of_sight_y = spawns[0].y;

        Level {
            terrain,
            rooms,
            spawns,
            line_of_sight_x,
            line_of_sight_y,
            animation_state: RefCell::new(LevelAnimation::default()),
        }
    }

    pub fn room_center_in_pixel_space(&self, in_room_point: Point) -> Option<Point> {
        for room in &self.rooms {
            if room.contains_point(in_room_point) {
                let x = room.x * TILE_STRIDE + room.width() as i32 * TILE_STRIDE / 2;
                let y = room.y * TILE_STRIDE + room.height() as i32 * TILE_STRIDE / 2 - TILE_STRIDE;
                return Some(Point::new(x, y));
            }
        }
        None
    }

    pub fn open_door(&mut self, x: i32, y: i32) {
        if x >= 0 && x < LEVEL_WIDTH as i32 && y >= 0 && y < LEVEL_HEIGHT as i32 {
            if let Terrain::Door = self.terrain[x as usize + y as usize * LEVEL_WIDTH] {
                self.terrain[x as usize + y as usize * LEVEL_WIDTH] = Terrain::DoorOpen;
                self.animation_state.borrow_mut().door_openings.insert((x, y), 0.066);
            }
        }
    }

    pub fn get_terrain(&self, x: i32, y: i32) -> Terrain {
        if x < 0 || y < 0 || x >= LEVEL_WIDTH as i32 || y >= LEVEL_HEIGHT as i32 {
            Terrain::Empty
        } else {
            self.terrain[x as usize + y as usize * LEVEL_WIDTH]
        }
    }

    pub fn in_line_of_sight<RT: RenderTarget>(
        &self,
        x: i32,
        y: i32,
        canvas: &mut Canvas<RT>,
        camera: &Camera,
        show_debug: bool,
    ) -> bool {
        if x == self.line_of_sight_x && y == self.line_of_sight_y {
            return true;
        }

        let (target_x, target_y) = (x as f32 + 0.5, y as f32 + 0.5);
        let (mut cursor_x, mut cursor_y) = (self.line_of_sight_x as f32 + 0.5, self.line_of_sight_y as f32 + 0.5);
        let dx = target_x - cursor_x;
        let dy = target_y - cursor_y;
        let dl = (dx * dx + dy * dy).sqrt();
        let dx = dx / dl;
        let dy = dy / dl;

        if show_debug {
            canvas.set_draw_color(Color::RGBA(
                (0xDD as f32 + 0x11 as f32 * dx) as u8,
                0xFF,
                (0xDD as f32 + 0x11 as f32 * dy) as u8,
                0x88,
            ));
        }

        let mut iterations = 0;
        while (((target_x - cursor_x) * dx).signum() == 1.0 || ((target_y - cursor_y) * dy).signum() == 1.0)
            && iterations < 200
        {
            iterations += 1;
            if show_debug {
                let _ = canvas.draw_point(sdl2::rect::Point::new(
                    (cursor_x * TILE_STRIDE as f32) as i32 - camera.x,
                    (cursor_y * TILE_STRIDE as f32) as i32 - camera.y,
                ));
            }

            let distance_to_next_x = if dx >= 0.0 {
                1.0 - cursor_x.fract()
            } else if cursor_x.fract() == 0.0 {
                1.0
            } else {
                cursor_x.fract()
            };
            let distance_to_next_y = if dy >= 0.0 {
                1.0 - cursor_y.fract()
            } else if cursor_y.fract() == 0.0 {
                1.0
            } else {
                cursor_y.fract()
            };
            let d = distance_to_next_x.min(distance_to_next_y) + 0.1;
            cursor_x += dx * d;
            cursor_y += dy * d;
            let tile_x = cursor_x as i32;
            let tile_y = cursor_y as i32;
            if self.get_terrain(tile_x, tile_y).unwalkable() {
                return false;
            } else if tile_x == x && tile_y == y {
                return true;
            }
        }

        false
    }

    pub fn animate(&self, delta_seconds: f32) {
        let mut animation = self.animation_state.borrow_mut();
        animation.door_openings.retain(|_, v| {
            *v -= delta_seconds;
            *v > 0.0
        });
    }

    pub fn draw<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        tile_painter: &mut TilePainter,
        camera: &Camera,
        above_layer: bool,
        show_debug: bool,
        dark_fade: bool,
    ) {
        let offset_x = camera.x / TILE_STRIDE;
        let offset_y = camera.y / TILE_STRIDE;
        let (screen_width, screen_height) = canvas.output_size().unwrap();
        let tiles_x = screen_width as i32 / TILE_STRIDE + 2;
        let tiles_y = screen_height as i32 / TILE_STRIDE + 2;

        // Precalculate line of sight (if needed)
        let mut line_of_sight = Vec::with_capacity((tiles_x * tiles_y) as usize);
        if above_layer {
            for y in 0..tiles_y {
                let tile_y = y + offset_y;
                for x in 0..tiles_x {
                    let tile_x = x + offset_x;
                    line_of_sight.push(self.in_line_of_sight(tile_x, tile_y, canvas, camera, show_debug));
                }
            }
        }
        let in_line_of_sight = |x: i32, y: i32| {
            if x < 0 || y < 0 || x >= tiles_x || y >= tiles_y {
                false
            } else {
                line_of_sight[(x + y * tiles_x) as usize]
            }
        };

        for y in 0..tiles_y {
            let tile_y = y + offset_y;
            for x in 0..tiles_x {
                let tile_x = x + offset_x;
                let terrain = self.get_terrain(tile_x, tile_y);

                const NO_FLAGS: u32 = 0;
                const FLAG_SHDW: u32 = 1 << 1; // Will render with a shadow
                const FLAG_FLIP_H: u32 = 1 << 2; // Flip horizontally
                const FLAG_FLIP_V: u32 = 1 << 3; // Flip vertically
                const FLAG_FLIP_BOTH: u32 = FLAG_FLIP_H | FLAG_FLIP_V;

                let tiles: &[(TileGraphic, i32, i32, u32)] = match (
                    terrain,                              // tile at cursor
                    self.get_terrain(tile_x, tile_y + 1), // tile below cursor
                    self.get_terrain(tile_x + 1, tile_y), // tile right of cursor
                    self.get_terrain(tile_x, tile_y - 1), // tile above cursor
                    self.get_terrain(tile_x - 1, tile_y), // tile left of cursor
                    self.get_terrain(tile_x, tile_y + 2), // tile two tiles below cursor
                ) {
                    // Closed doors
                    (Terrain::Door, _, Terrain::Wall, _, Terrain::Wall, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::DoorClosed, 0, -TILE_STRIDE / 2, NO_FLAGS),
                    ],
                    (Terrain::Door, Terrain::Wall, _, Terrain::Wall, _, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::SideDoorClosed, 0, -TILE_STRIDE + 12, FLAG_SHDW),
                        (TileGraphic::SideDoorClosed, 0, 12, FLAG_SHDW), // For the shadow
                        (TileGraphic::WallTop, 0, 12, NO_FLAGS),
                    ],

                    // Open doors
                    (Terrain::DoorOpen, _, Terrain::Wall, _, Terrain::Wall, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::DoorOpen, 0, -TILE_STRIDE / 2, NO_FLAGS),
                    ],
                    (Terrain::DoorOpen, Terrain::Wall, _, Terrain::Wall, _, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::SideDoorOpen, 0, 0, NO_FLAGS),
                        (TileGraphic::WallTop, 0, 12, NO_FLAGS),
                    ],

                    // Tops of walls
                    (_, Terrain::Wall, _, _, _, _) => &[(TileGraphic::WallTop, 0, 0, NO_FLAGS)],
                    // Sides of walls
                    (Terrain::Wall, _, _, _, _, _) => &[(TileGraphic::WallSide, 0, 0, NO_FLAGS)],

                    // Floors (with varying corner shadows)
                    (Terrain::Floor, _, t, _, _, Terrain::Wall) if t != Terrain::Floor => &[
                        // Bottom-right
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::CornerShadowTopLeft, 0, 0, FLAG_FLIP_BOTH),
                    ],
                    (Terrain::Floor, _, _, _, t, Terrain::Wall) if t != Terrain::Floor => &[
                        // Bottom-left
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::CornerShadowTopLeft, 0, 0, FLAG_FLIP_V),
                    ],
                    (Terrain::Floor, _, t, Terrain::Wall, _, _) if t != Terrain::Floor => &[
                        // Top-right
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::CornerShadowTopLeft, 0, 0, FLAG_FLIP_H),
                    ],
                    (Terrain::Floor, _, _, Terrain::Wall, t, _) if t != Terrain::Floor => &[
                        // Top-left
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::CornerShadowTopLeft, 0, 0, NO_FLAGS),
                    ],
                    (Terrain::Floor, _, _, _, _, _) => &[(TileGraphic::Ground, 0, 0, NO_FLAGS)],
                    (Terrain::Exit, _, _, _, _, _) => &[(TileGraphic::LevelExit, 0, 0, NO_FLAGS)],

                    (_, _, _, _, _, _) => &[],
                };

                // The actual tile rendering
                for (mut tile, x_offset, mut y_offset, mut flags) in tiles.into_iter() {
                    if above_layer != tile.is_above() {
                        continue;
                    }

                    // Animate if needed
                    let key = (tile_x, tile_y);
                    if tile == TileGraphic::DoorOpen && self.animation_state.borrow().door_openings.contains_key(&key) {
                        tile = TileGraphic::DoorOpening;
                    } else if tile == TileGraphic::SideDoorOpen
                        && self.animation_state.borrow().door_openings.contains_key(&key)
                    {
                        tile = TileGraphic::SideDoorOpening;
                        y_offset = -TILE_STRIDE / 3;
                        flags |= FLAG_SHDW;
                    }

                    // Draw the tile
                    let x = tile_x as i32 * TILE_STRIDE + x_offset - camera.x;
                    let y = tile_y as i32 * TILE_STRIDE + y_offset - camera.y;
                    let flip_h = (flags & FLAG_FLIP_H) != 0;
                    let flip_v = (flags & FLAG_FLIP_V) != 0;
                    if (flags & FLAG_SHDW) != 0 {
                        tile_painter.draw_tile_shadowed(canvas, tile, x, y, flip_h, flip_v);
                    } else {
                        tile_painter.draw_tile(canvas, tile, x, y, flip_h, flip_v);
                    }
                }

                // Line of sight stuff
                if above_layer {
                    let mut current_tile_is_in_los = false;
                    'los_check: for y_ in 0..=2 {
                        for x_ in -1..=1 {
                            if in_line_of_sight(x + x_, y + y_) {
                                current_tile_is_in_los = true;
                                break 'los_check;
                            }
                        }
                    }
                    if !current_tile_is_in_los {
                        if dark_fade {
                            canvas.set_draw_color(Color::RGB(0x22, 0x22, 0x33));
                        } else {
                            canvas.set_draw_color(Color::RGB(0x44, 0x44, 0x44));
                        }
                    } else if dark_fade {
                        let dx = (tile_x - self.line_of_sight_x) as f32;
                        let dy = (tile_y - self.line_of_sight_y) as f32;
                        let alpha = (0xFF as f32 * ((dx * dx + dy * dy).sqrt() / 7.0).min(1.0).powf(2.0)) as u8;
                        canvas.set_draw_color(Color::RGBA(0x22, 0x22, 0x33, alpha));
                    }
                    if !current_tile_is_in_los || dark_fade {
                        let _ = canvas.fill_rect(Rect::new(
                            tile_x * TILE_STRIDE - camera.x,
                            tile_y * TILE_STRIDE - camera.y,
                            TILE_STRIDE as u32,
                            TILE_STRIDE as u32,
                        ));
                    }
                }

                // Debug rectangles
                if show_debug && terrain.unwalkable() {
                    canvas.set_draw_color(Color::RGB(0xCC, 0x44, 0x11));
                    let _ = canvas.draw_rect(Rect::new(
                        tile_x * TILE_STRIDE - camera.x,
                        tile_y * TILE_STRIDE - camera.y,
                        TILE_STRIDE as u32,
                        TILE_STRIDE as u32,
                    ));
                }
            }
        }
    }

    pub fn draw_shadows<RT: RenderTarget>(
        &self,
        canvas: &mut Canvas<RT>,
        tile_painter: &mut TilePainter,
        camera: &Camera,
    ) {
        let offset_x = camera.x / TILE_STRIDE;
        let offset_y = camera.y / TILE_STRIDE;
        let (screen_width, screen_height) = canvas.output_size().unwrap();
        let tiles_x = screen_width as i32 / TILE_STRIDE + 2;
        let tiles_y = screen_height as i32 / TILE_STRIDE + 2;

        for y in 0..tiles_y {
            let tile_y = y + offset_y;
            for x in 0..tiles_x {
                let tile_x = x + offset_x;

                let get_terrain = |x: i32, y: i32| {
                    if x < 0 || y < 0 || x >= LEVEL_WIDTH as i32 || y >= LEVEL_HEIGHT as i32 {
                        Terrain::Empty
                    } else {
                        self.terrain[x as usize + y as usize * LEVEL_WIDTH]
                    }
                };

                let tiles: &[TileGraphic] = match (
                    get_terrain(tile_x, tile_y),         // tile at cursor
                    get_terrain(tile_x, tile_y + 1),     // tile below cursor
                    get_terrain(tile_x, tile_y + 2),     // tile two tiles below cursor
                    get_terrain(tile_x - 1, tile_y),     // tile left of cursor
                    get_terrain(tile_x - 1, tile_y + 1), // tile below and left of cursor
                ) {
                    (Terrain::Floor, _, Terrain::Wall, Terrain::Wall, _) => &[TileGraphic::ShadowBottomLeft],
                    (Terrain::Floor, _, Terrain::Wall, _, _) => &[TileGraphic::ShadowBottom],
                    (Terrain::Floor, t, _, Terrain::Wall, _) if t != Terrain::Wall => &[TileGraphic::ShadowLeft],
                    (Terrain::Wall, _, _, Terrain::Wall, Terrain::Wall) => &[TileGraphic::ShadowTopLeft],

                    (_, _, _, _, _) => &[],
                };

                for tile in tiles {
                    let x = tile_x as i32 * TILE_STRIDE - camera.x;
                    let y = tile_y as i32 * TILE_STRIDE - camera.y;
                    tile_painter.draw_tile(canvas, *tile, x, y, false, false);
                }
            }
        }
    }
}

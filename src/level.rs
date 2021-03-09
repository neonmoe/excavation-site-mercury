use crate::{Camera, TileGraphic, TilePainter, TILE_STRIDE};
use rand_core::RngCore;
use rand_pcg::Pcg32;
use sdl2::render::{Canvas, RenderTarget};
use std::cell::RefCell;
use std::collections::HashMap;

const LEVEL_WIDTH: usize = 128;
const LEVEL_HEIGHT: usize = 128;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Terrain {
    Empty,
    Floor,
    Wall,
    Door,
    DoorOpen,
}

impl Terrain {
    pub const fn unwalkable(self) -> bool {
        match self {
            Terrain::Wall => true,
            Terrain::Door => true,
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
    terrain: [Terrain; LEVEL_WIDTH * LEVEL_HEIGHT],

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
    pub fn new(rng: &mut Pcg32) -> Level {
        let mut terrain = [Terrain::Empty; LEVEL_WIDTH * LEVEL_HEIGHT];

        let x0 = 2;
        let y0 = 2;
        let x1 = 6 + rng.next_u32() as usize % 8;
        let y1 = 6 + rng.next_u32() as usize % 5;
        for y in y0..=y1 {
            for x in x0..=x1 {
                terrain[x + y * LEVEL_WIDTH] = if (x == (x1 + x0) / 2 && y == y0) || (x == x1 && y == (y1 + y0) / 2) {
                    if cfg!(debug_assertions) {
                        print!("+");
                    }
                    Terrain::Door
                } else if x == x0 || x == x1 || y == y0 || y == y1 {
                    if cfg!(debug_assertions) {
                        print!("#");
                    }
                    Terrain::Wall
                } else {
                    if cfg!(debug_assertions) {
                        print!(".");
                    }
                    Terrain::Floor
                };
            }
            if cfg!(debug_assertions) {
                println!("");
            }
        }

        Level {
            terrain,
            animation_state: RefCell::new(LevelAnimation::default()),
        }
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
    ) {
        let offset_x = camera.x / TILE_STRIDE;
        let offset_y = camera.y / TILE_STRIDE;
        let (screen_width, screen_height) = canvas.output_size().unwrap();
        let tiles_x = screen_width as i32 / TILE_STRIDE + 1;
        let tiles_y = screen_height as i32 / TILE_STRIDE + 1;

        for y in 0..tiles_y {
            let tile_y = y + offset_y;
            for x in 0..tiles_x {
                let tile_x = x + offset_x;

                const NO_FLAGS: u32 = 0;
                const FLAG_SHDW: u32 = 1 << 1; // Will render with a shadow
                const FLAG_FLIP_H: u32 = 1 << 2; // Flip horizontally
                const FLAG_FLIP_V: u32 = 1 << 3; // Flip vertically
                const FLAG_FLIP_BOTH: u32 = FLAG_FLIP_H | FLAG_FLIP_V;

                let tiles: &[(TileGraphic, i32, i32, u32)] = match (
                    self.get_terrain(tile_x, tile_y),     // tile at cursor
                    self.get_terrain(tile_x, tile_y + 1), // tile below cursor
                    self.get_terrain(tile_x + 1, tile_y), // tile right of cursor
                    self.get_terrain(tile_x, tile_y - 1), // tile above cursor
                    self.get_terrain(tile_x - 1, tile_y), // tile left of cursor
                    self.get_terrain(tile_x, tile_y + 2), // tile two tiles below cursor
                ) {
                    // Closed doors
                    (Terrain::Door, _, Terrain::Wall, _, Terrain::Wall, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::DoorClosed, 0, -TILE_STRIDE / 2, FLAG_SHDW),
                        (TileGraphic::WallTop, TILE_STRIDE, -TILE_STRIDE, NO_FLAGS),
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
                        (TileGraphic::WallTop, TILE_STRIDE, -TILE_STRIDE, NO_FLAGS),
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

                    (_, _, _, _, _, _) => &[],
                };

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
        let tiles_x = screen_width as i32 / TILE_STRIDE + 1;
        let tiles_y = screen_height as i32 / TILE_STRIDE + 1;

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

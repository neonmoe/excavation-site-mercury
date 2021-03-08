use crate::{TileGraphic, TilePainter, TILE_STRIDE};
use sdl2::render::{Canvas, RenderTarget};

const LEVEL_WIDTH: usize = 128;
const LEVEL_HEIGHT: usize = 128;

#[derive(Clone, Copy, PartialEq)]
enum Terrain {
    Empty,
    Floor,
    Wall,
    Door,
    DoorOpening(f32),
    DoorOpen,
}

pub struct Dungeon {
    terrain: [Terrain; LEVEL_WIDTH * LEVEL_HEIGHT],
}

impl Dungeon {
    pub fn new() -> Dungeon {
        let mut terrain = [Terrain::Empty; LEVEL_WIDTH * LEVEL_HEIGHT];

        for y in 2..=8 {
            for x in 2..=10 {
                terrain[x + y * LEVEL_WIDTH] = if (x == 5 && y == 2) || (x == 10 && y == 6) {
                    if cfg!(debug_assertions) {
                        print!("+");
                    }
                    Terrain::Door
                } else if x == 2 || x == 10 || y == 2 || y == 8 {
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

        Dungeon { terrain }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        for terrain in &mut self.terrain {
            match terrain {
                Terrain::DoorOpening(f) => {
                    *f -= delta_seconds;
                    if *f <= 0.0 {
                        *terrain = Terrain::DoorOpen;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn draw<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, tile_painter: &mut TilePainter) {
        let offset_x = 0;
        let offset_y = 0;
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
                let get_terrain = |x: i32, y: i32| {
                    if x < 0 || y < 0 || x >= LEVEL_WIDTH as i32 || y >= LEVEL_HEIGHT as i32 {
                        Terrain::Empty
                    } else {
                        self.terrain[x as usize + y as usize * LEVEL_WIDTH]
                    }
                };

                let tiles: &[(TileGraphic, i32, i32, u32)] = match (
                    get_terrain(tile_x, tile_y),     // tile at cursor
                    get_terrain(tile_x, tile_y + 1), // tile below cursor
                    get_terrain(tile_x + 1, tile_y), // tile right of cursor
                    get_terrain(tile_x, tile_y - 1), // tile above cursor
                    get_terrain(tile_x - 1, tile_y), // tile left of cursor
                    get_terrain(tile_x, tile_y + 2), // tile two tiles below cursor
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

                    // Opening doors
                    (Terrain::DoorOpening(_), _, Terrain::Wall, _, Terrain::Wall, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::DoorOpening, 0, -TILE_STRIDE / 2, FLAG_SHDW),
                        (TileGraphic::WallTop, TILE_STRIDE, -TILE_STRIDE, NO_FLAGS),
                    ],
                    (Terrain::DoorOpening(_), Terrain::Wall, _, Terrain::Wall, _, _) => &[
                        (TileGraphic::Ground, 0, 0, NO_FLAGS),
                        (TileGraphic::SideDoorOpening, 0, -TILE_STRIDE + 12, FLAG_SHDW),
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
                        (TileGraphic::SideDoorClosed, 0, 0, NO_FLAGS),
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

                for (tile, x_offset, y_offset, flags) in tiles {
                    let x = x as i32 * TILE_STRIDE + x_offset;
                    let y = y as i32 * TILE_STRIDE + y_offset;
                    let flip_h = (flags & FLAG_FLIP_H) != 0;
                    let flip_v = (flags & FLAG_FLIP_V) != 0;
                    if (flags & FLAG_SHDW) != 0 {
                        tile_painter.draw_tile_shadowed(canvas, *tile, x, y, flip_h, flip_v);
                    } else {
                        tile_painter.draw_tile(canvas, *tile, x, y, flip_h, flip_v);
                    }
                }
            }
        }
    }

    pub fn draw_shadows<RT: RenderTarget>(&self, canvas: &mut Canvas<RT>, tile_painter: &mut TilePainter) {
        let offset_x = 0;
        let offset_y = 0;
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
                    let x = x as i32 * TILE_STRIDE;
                    let y = y as i32 * TILE_STRIDE;
                    tile_painter.draw_tile(canvas, *tile, x, y, false, false);
                }
            }
        }
    }
}

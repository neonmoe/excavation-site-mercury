//! Welcome to the spotty documentation of the internals of Excavation
//! Site Mercury, an entry to the 7DRLx17 game jam.
//!
//! # The documentation
//!
//! I've added some explanatory documentation for structs / functions
//! that aren't self-explanatory, but for the most part I haven't paid
//! attention to documentation, as I don't expect anyone to reuse this
//! code.
//!
//! # Roadmap
//!
//! Here's a list of features I have planned and what I've already
//! implemented:
//!
//! - ~~Dungeon rendering~~
//! - ~~De/serializable game struct~~
//! - ~~Entity/mover/mob base~~
//! - ~~On-screen log and localization~~
//! - ~~Enemy graphics and AI design~~
//!   - ~~Design: easiest enemy~~ (Classic slime enemy, only moves when attacked, towards attack.)
//!   - ~~Design: easy enemy~~ (Big insect? Grown in low gravity. Moves randomly, backs off when attacked.)
//!   - ~~Design: hard enemy~~ (Rock person. Hunts player until at low health, then backs to top-right corner.)
//!   - ~~Design: hardest enemy~~ (Flying bits of metal, very menacing. Hits in a + shape every 3 turns, avoids the player.)
//! - ~~Fighter stats inspection UI~~
//! - Enemy AI implementations (~~slime~~, ~~roach~~, ~~rockman~~, sentient metal)
//! - Attack effects
//!   - Required to implement sentient metal's ranged attack
//! - ~~Dungeon generation~~
//!   - ~~Design: abstract map struct for arranging rooms, for minimap rendering~~
//! - ~~Level progression (level exits and difficulty curve)~~
//! - Line of sight
//! - Player death handling, game over UI
//! - Stat increases at the start of each level
//! - Items
//!   - Design: item storage, use, pickup UI
//! - Locked rooms with treasure, openable with the Finger stat
//! - Hazard rooms to get treasure or circumvent enemies
//!   - Design: hazard + challenged stat combinations (Brain is still useless)
//!   - Design: dungeon generation rules to allow skipping enemy rooms
//!
//! And here's some "polish" features I'll add if I have the time:
//!
//! - Culling the zipped binary to <1MiB
//!   - Fonts have way too many glyphs, probably fixable with: https://github.com/fonttools/fonttools
//! - Quicksaves to the proper directory
//! - UI for saving and loading
//! - Better hop animation, attack animation, defend animation, dying animation
//!   - Design: generic animation struct
//! - Main menu
//! - Class choice UI (different sets of starting stats)
//! - Volume settings
//! - Sound effects
//! - Background loop (music or ambient sfx)
//! - Mouse control (pathfinding)

use fontdue::layout::LayoutSettings;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use std::time::{Duration, Instant};

mod text_painter;
pub use text_painter::{Font, Text, TextPainter};
mod tile_painter;
pub use tile_painter::{TileGraphic, TilePainter, TILE_STRIDE};
mod level;
pub use level::{FighterSpawn, Level, Terrain};
mod dungeon;
pub use dungeon::{Dungeon, DungeonEvent};
mod fighter;
pub use fighter::Fighter;
mod camera;
pub use camera::Camera;
pub mod stats;
pub use stats::Stats;
mod game_log;
pub use game_log::GameLog;
mod localization;
pub use localization::{Language, LocalizableString, Name};
pub mod enemy_ai;
pub use enemy_ai::EnemyAi;

static QUICK_SAVE_FILE: &str = "excavation-site-mercury-quicksave.bin";

// TODO: Catch panics and show a message box before crashing?
pub fn main() {
    #[cfg(feature = "env_logger")]
    env_logger::init();
    let initialization_start = Instant::now();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Excavation Site Mercury", 800, 600)
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut text_painter = TextPainter::new(&texture_creator).unwrap();
    let mut tile_painter = TilePainter::new(&texture_creator).unwrap();

    let mut dungeon = Dungeon::new((Instant::now() - initialization_start).subsec_nanos() as u64);
    let mut camera = Camera::new();
    let mut camera_position = dungeon
        .level()
        .room_center_in_pixel_space(dungeon.player().position())
        .unwrap();

    let mut show_debug = false;
    let mut mouse = Point::new(0, 0);
    let mut selected_fighter = None;

    log::info!("Game startup took {:?}.", Instant::now() - initialization_start);

    let mut frame_times = Vec::new();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let mut fts = frame_times.iter();
        let delta_seconds = if let (Some(latest), Some(previous)) = (fts.nth_back(0), fts.nth_back(0)) {
            let frame_duration: Duration = *latest - *previous;
            frame_duration.as_secs_f32()
        } else {
            0.01667
        };

        let mut should_select = false;
        let mut should_move = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => should_select = true,
                    MouseButton::Right => should_move = true,
                    _ => {}
                },

                Event::MouseMotion { x, y, .. } => {
                    mouse.x = x;
                    mouse.y = y;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } => {
                    log::info!("Quicksaving game to {}...", QUICK_SAVE_FILE);
                    match dungeon
                        .to_bytes()
                        .ok()
                        .and_then(|bytes| std::fs::write(QUICK_SAVE_FILE, bytes).ok())
                    {
                        Some(_) => log::info!("Game quicksaved to {}!", QUICK_SAVE_FILE),
                        None => log::error!("Failed quicksaving to {}.", QUICK_SAVE_FILE),
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F9),
                    ..
                } => {
                    log::info!("Loading quicksave from {}...", QUICK_SAVE_FILE);
                    match std::fs::read(QUICK_SAVE_FILE)
                        .ok()
                        .and_then(|bytes| Dungeon::from_bytes(&bytes).ok())
                    {
                        Some(loaded_dungeon) => {
                            dungeon = loaded_dungeon;
                            log::info!("Quicksave loaded from {}!", QUICK_SAVE_FILE);
                        }
                        None => {
                            log::error!("Error loading quicksave from {}.", QUICK_SAVE_FILE);
                        }
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } => show_debug = !show_debug,

                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    if show_debug {
                        dungeon = Dungeon::new((delta_seconds * 1_000_000.0) as u64)
                    }
                }

                Event::KeyDown {
                    keycode: Some(keycode), ..
                } => {
                    let event = match keycode {
                        Keycode::W | Keycode::K | Keycode::Up => Some(DungeonEvent::MoveUp),
                        Keycode::S | Keycode::J | Keycode::Down => Some(DungeonEvent::MoveDown),
                        Keycode::A | Keycode::H | Keycode::Left => Some(DungeonEvent::MoveLeft),
                        Keycode::D | Keycode::L | Keycode::Right => Some(DungeonEvent::MoveRight),
                        _ => None,
                    };
                    if dungeon.can_run_events() {
                        if let Some(event) = event {
                            dungeon.run_event(event);
                            dungeon.run_event(DungeonEvent::ProcessTurn);
                        }
                    }
                }
                _ => {}
            }
        }

        dungeon.try_load_next_level(false);

        if should_move {
            log::info!("TODO: Player should pathfind to mouse now");
        }

        if should_select {
            selected_fighter = dungeon
                .fighters()
                .iter()
                .filter(|fighter| fighter.mouse_over(&camera, mouse))
                .map(|fighter| fighter.id)
                .next();
        }

        dungeon.level().animate(delta_seconds);
        for fighter in dungeon.fighters() {
            fighter.animate(delta_seconds, dungeon.level());
        }

        if let Some(new_position) = dungeon.level().room_center_in_pixel_space(dungeon.player().position()) {
            camera_position = new_position;
        }

        let (width, height) = canvas.output_size().unwrap();
        let camera_target_x = camera_position.x - width as i32 / 2;
        let camera_target_y = camera_position.y - (height as i32 - 150) / 2;
        if dungeon.level_changed() {
            camera.x = camera_target_x;
            camera.y = camera_target_y;
        } else {
            camera.update(delta_seconds, camera_target_x, camera_target_y);
        }

        canvas.set_draw_color(Color::RGB(0x44, 0x44, 0x44));
        canvas.clear();

        dungeon
            .level()
            .draw(&mut canvas, &mut tile_painter, &camera, false, show_debug);
        for fighter in dungeon.fighters() {
            let selected = Some(fighter.id) == selected_fighter;
            fighter.draw(&mut canvas, &mut tile_painter, &camera, true, show_debug, selected);
        }
        for fighter in dungeon.fighters() {
            let selected = Some(fighter.id) == selected_fighter;
            fighter.draw(&mut canvas, &mut tile_painter, &camera, false, show_debug, selected);
        }
        dungeon.level().draw_shadows(&mut canvas, &mut tile_painter, &camera);
        dungeon
            .level()
            .draw(&mut canvas, &mut tile_painter, &camera, true, show_debug);

        dungeon.log().draw_messages(&mut canvas, &mut text_painter);

        if let Some(selected_fighter) = selected_fighter.and_then(|id| dungeon.get_fighter(id)) {
            let background_rect = Rect::new(width as i32 - 310, height as i32 - 20 - 16 * 12 - 150, 300, 150);
            canvas.set_draw_color(Color::RGBA(0x44, 0x44, 0x44, 0xAA));
            let _ = canvas.fill_rect(background_rect);

            let layout = LayoutSettings {
                x: (background_rect.x + 8) as f32,
                y: (background_rect.y + 8) as f32,
                max_width: Some((background_rect.width() - 16) as f32),
                max_height: Some((background_rect.height() - 16) as f32),
                ..LayoutSettings::default()
            };
            let fighter_description = LocalizableString::FighterDescription {
                id: selected_fighter.id,
                name: selected_fighter.name.clone(),
                max_health: selected_fighter.stats.max_health,
                health: selected_fighter.stats.health,
                arm: selected_fighter.stats.arm,
                leg: selected_fighter.stats.leg,
                finger: selected_fighter.stats.finger,
                brain: selected_fighter.stats.brain,
            }
            .localize(Language::English);
            canvas.set_clip_rect(background_rect);
            text_painter.draw_text(&mut canvas, &layout, &fighter_description);
            canvas.set_clip_rect(None);

            canvas.set_draw_color(Color::RGB(0x77, 0x88, 0x88));
            let _ = canvas.draw_rect(background_rect);
        }

        if show_debug {
            let color = Color::RGB(0xFF, 0xFF, 0x88);
            let title = Text(Font::RegularUi, 28.0, color, String::from("Excavation Site Mercury\n"));
            let info = Text(Font::RegularUi, 18.0, color, String::from("R to regenerate dungeon\nF5 to quicksave in working directory\nF9 to load quicksave from working directory\n"));
            let fps = frame_times.len();
            let fps = Text(Font::RegularUi, 18.0, color, format!("FPS: {}", fps));
            let layout = LayoutSettings::default();
            text_painter.draw_text(&mut canvas, &layout, &[title, info, fps]);
        }

        canvas.present();

        let now = Instant::now();
        frame_times.push(now);
        frame_times.retain(|i| now - *i <= Duration::from_secs(1));
    }
}

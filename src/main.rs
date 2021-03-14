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
//!   - ~~Design: easiest enemy (Classic slime enemy, only moves when attacked, towards attack.)~~
//!   - ~~Design: easy enemy (Big insect? Grown in low gravity. Moves randomly, backs off when attacked.)~~
//!   - ~~Design: hard enemy (Rock person. Hunts player until at low health, then backs to top-right corner.)~~
//!   - ~~Design: hardest enemy~~ (Flying bits of metal, very menacing. Hits in a + shape every 3 turns, avoids the player.)
//! - ~~Easy enemy AI implementations (slime, roach, rockman)~~
//! - ~~Fighter stats inspection UI~~
//! - ~~Dungeon generation~~
//!   - ~~Design: abstract map struct for arranging rooms, for minimap rendering~~
//! - ~~Level progression (level exits and difficulty curve)~~
//! - ~~Line of sight~~
//! - ~~Player death handling, game over UI~~
//! - ~~Treasure UI and treasure tiles~~
//! - ~~Final treasure for the end of the 4th level, and run finish UI~~
//! - ~~Stat increases at the start of each level~~
//! - ~~Culling the zipped binary to <1MiB~~
//!   - ~~Fonts have way too many glyphs, probably fixable with: https://github.com/fonttools/fonttools~~
//!   - ~~Smaller tileset png with pngquant~~
//! - ~~Locked rooms with treasure, openable with the Finger stat~~
//! - ~~Different types of wall and floor for 4th level~~
//! - ~~Attack effects~~
//!   - ~~Required to implement sentient metal's ranged attack~~
//! - ~~Sentient Metal AI~~
//!
//! Final day todos:
//! - ~~GitHub Actions~~
//! - ~~Keyboard controls for everything~~
//! - ~~Remove Brain, it isn't used~~
//! - ~~Inherent level ups: Leg from walking, Arm from fighting, Finger from unlocks~~
//! - Class choice UI (different sets of starting stats)
//! - Leaderboard for comparing treasure scores
//!
//! And here's the features I had planned but didn't have the time to
//! implement:
//!
//! - Hazard rooms to get treasure
//!   - Design: hazard + challenged stat combinations (Brain is still useless)
//! - Items
//!   - Design: item storage, use, pickup UI
//! - Quicksaves to the proper directory
//! - UI for saving and loading
//! - Better hop animation, attack animation, defend animation, dying animation
//!   - Design: generic animation struct
//! - Main menu
//! - Volume settings
//! - Sound effects
//! - Background loop (music or ambient sfx)
//! - Mouse control (pathfinding)
//! - Saving and loading UI

#![windows_subsystem = "windows"]

use fontdue::layout::LayoutSettings;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{Cursor, MouseButton, MouseWheelDirection, SystemCursor};
use sdl2::rect::Rect;
use std::time::{Duration, Instant};

mod text_painter;
pub use text_painter::{Font, Text, TextPainter};
mod tile_painter;
pub use tile_painter::{TileGraphic, TileLayer, TilePainter, TILE_STRIDE};
mod level;
pub use level::{FighterSpawn, Level, Terrain};
mod dungeon;
pub use dungeon::{Dungeon, DungeonEvent};
mod fighter;
pub use fighter::Fighter;
mod camera;
pub use camera::Camera;
pub mod stats;
pub use stats::{StatIncrease, Stats};
mod game_log;
pub use game_log::GameLog;
mod localization;
pub use localization::{Language, LocalizableString, Name};
pub mod enemy_ai;
pub use enemy_ai::EnemyAi;
pub mod interface;
pub use interface::UserInterface;
mod leaderboard;
pub use leaderboard::{Leaderboard, LeaderboardEntry};
mod leaderboard_server;

static QUICK_SAVE_FILE: &str = "excavation-site-mercury-quicksave.bin";

#[derive(PartialEq)]
enum Screen {
    InGame,
    Leaderboard,
}

// TODO: Catch panics and show a message box before crashing?
pub fn main() {
    #[cfg(feature = "env_logger")]
    env_logger::init();

    if std::env::args().find(|s| s == "--leaderboard-server").is_some() {
        leaderboard_server::serve();
        return;
    }

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
    let mut selected_fighter: Option<usize> = None;
    let mut ui = UserInterface::new();
    let mut leaderboard = Leaderboard::new();

    let mut screen = Screen::InGame;

    let normal_cursor = Cursor::from_system(SystemCursor::Arrow).unwrap();
    let hovering_cursor =
        Cursor::from_system(SystemCursor::Hand).unwrap_or_else(|_| Cursor::from_system(SystemCursor::Arrow).unwrap());

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

        let (width, height) = canvas.output_size().unwrap();
        let on_screen_fighters = dungeon.get_selectable_fighter_ids();
        if let Some(currently_selected) = selected_fighter {
            if !on_screen_fighters.contains(&currently_selected) {
                selected_fighter = None;
            }
        }

        ui.reset_for_new_frame();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => ui.mouse_left_pressed = true,
                    MouseButton::Right => ui.mouse_right_pressed = true,
                    _ => {}
                },

                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => {
                        ui.mouse_left_pressed = false;
                        ui.mouse_left_released = true;
                    }
                    MouseButton::Right => {
                        ui.mouse_right_pressed = false;
                        ui.mouse_right_released = true;
                    }
                    _ => {}
                },

                Event::MouseMotion { x, y, .. } => {
                    ui.mouse_position.x = x;
                    ui.mouse_position.y = y;
                }

                Event::MouseWheel { y, direction, .. } => {
                    ui.scroll = y * if direction == MouseWheelDirection::Flipped {
                        -1
                    } else {
                        1
                    };
                }

                Event::TextInput { text, .. } => {
                    ui.text_input = Some(text);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::F5),
                    ..
                } if screen == Screen::InGame => {
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
                } if screen == Screen::InGame => {
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
                } if screen == Screen::InGame => {
                    if show_debug {
                        dungeon = Dungeon::new((delta_seconds * 1_000_000_000.0) as u64)
                    }
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    ..
                } if screen == Screen::InGame => {
                    if let Some(selected_index) =
                        selected_fighter.and_then(|id| on_screen_fighters.iter().position(|id_| *id_ == id))
                    {
                        if selected_index + 1 >= on_screen_fighters.len() {
                            selected_fighter = None;
                        } else {
                            let new_index = selected_index + 1;
                            selected_fighter = Some(on_screen_fighters[new_index]);
                        }
                    } else {
                        selected_fighter = Some(on_screen_fighters[0]);
                    }
                }

                Event::KeyDown {
                    keycode: Some(keycode), ..
                } if screen == Screen::InGame => {
                    let event = match keycode {
                        Keycode::W | Keycode::K | Keycode::Up => Some(DungeonEvent::MoveUp),
                        Keycode::S | Keycode::J | Keycode::Down => Some(DungeonEvent::MoveDown),
                        Keycode::A | Keycode::H | Keycode::Left => Some(DungeonEvent::MoveLeft),
                        Keycode::D | Keycode::L | Keycode::Right => Some(DungeonEvent::MoveRight),
                        _ => None,
                    };
                    if let Some(event) = event {
                        if dungeon.can_run_events() {
                            dungeon.run_event(event);

                            let player = dungeon.player();
                            let (x, y) = (player.x, player.y);
                            let level = dungeon.level_mut();
                            level.line_of_sight_x = x;
                            level.line_of_sight_y = y;
                        }
                    }
                }

                Event::KeyUp {
                    keycode: Some(keycode), ..
                } => match keycode {
                    Keycode::Num1 => ui.released_buttons[0] = true,
                    Keycode::Num2 => ui.released_buttons[1] = true,
                    Keycode::Num3 => ui.released_buttons[2] = true,
                    Keycode::Num4 => ui.released_buttons[3] = true,
                    Keycode::Num5 => ui.released_buttons[4] = true,
                    Keycode::Num6 => ui.released_buttons[5] = true,
                    Keycode::Num7 => ui.released_buttons[6] = true,
                    Keycode::Num8 => ui.released_buttons[7] = true,
                    Keycode::Num9 => ui.released_buttons[8] = true,
                    _ => {}
                },
                _ => {}
            }
        }

        canvas.set_draw_color(interface::WINDOW_BACKGROUND);
        canvas.clear();

        match screen {
            Screen::Leaderboard => {
                leaderboard.run(delta_seconds, &mut canvas, &mut text_painter, &mut ui);
                if leaderboard.should_restart {
                    screen = Screen::InGame;
                    dungeon = Dungeon::new((delta_seconds * 1_000_000_000.0) as u64);
                    leaderboard.should_restart = false;
                } else if leaderboard.should_quit {
                    break 'running;
                }
            }

            Screen::InGame => {
                // TODO: All this in-game-specific stuff should be in its own module
                // Just lacking the time right now.

                dungeon.try_load_next_level(false);

                if ui.mouse_right_released {
                    log::info!("TODO: Player should pathfind to mouse now");
                }

                if ui.mouse_left_released {
                    selected_fighter = dungeon
                        .fighters()
                        .iter()
                        .filter(|fighter| fighter.mouse_over(&camera, ui.mouse_position))
                        .map(|fighter| fighter.id)
                        .next();
                }

                // Animate
                dungeon.level().animate(delta_seconds);
                for fighter in dungeon.fighters() {
                    fighter.animate(delta_seconds, dungeon.level());
                }

                // Update camera
                if let Some(new_position) = dungeon.level().room_center_in_pixel_space(dungeon.player().position()) {
                    camera_position = new_position;
                }
                let camera_target_x = camera_position.x - width as i32 / 2;
                let camera_target_y = camera_position.y - (height as i32 - 150) / 2;
                if dungeon.level_changed() {
                    camera.x = camera_target_x;
                    camera.y = camera_target_y;
                } else {
                    camera.update(delta_seconds, camera_target_x, camera_target_y);
                }

                // Draw the world
                dungeon.level().draw(
                    &mut canvas,
                    &mut tile_painter,
                    &camera,
                    TileLayer::BelowFighters,
                    show_debug,
                    false,
                    dungeon.level_nth() >= 3,
                );
                dungeon.level().draw_treasure(&mut canvas, &mut tile_painter, &camera);
                if dungeon.is_first_level() {
                    dungeon.level().draw_shadows(&mut canvas, &mut tile_painter, &camera);
                }
                for fighter in dungeon.fighters() {
                    let selected = Some(fighter.id) == selected_fighter;
                    fighter.draw(&mut canvas, &mut tile_painter, &camera, true, show_debug, selected);
                }
                for fighter in dungeon.fighters() {
                    let selected = Some(fighter.id) == selected_fighter;
                    fighter.draw(&mut canvas, &mut tile_painter, &camera, false, show_debug, selected);
                }
                for fighter in dungeon.fighters() {
                    fighter.draw_particles(&mut canvas, &mut tile_painter, &camera);
                }
                dungeon.level().draw(
                    &mut canvas,
                    &mut tile_painter,
                    &camera,
                    TileLayer::AboveFighters,
                    show_debug,
                    false,
                    dungeon.level_nth() >= 3,
                );
                for fighter in dungeon.fighters() {
                    fighter.draw_health(&mut canvas, &camera);
                }
                dungeon.level().draw(
                    &mut canvas,
                    &mut tile_painter,
                    &camera,
                    TileLayer::AboveAll,
                    show_debug,
                    !dungeon.is_first_level(),
                    dungeon.level_nth() >= 3,
                );

                // Draw the treasure counter
                {
                    let mineral_counter_bg = Rect::new(10, 10, 140, 46);
                    canvas.set_draw_color(interface::HUD_BACKGROUND_TRANSPARENT);
                    let _ = canvas.fill_rect(mineral_counter_bg);
                    canvas.set_draw_color(interface::HUD_BORDER);
                    let _ = canvas.draw_rect(mineral_counter_bg);
                    tile_painter.draw_tile(
                        &mut canvas,
                        TileGraphic::MineralCounter,
                        mineral_counter_bg.x - 12,
                        mineral_counter_bg.y - 6,
                        false,
                        false,
                    );

                    use fontdue::layout::HorizontalAlign;
                    use sdl2::pixels::Color;
                    let layout = LayoutSettings {
                        x: (mineral_counter_bg.x + 8) as f32,
                        y: (mineral_counter_bg.y + 8) as f32,
                        max_width: Some((mineral_counter_bg.width() - 16) as f32),
                        horizontal_align: HorizontalAlign::Right,
                        ..LayoutSettings::default()
                    };
                    let text = &[Text(
                        Font::RegularUi,
                        26.0,
                        Color::WHITE,
                        format!("{}", dungeon.treasure()),
                    )];
                    canvas.set_clip_rect(mineral_counter_bg);
                    text_painter.draw_text(&mut canvas, &layout, text);
                    canvas.set_clip_rect(None);
                }

                // Draw the combat log
                dungeon.log().draw_messages(&mut canvas, &mut text_painter);

                // Draw the fighter selection HUD
                if let Some(selected_fighter) = selected_fighter.and_then(|id| dungeon.get_fighter(id)) {
                    let background_rect = Rect::new(width as i32 - 310, height as i32 - 20 - 16 * 12 - 135, 300, 125);
                    canvas.set_draw_color(interface::HUD_BACKGROUND_TRANSPARENT);
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
                    }
                    .localize(Language::English);
                    canvas.set_clip_rect(background_rect);
                    text_painter.draw_text(&mut canvas, &layout, &fighter_description);
                    canvas.set_clip_rect(None);

                    canvas.set_draw_color(interface::HUD_BORDER);
                    let _ = canvas.draw_rect(background_rect);
                }

                // Draw the game over screen (if needed)
                if dungeon.is_game_over() {
                    let bg_width = 400;
                    let bg_height = 140;
                    let background_rect = Rect::new(
                        (width as i32 - bg_width as i32) / 2,
                        (height as i32 - bg_height as i32) / 2,
                        bg_width,
                        bg_height,
                    );
                    let game_over_string = LocalizableString::GameOver {
                        name: dungeon.player().name.clone(),
                    };
                    ui.text_box(&mut canvas, &mut text_painter, &game_over_string, background_rect, true);

                    let restart_button = Rect::new(
                        background_rect.x + 10,
                        background_rect.y + background_rect.height() as i32 - 46,
                        160,
                        36,
                    );
                    if ui.button(
                        &mut canvas,
                        &mut text_painter,
                        &LocalizableString::RestartButton,
                        restart_button,
                        true,
                    ) {
                        dungeon = Dungeon::new((delta_seconds * 1_000_000_000.0) as u64)
                    }

                    let submit_button = Rect::new(
                        restart_button.x + restart_button.width() as i32 + 10,
                        background_rect.y + background_rect.height() as i32 - 46,
                        160,
                        36,
                    );
                    if ui.button(
                        &mut canvas,
                        &mut text_painter,
                        &LocalizableString::SubmitToLeaderboardsButton,
                        submit_button,
                        true,
                    ) {
                        screen = Screen::Leaderboard;
                        leaderboard.submit_run(&dungeon);
                    }
                }

                // Draw the victory screen (if the final treasure has been found)
                if dungeon.final_treasure_found() && !dungeon.is_game_over() || show_debug {
                    let bg_width = 450;
                    let bg_height = 160;
                    let background_rect = Rect::new((width - 10 - bg_width) as i32, 10, bg_width, bg_height);
                    ui.text_box(
                        &mut canvas,
                        &mut text_painter,
                        &LocalizableString::Victory,
                        background_rect,
                        false,
                    );

                    let restart_button = Rect::new(
                        background_rect.x + 10,
                        background_rect.y + background_rect.height() as i32 - 46,
                        160,
                        36,
                    );
                    if ui.button(
                        &mut canvas,
                        &mut text_painter,
                        &LocalizableString::RestartButton,
                        restart_button,
                        true,
                    ) {
                        dungeon = Dungeon::new((delta_seconds * 1_000_000_000.0) as u64)
                    }

                    let submit_button = Rect::new(
                        restart_button.x + restart_button.width() as i32 + 10,
                        background_rect.y + background_rect.height() as i32 - 46,
                        160,
                        36,
                    );
                    if ui.button(
                        &mut canvas,
                        &mut text_painter,
                        &LocalizableString::SubmitToLeaderboardsButton,
                        submit_button,
                        true,
                    ) {
                        screen = Screen::Leaderboard;
                        leaderboard.submit_run(&dungeon);
                    }
                }

                // Draw the stat increase screen (if available)
                if dungeon.stat_increase_pending() {
                    canvas.set_draw_color(interface::SCREEN_FADE_COLOR);
                    let _ = canvas.fill_rect(Rect::new(0, 0, width, height));

                    let bg_width = 900.min(width - 20);
                    let bg_height = (380 + 2 * (600 - bg_width as i32).max(0) as u32).min(height - 10);
                    let background_rect = Rect::new(
                        (width - bg_width) as i32 / 2,
                        (height - bg_height) as i32 / 2,
                        bg_width,
                        bg_height,
                    );
                    ui.text_box(
                        &mut canvas,
                        &mut text_painter,
                        &LocalizableString::LevelUpMessage(dungeon.level_nth().max(1) as u32 - 1),
                        background_rect,
                        true,
                    );

                    use StatIncrease::*;
                    for (i, inc) in [Arm, Leg, Finger].iter().enumerate() {
                        let padding = 10;
                        let section_width = (background_rect.width() - padding as u32) / 3;
                        let section_rect = Rect::new(
                            background_rect.x + padding + (section_width as i32) * i as i32,
                            background_rect.y + 160,
                            section_width - padding as u32,
                            background_rect.height() - 170,
                        );
                        ui.text_box(
                            &mut canvas,
                            &mut text_painter,
                            &LocalizableString::StatInfo(*inc),
                            section_rect,
                            true,
                        );

                        let button_rect = Rect::new(
                            section_rect.x + 10,
                            section_rect.y + section_rect.height() as i32 - 46,
                            section_rect.width() - 20,
                            36,
                        );
                        if ui.button(
                            &mut canvas,
                            &mut text_painter,
                            &LocalizableString::IncreaseStatButton(*inc),
                            button_rect,
                            true,
                        ) {
                            dungeon.run_event(DungeonEvent::LevelUp(*inc));
                        }
                    }
                }
            }
        }

        // Draw debug information (if enabled)
        if show_debug {
            let color = interface::DEBUG_TEXT;
            let title = Text(Font::RegularUi, 28.0, color, String::from("Excavation Site Mercury\n"));
            let info = Text(Font::RegularUi, 18.0, color, String::from("R to regenerate dungeon\nF5 to quicksave in working directory\nF9 to load quicksave from working directory\n"));
            let fps = frame_times.len();
            let fps = Text(Font::RegularUi, 18.0, color, format!("FPS: {}", fps));
            let layout = LayoutSettings::default();
            text_painter.draw_text(&mut canvas, &layout, &[title, info, fps]);
        }

        // Update cursor
        if ui.hovering {
            hovering_cursor.set();
        } else {
            normal_cursor.set();
        }

        // Whew, done with this frame.
        canvas.present();

        let now = Instant::now();
        frame_times.push(now);
        frame_times.retain(|i| now - *i <= Duration::from_secs(1));
    }
}

pub fn move_towards(value: i32, target: i32, speed: i32) -> i32 {
    let direction_towards_value = (target - value).signum();
    let new_value = value + speed * direction_towards_value;
    if direction_towards_value != (target - new_value).signum() {
        // If the sign changes, this step would "shoot past", so just return the target.
        target
    } else {
        new_value
    }
}

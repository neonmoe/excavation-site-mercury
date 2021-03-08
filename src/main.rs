use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

mod text_painter;
pub use text_painter::{Font, TextPainter};

mod tile_painter;
pub use tile_painter::{TileGraphic, TilePainter, TILE_STRIDE};

mod level;
pub use level::{Level, Terrain};

mod dungeon;
pub use dungeon::{Dungeon, DungeonEvent};

mod fighter;
pub use fighter::Fighter;

pub fn main() {
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
    let mut dungeon = if let Ok(save) = std::fs::read("testingsave.bin") {
        Dungeon::from_bytes(&save).unwrap()
    } else {
        Dungeon::new(1234)
    };

    let mut frame_times = Vec::new();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
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
                    if let Some(event) = event {
                        dungeon.run_event(event);
                        dungeon.run_event(DungeonEvent::ProcessTurn);
                    }
                }
                _ => {}
            }
        }

        let mut fts = frame_times.iter();
        let delta_seconds = if let (Some(latest), Some(previous)) = (fts.nth_back(0), fts.nth_back(0)) {
            let frame_duration: Duration = *latest - *previous;
            frame_duration.as_secs_f32()
        } else {
            0.01667
        };
        dungeon.level().animate(delta_seconds);

        canvas.set_draw_color(Color::RGB(0x44, 0x44, 0x44));
        canvas.clear();

        dungeon.level().draw(&mut canvas, &mut tile_painter, false);
        for fighter in dungeon.fighters() {
            fighter.draw(&mut canvas, &mut tile_painter);
        }
        dungeon.level().draw(&mut canvas, &mut tile_painter, true);
        dungeon.level().draw_shadows(&mut canvas, &mut tile_painter);

        let color = Color::RGB(0xFF, 0xFF, 0x88);
        let title = (Font::RegularUi, 28.0, color, "Excavation Site Mercury\n");
        let fps = frame_times.len();
        let fps = (Font::RegularUi, 18.0, color, &*format!("FPS: {}", fps));
        text_painter.draw_text(&mut canvas, &[title, fps]);

        canvas.present();

        let now = Instant::now();
        frame_times.push(now);
        frame_times.retain(|i| now - *i <= Duration::from_secs(1));
    }
}

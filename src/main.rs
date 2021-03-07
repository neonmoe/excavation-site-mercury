use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::{Duration, Instant};

mod text_painter;
pub use text_painter::{Font, TextPainter};

mod tile_painter;
pub use tile_painter::{TileGraphic, TilePainter, TILE_STRIDE};

mod dungeon;
pub use dungeon::Dungeon;

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
    let tile_painter = TilePainter::new(&texture_creator).unwrap();
    let dungeon = Dungeon::new();

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
                _ => {}
            }
        }

        canvas.clear();

        dungeon.draw(&mut canvas, &tile_painter);
        tile_painter.draw_tile(
            &mut canvas,
            TileGraphic::Player,
            5 * TILE_STRIDE,
            5 * TILE_STRIDE,
        );

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

use crate::{
    interface, leaderboard_server, move_towards, Dungeon, Font, Language, LocalizableString, Text, TextPainter,
    UserInterface,
};
use bincode::config::DefaultOptions;
use bincode::Options;
use fontdue::layout::{HorizontalAlign, LayoutSettings};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

const SERVER_ADDRESS: &str = "excavationsitemercury.neon.moe:8582";
const VALID_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn valid_name_character(c: char) -> bool {
    VALID_CHARS.contains(c)
}

pub fn valid_name(name: [char; 3]) -> bool {
    valid_name_character(name[0]) && valid_name_character(name[1]) && valid_name_character(name[2])
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub struct LeaderboardEntry {
    pub name: [char; 3],
    pub treasure: i32,
    pub rounds: Option<u64>,
    pub size: usize,
}

pub struct Leaderboard {
    pub should_quit: bool,
    pub should_restart: bool,
    entries: Vec<LeaderboardEntry>,
    highlighted_entry: Option<LeaderboardEntry>,
    scroll_offset: i32,
    scroll_offset_target: i32,
    pending_run: Option<([char; 3], usize, Vec<u8>)>,
    error_message: Option<String>,
}

impl Leaderboard {
    pub fn new() -> Leaderboard {
        Leaderboard {
            should_quit: false,
            should_restart: false,
            entries: Vec::new(),
            highlighted_entry: None,
            scroll_offset: 0,
            scroll_offset_target: 0,
            pending_run: None,
            error_message: None,
        }
    }

    pub fn submit_run(&mut self, dungeon: &Dungeon) {
        let name = [' ', ' ', ' '];
        let dungeon_bytes = dungeon.to_bytes().unwrap();
        self.highlighted_entry = Some(LeaderboardEntry {
            name,
            treasure: dungeon.treasure(),
            rounds: if dungeon.is_game_over() {
                None
            } else {
                Some(dungeon.round())
            },
            size: dungeon_bytes.len(),
        });
        self.pending_run = Some((name, 0, dungeon_bytes));
    }

    fn send_run(&mut self) {
        if let Some((name, _, dungeon_bytes)) = self.pending_run.take() {
            if let Some(highlighted_entry) = &mut self.highlighted_entry {
                highlighted_entry.name = name;
            }
            if let Err(LeaderboardError::Server(message)) = upload_run(name, &dungeon_bytes) {
                self.error_message = Some(message);
            }
            self.entries = download_runs().unwrap_or_else(|_| Vec::new());
        }
    }

    pub fn run<RT: RenderTarget>(
        &mut self,
        delta_seconds: f32,
        canvas: &mut Canvas<RT>,
        text_painter: &mut TextPainter,
        ui: &mut UserInterface,
    ) {
        let (width, height) = canvas.output_size().unwrap();

        // Show the error message if there is one
        if let Some(error_message) = &self.error_message {
            let layout = LayoutSettings {
                x: 20.0,
                y: 20.0,
                max_width: Some((width - 20).min(600) as f32),
                ..LayoutSettings::default()
            };
            text_painter.draw_text(
                canvas,
                &layout,
                &[
                    Text(
                        Font::RegularUi,
                        24.0,
                        Color::WHITE,
                        String::from("Error during submission. Server response:\n"),
                    ),
                    Text(Font::RegularUi, 24.0, Color::YELLOW, error_message.clone()),
                ],
            );
            return;
        }

        // Show the name prompt when there's a pending run
        if let Some((pending_name, index, _)) = self.pending_run {
            if let (Some(input), Some((ref mut pending_name, ref mut index, _))) =
                (&ui.text_input, &mut self.pending_run)
            {
                for c in input
                    .chars()
                    .map(|c| c.to_ascii_uppercase())
                    .filter(|c| valid_name_character(*c))
                {
                    if *index < 3 {
                        pending_name[*index] = c;
                        *index += 1;
                    }
                }
            }

            ui.text_box(
                canvas,
                text_painter,
                &LocalizableString::NameInputInfo,
                Rect::new(width as i32 / 2 - 230, height as i32 / 2 - 180, 460, 360),
                false,
            );

            for (i, c) in pending_name.iter().enumerate() {
                let layout = LayoutSettings {
                    x: (width as i32 / 2 + i as i32 * 100 - 200) as f32,
                    y: (height as i32 / 2 - 60) as f32,
                    max_width: Some(200.0),
                    horizontal_align: HorizontalAlign::Center,
                    ..LayoutSettings::default()
                };
                text_painter.draw_text(
                    canvas,
                    &layout,
                    &LocalizableString::Character(*c, 100.0, Color::WHITE).localize(Language::English),
                );

                let underscore_color = if i == index {
                    Color::WHITE
                } else {
                    Color::RGB(0x77, 0x77, 0x77)
                };
                let layout = LayoutSettings {
                    x: (width as i32 / 2 + i as i32 * 100 - 200) as f32,
                    y: (height as i32 / 2 - 50) as f32,
                    max_width: Some(200.0),
                    horizontal_align: HorizontalAlign::Center,
                    ..LayoutSettings::default()
                };
                text_painter.draw_text(
                    canvas,
                    &layout,
                    &LocalizableString::Character('_', 100.0, underscore_color).localize(Language::English),
                );
            }

            if ui.button(
                canvas,
                text_painter,
                &LocalizableString::BigConfirmButton,
                Rect::new((width as i32 - 200) / 2, height as i32 / 2 + 110, 200, 50),
                valid_name(pending_name),
            ) {
                self.send_run();
            }

            if ui.button(
                canvas,
                text_painter,
                &LocalizableString::EraseButton,
                Rect::new((width as i32 - 200) / 2 + 220, height as i32 / 2 + 115, 95, 40),
                valid_name(pending_name),
            ) {
                if let Some((ref mut name, ref mut index, _)) = &mut self.pending_run {
                    *name = [' ', ' ', ' '];
                    *index = 0;
                }
            }
            return;
        }

        // The actual leaderboards UI
        ui.text(canvas, text_painter, &LocalizableString::LeaderboardsHeader, 10, 10);

        let extra_space = (width as i32 - 800).max(0);
        let margin = 10;
        let name_x = margin;
        let treasure_x = name_x + 168 + extra_space / 3;
        let rounds_x = treasure_x + 295 + extra_space / 3;

        ui.text(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsTitleName,
            name_x,
            50,
        );
        ui.text(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsTitleTreasure,
            treasure_x,
            50,
        );
        ui.text(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsTitleRounds,
            rounds_x,
            50,
        );

        if self.entries.is_empty() {
            ui.text(
                canvas,
                text_painter,
                &LocalizableString::LeaderboardsEmpty,
                (width as i32 - 300) / 2,
                height as i32 / 2,
            );
        } else {
            let scroll_width = 20;
            let padding = 8;
            let row_height = 20 + padding * 2;
            let entries_start_y = 70 + padding;
            let entries_end_y = height as i32 - (70 + padding);
            let entries_height = entries_end_y - entries_start_y;
            canvas.set_clip_rect(Rect::new(
                margin,
                entries_start_y,
                width - margin as u32,
                entries_height as u32,
            ));

            self.scroll_offset_target += ui.scroll * row_height * 3 / 2;
            self.scroll_offset_target = self
                .scroll_offset_target
                .max(entries_height - row_height * self.entries.len() as i32)
                .min(0);
            self.scroll_offset = move_towards(
                self.scroll_offset,
                self.scroll_offset_target,
                (20.0 * (self.scroll_offset_target - self.scroll_offset).abs().max(30) as f32 * delta_seconds) as i32,
            );

            let mut y = entries_start_y;
            for (i, entry) in self.entries.iter().enumerate() {
                if y + self.scroll_offset + row_height < entries_start_y {
                    y += row_height;
                    continue;
                }

                canvas.set_draw_color(if self.highlighted_entry.filter(|e| e == entry).is_some() {
                    interface::ROW_BACKGROUND_HIGHLIGHT
                } else if i % 2 == 0 {
                    interface::ROW_BACKGROUND
                } else {
                    interface::ROW_BACKGROUND_ALT
                });
                let _ = canvas.fill_rect(Rect::new(
                    name_x,
                    y + self.scroll_offset,
                    width - margin as u32 * 2 - scroll_width - 5,
                    row_height as u32,
                ));

                ui.text(
                    canvas,
                    text_painter,
                    &LocalizableString::LeaderboardsName(entry.name),
                    name_x + padding,
                    y + padding + self.scroll_offset,
                );
                ui.text(
                    canvas,
                    text_painter,
                    &LocalizableString::LeaderboardsTreasure(entry.treasure),
                    treasure_x + padding,
                    y + padding + self.scroll_offset,
                );
                ui.text(
                    canvas,
                    text_painter,
                    &LocalizableString::LeaderboardsRounds(entry.rounds),
                    rounds_x + padding,
                    y + padding + self.scroll_offset,
                );

                y += row_height;
                if y + self.scroll_offset > entries_end_y {
                    break;
                }
            }

            // Scroll background
            canvas.set_draw_color(interface::SCROLL_BACKGROUND);
            let _ = canvas.fill_rect(Rect::new(
                width as i32 - margin - scroll_width as i32,
                entries_start_y,
                scroll_width,
                (entries_end_y - entries_start_y) as u32,
            ));

            // Scroll handle
            canvas.set_draw_color(interface::SCROLL_HANDLE);
            let scroll_y =
                entries_start_y - entries_height * self.scroll_offset / row_height / self.entries.len() as i32;
            let _ = canvas.fill_rect(Rect::new(
                width as i32 - margin - scroll_width as i32,
                scroll_y,
                scroll_width,
                (entries_height * entries_height / row_height / self.entries.len() as i32).max(30) as u32,
            ));

            canvas.set_clip_rect(None);
        }

        // Restart, quit buttons
        let restart_width = 120;
        let quit_width = 100;
        if ui.button(
            canvas,
            text_painter,
            &LocalizableString::RestartButton,
            Rect::new(
                (width as i32 - (restart_width + 20 + quit_width) as i32) / 2 - 10,
                height as i32 - margin - 50,
                restart_width,
                40,
            ),
            true,
        ) {
            self.should_restart = true;
        }

        if ui.button(
            canvas,
            text_painter,
            &LocalizableString::QuitButton,
            Rect::new(
                (width as i32 - (restart_width + 20 + quit_width) as i32) / 2 + restart_width as i32 + 10,
                height as i32 - margin - 50,
                restart_width,
                40,
            ),
            true,
        ) {
            self.should_quit = true;
        }

        // Sorting buttons
        if ui.button(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsSortByButton,
            Rect::new(name_x + 58, 49, 105, 22),
            true,
        ) {
            self.entries.sort_by(|a, b| {
                if a.name[0] == b.name[0] && a.name[1] == b.name[1] {
                    a.name[2].cmp(&b.name[2])
                } else if a.name[0] == b.name[0] {
                    a.name[1].cmp(&b.name[1])
                } else {
                    a.name[0].cmp(&b.name[0])
                }
            });
        }

        if ui.button(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsSortByButton,
            Rect::new(treasure_x + 185, 49, 105, 22),
            true,
        ) {
            self.entries.sort_by(|a, b| b.treasure.cmp(&a.treasure));
        }

        if ui.button(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsSortByButton,
            Rect::new(rounds_x + 217, 49, 105, 22),
            true,
        ) {
            self.entries.sort_by(|a, b| match (a.rounds, b.rounds) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            });
        }
    }
}

pub fn upload_run(name: [char; 3], dungeon_bytes: &[u8]) -> Result<(), LeaderboardError> {
    let mut stream = TcpStream::connect(SERVER_ADDRESS)?;
    stream.write_all(leaderboard_server::UPLOAD_MAGIC_STRING.as_bytes())?;
    stream.write_all(&['>' as u8])?;
    stream.write_all(&[name[0] as u8, name[1] as u8, name[2] as u8])?;
    stream.write_all(&['<' as u8])?;
    stream.write_all(dungeon_bytes)?;
    let _ = stream.shutdown(Shutdown::Write);
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    if response == "OK." {
        Ok(())
    } else {
        Err(LeaderboardError::Server(response))
    }
}

fn download_runs() -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
    let mut stream = TcpStream::connect(SERVER_ADDRESS)?;
    stream.write_all(leaderboard_server::DOWNLOAD_MAGIC_STRING.as_bytes())?;
    let mut entries_bytes = Vec::with_capacity(10_000);
    stream.read_to_end(&mut entries_bytes)?;
    let entries = Options::deserialize(DefaultOptions::new(), &entries_bytes)?;
    Ok(entries)
}

#[derive(Debug)]
pub enum LeaderboardError {
    Io(std::io::Error),
    Bincode(bincode::Error),
    Server(String),
}

impl From<std::io::Error> for LeaderboardError {
    fn from(err: std::io::Error) -> Self {
        LeaderboardError::Io(err)
    }
}

impl From<bincode::Error> for LeaderboardError {
    fn from(err: bincode::Error) -> Self {
        LeaderboardError::Bincode(err)
    }
}

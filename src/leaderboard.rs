use crate::{interface, move_towards, LocalizableString, TextPainter, UserInterface};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq)]
pub struct LeaderboardEntry {
    pub name: [char; 3],
    pub treasure: i32,
    pub rounds: Option<u64>,
}

pub struct Leaderboard {
    pub should_quit: bool,
    pub should_restart: bool,
    entries: Vec<LeaderboardEntry>,
    highlighted_entry: Option<LeaderboardEntry>,
    scroll_offset: i32,
    scroll_offset_target: i32,
}

impl Leaderboard {
    pub fn new() -> Leaderboard {
        let mut entries = vec![
            LeaderboardEntry {
                name: ['F', 'O', 'O'],
                treasure: 123,
                rounds: None,
            },
            LeaderboardEntry {
                name: ['B', 'A', 'R'],
                treasure: 321,
                rounds: Some(20),
            },
            LeaderboardEntry {
                name: ['A', 'S', 'D'],
                treasure: 0,
                rounds: Some(10),
            },
            LeaderboardEntry {
                name: ['Q', 'W', 'E'],
                treasure: -1,
                rounds: None,
            },
        ];
        for _ in 0..2 {
            entries.extend(&entries.clone());
        }

        let highlighted_entry = Some(entries[2].clone());

        Leaderboard {
            should_quit: false,
            should_restart: false,
            entries,
            highlighted_entry,
            scroll_offset: 0,
            scroll_offset_target: 0,
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
        let scroll_y = entries_start_y - entries_height * self.scroll_offset / row_height / self.entries.len() as i32;
        let _ = canvas.fill_rect(Rect::new(
            width as i32 - margin - scroll_width as i32,
            scroll_y,
            scroll_width,
            (entries_height * entries_height / row_height / self.entries.len() as i32).max(30) as u32,
        ));

        canvas.set_clip_rect(None);

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
            Rect::new(name_x + 60, 49, 100, 22),
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
            Rect::new(treasure_x + 185, 49, 100, 22),
            true,
        ) {
            self.entries.sort_by(|a, b| b.treasure.cmp(&a.treasure));
        }

        if ui.button(
            canvas,
            text_painter,
            &LocalizableString::LeaderboardsSortByButton,
            Rect::new(rounds_x + 217, 49, 100, 22),
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

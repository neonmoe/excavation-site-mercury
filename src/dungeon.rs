// TODO: DungeonEvents (and DungeonSaves) should be versioned.

use crate::{stats, Fighter, GameLog, Level, Name, TileGraphic};
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

/// Messages that cause things to happen in the Dungeon. Saves consist
/// of a seed, a bunch of these, and some metadata.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum DungeonEvent {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    ProcessTurn,
}

#[derive(Clone, PartialEq, Debug)]
struct DungeonState {
    rng: Pcg32,
    log: GameLog,
    level: Level,
    fighters: Vec<Fighter>,
    round: u64,
}

impl DungeonState {
    pub fn new(seed: u64) -> DungeonState {
        let mut rng = Pcg32::seed_from_u64(seed);
        let log = GameLog::new();
        let level = Level::new(&mut rng);
        let mut fighters = Vec::new();
        let name = Name::UserInput(String::from("Astronaut"));
        fighters.push(Fighter::new(name, TileGraphic::Player, 4, 4, stats::PLAYER));
        let enemy_list = vec![
            (Name::Slime, TileGraphic::Slime, stats::SLIME),
            (Name::Roach, TileGraphic::Roach, stats::ROACH),
            (Name::Rockman, TileGraphic::Rockman, stats::ROCKMAN),
            (Name::SentientMetal, TileGraphic::SentientMetal, stats::SENTIENT_METAL),
        ];
        let mut x = 3;
        for (name, tile, stats) in enemy_list {
            fighters.push(Fighter::new(name, tile, x, 5, stats));
            x += 1;
        }
        DungeonState {
            rng,
            log,
            level,
            fighters,
            round: 1,
        }
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let mut player = Fighter::dummy();
        std::mem::swap(&mut player, &mut self.fighters[0]);
        player.step(
            dx,
            dy,
            &mut self.fighters,
            &mut self.level,
            &mut self.rng,
            &mut self.log,
            self.round,
        );
        std::mem::swap(&mut self.fighters[0], &mut player);
    }

    pub fn process_turn(&mut self) {
        self.round += 1;
    }
}

#[derive(Serialize, Deserialize)]
pub struct DungeonSave {
    game_version: String,
    seed: u64,
    events: Vec<DungeonEvent>,
}

/// The main game-logic runner and bookkeeper.
pub struct Dungeon {
    seed: u64,
    events: Vec<DungeonEvent>,
    state: DungeonState,
}

impl Dungeon {
    pub fn new(seed: u64) -> Dungeon {
        Dungeon {
            seed,
            events: Vec::new(),
            state: DungeonState::new(seed),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Dungeon, bincode::Error> {
        let save: DungeonSave = bincode::deserialize(bytes)?;
        let mut dungeon = Dungeon {
            seed: save.seed,
            events: Vec::new(),
            state: DungeonState::new(save.seed),
        };
        for event in &save.events {
            dungeon.run_event(*event);
        }
        Ok(dungeon)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(&DungeonSave {
            game_version: format!("\r\nexcavation-site-mercury version: {}\r\n", env!("CARGO_PKG_VERSION")),
            seed: self.seed,
            events: self.events.clone(),
        })
    }

    pub fn run_event(&mut self, event: DungeonEvent) {
        // First, run the event and save the results:
        let state_before_event = self.state.clone();
        self.apply_event_to_state(event);
        let state_after_event = self.state.clone();

        // Run the event again, ensure that the results are the same.
        self.state = state_before_event;
        self.apply_event_to_state(event);
        debug_assert_eq!(state_after_event, self.state);

        // Finally, register it to the event list.
        self.events.push(event);
    }

    fn apply_event_to_state(&mut self, event: DungeonEvent) {
        use DungeonEvent::*;
        match event {
            MoveUp => self.state.move_player(0, -1),
            MoveDown => self.state.move_player(0, 1),
            MoveLeft => self.state.move_player(-1, 0),
            MoveRight => self.state.move_player(1, 0),
            ProcessTurn => self.state.process_turn(),
        }
    }

    pub fn level(&self) -> &Level {
        &self.state.level
    }

    pub fn fighters(&self) -> &[Fighter] {
        &self.state.fighters
    }

    pub fn player(&self) -> &Fighter {
        &self.state.fighters[0]
    }

    pub fn log(&self) -> &GameLog {
        &self.state.log
    }

    pub fn round(&self) -> u64 {
        self.state.round
    }
}

// TODO: DungeonEvents (and DungeonSaves) should be versioned.

use crate::{enemy_ai, stats, EnemyAi, Fighter, GameLog, Level, Name, Stats, TileGraphic};
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
    ais: Vec<Option<EnemyAi>>,
    round: u64,
}

impl DungeonState {
    pub fn new(seed: u64) -> DungeonState {
        let mut rng = Pcg32::seed_from_u64(seed);
        let log = GameLog::new();
        let level = Level::new(&mut rng);
        let mut state = DungeonState {
            rng,
            log,
            level,
            fighters: Vec::new(),
            ais: Vec::new(),
            round: 1,
        };

        let name = Name::UserInput(String::from("Astronaut"));
        state.spawn_fighter(name, TileGraphic::Player, stats::PLAYER, None, 4, 4);

        let enemy_list = vec![
            (Name::Slime, TileGraphic::Slime, stats::SLIME, enemy_ai::SLIME),
            (Name::Roach, TileGraphic::Roach, stats::ROACH, enemy_ai::ROACH),
            (Name::Rockman, TileGraphic::Rockman, stats::ROCKMAN, enemy_ai::ROCKMAN),
            (
                Name::SentientMetal,
                TileGraphic::SentientMetal,
                stats::SENTIENT_METAL,
                enemy_ai::SENTIENT_METAL,
            ),
        ];
        let mut x = 3;
        for (name, tile, stats, ai) in enemy_list {
            state.spawn_fighter(name, tile, stats, Some(ai), x, x + 2);
            x += 1;
        }

        state
    }

    pub fn spawn_fighter(&mut self, name: Name, tile: TileGraphic, stats: Stats, ai: Option<EnemyAi>, x: i32, y: i32) {
        self.fighters
            .push(Fighter::new(self.fighters.len(), name, tile, x, y, stats));
        self.ais.push(ai);
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
        debug_assert_eq!(self.fighters.len(), self.ais.len());
        let mut current_fighter = Fighter::dummy();
        let mut current_ai = None;
        for i in 0..self.fighters.len() {
            // Swap out the fighter being processed for the dummy
            std::mem::swap(&mut current_fighter, &mut self.fighters[i]);
            std::mem::swap(&mut current_ai, &mut self.ais[i]);

            if let Some(ai) = current_ai.as_mut() {
                ai.process(
                    &mut current_fighter,
                    &mut self.fighters,
                    &mut self.level,
                    &mut self.rng,
                    &mut self.log,
                    self.round,
                );
            }

            // Swap the dummy back from the array
            std::mem::swap(&mut self.fighters[i], &mut current_fighter);
            std::mem::swap(&mut self.ais[i], &mut current_ai);
        }
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

    pub fn get_fighter(&self, id: usize) -> Option<&Fighter> {
        if id < self.state.fighters.len() {
            Some(&self.state.fighters[id])
        } else {
            None
        }
    }
}

// TODO: DungeonEvents (and DungeonSaves) should be versioned.

use crate::Level;
use rand_core::SeedableRng;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

/// Messages that cause things to happen in the Dungeon.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum DungeonEvent {}

#[derive(Clone, PartialEq, Debug)]
struct DungeonState {
    rng: Pcg32,
    level: Level,
}

impl DungeonState {
    pub fn new(seed: u64) -> DungeonState {
        let mut rng = Pcg32::seed_from_u64(seed);
        let level = Level::new(&mut rng);
        DungeonState { rng, level }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DungeonSave {
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

        self.events.push(event);
    }

    fn apply_event_to_state(&mut self, event: DungeonEvent) {
        match event {}
    }

    pub fn level(&self) -> &Level {
        &self.state.level
    }
}

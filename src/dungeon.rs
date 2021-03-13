// TODO: DungeonEvents (and DungeonSaves) should be versioned.

use crate::{EnemyAi, Fighter, FighterSpawn, GameLog, Level, StatIncrease, Terrain};
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
    LevelUp(StatIncrease),
}

#[derive(Clone, PartialEq, Debug)]
struct DungeonState {
    rng: Pcg32,
    log: GameLog,
    levels: Vec<Level>,
    current_level: usize,
    fighters: Vec<Fighter>,
    ais: Vec<Option<EnemyAi>>,
    round: u64,
    level_changed: bool,
    stat_increase_pending: bool,
}

impl DungeonState {
    pub fn new(seed: u64) -> DungeonState {
        let mut rng = Pcg32::seed_from_u64(seed);
        let log = GameLog::new();
        let mut levels = Vec::new();
        for difficulty in 0..4 {
            levels.push(Level::new(&mut rng, difficulty));
        }

        let mut state = DungeonState {
            rng,
            log,
            levels,
            current_level: 0,
            fighters: Vec::new(),
            ais: Vec::new(),
            round: 1,
            level_changed: false,
            stat_increase_pending: false,
        };

        for level in &state.levels {
            debug_assert!(!level.spawns.is_empty());
        }
        state.load_level();

        state
    }

    pub fn spawn_fighter(&mut self, spawn: FighterSpawn, levels_up: bool) {
        self.fighters.push(Fighter::new(
            self.fighters.len(),
            spawn.name,
            spawn.tile,
            spawn.x,
            spawn.y,
            spawn.stats,
            levels_up,
        ));
        self.ais.push(spawn.ai);
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        let mut player = Fighter::dummy();
        std::mem::swap(&mut player, &mut self.fighters[0]);
        player.step(
            dx,
            dy,
            &mut self.fighters,
            &mut self.levels[self.current_level],
            &mut self.rng,
            &mut self.log,
            self.round,
        );
        player.stats.treasure += self.levels[self.current_level].take_treasure(player.x, player.y);
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
                    &mut self.levels[self.current_level],
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
        self.level_changed = false;
    }

    pub fn load_level(&mut self) {
        let player = self.fighters.get(0).map(|f| f.clone());
        self.fighters.clear();
        self.ais.clear();
        self.level_changed = true;
        self.stat_increase_pending = self.current_level > 0;

        let mut spawns_iter = self.levels[self.current_level].spawns.clone().into_iter();

        if let Some(mut player) = player {
            let player_spawn = &self.levels[self.current_level].spawns[0];
            player.x = player_spawn.x;
            player.y = player_spawn.y;
            self.fighters.push(player);
            self.ais.push(None);
            let _ = spawns_iter.next();
        } else {
            self.spawn_fighter(spawns_iter.next().unwrap(), true);
        }

        for spawn in spawns_iter {
            self.spawn_fighter(spawn, false);
        }
    }

    pub fn increase_stat(&mut self, inc: StatIncrease) {
        if self.stat_increase_pending {
            self.fighters[0].stats.apply_increase(inc);
            self.stat_increase_pending = false;
        }
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
            dungeon.try_load_next_level(true);
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
            MoveUp => {
                self.state.move_player(0, -1);
                self.state.process_turn();
            }
            MoveDown => {
                self.state.move_player(0, 1);
                self.state.process_turn();
            }
            MoveLeft => {
                self.state.move_player(-1, 0);
                self.state.process_turn();
            }
            MoveRight => {
                self.state.move_player(1, 0);
                self.state.process_turn();
            }
            LevelUp(inc) => self.state.increase_stat(inc),
        }
    }

    pub fn can_run_events(&self) -> bool {
        let player = &self.state.fighters[0];
        self.state.levels[self.state.current_level].get_terrain(player.x, player.y) != Terrain::Exit
            && !self.is_game_over()
            && !self.stat_increase_pending()
    }

    pub fn stat_increase_pending(&self) -> bool {
        self.state.stat_increase_pending
    }

    pub fn is_game_over(&self) -> bool {
        self.state.fighters[0].stats.health <= 0
    }

    pub fn final_treasure_found(&self) -> bool {
        self.level().final_treasure_found
    }

    pub fn try_load_next_level(&mut self, skip_animation: bool) {
        let player = &self.state.fighters[0];
        let on_exit = self.state.levels[self.state.current_level].get_terrain(player.x, player.y) == Terrain::Exit;
        if on_exit && (!player.is_animating() || skip_animation) {
            self.state.current_level += 1;
            self.state.load_level();
        }
    }

    pub fn level_nth(&self) -> usize {
        self.state.current_level
    }

    pub fn level(&self) -> &Level {
        &self.state.levels[self.state.current_level]
    }

    pub fn level_mut(&mut self) -> &mut Level {
        &mut self.state.levels[self.state.current_level]
    }

    pub fn is_first_level(&self) -> bool {
        self.state.current_level == 0
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

    pub fn get_selectable_fighter_ids(&self) -> Vec<usize> {
        if let Some(rect) = self.level().room_at_position(self.player().position()) {
            let mut result = Vec::with_capacity(self.state.fighters.len());
            for fighter in &self.state.fighters {
                if rect.contains_point(fighter.position()) {
                    result.push(fighter.id);
                }
            }
            result
        } else {
            vec![0]
        }
    }

    pub fn level_changed(&self) -> bool {
        self.state.level_changed
    }
}

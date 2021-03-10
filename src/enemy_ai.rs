use crate::{Fighter, GameLog, Level, Terrain};
use rand_core::RngCore;
use rand_pcg::Pcg32;

pub const SLIME: EnemyAi = EnemyAi::new(Personality::SelfDefense { was_attacked: false });
pub const ROACH: EnemyAi = EnemyAi::new(Personality::Skitterer);
pub const ROCKMAN: EnemyAi = EnemyAi::new(Personality::Hunter { distance: 4.0 });
pub const SENTIENT_METAL: EnemyAi = EnemyAi::new(Personality::Tower { attack_interval: 3 });

#[derive(Clone, PartialEq, Debug)]
enum Personality {
    /// Does nothing.
    Passive,
    /// Stands still until attacked, and attacks back.
    SelfDefense { was_attacked: bool },
    /// Runs around randomly.
    Skitterer,
    /// Runs towards the player to attack once they're in range.
    Hunter { distance: f32 },
    /// Avoids the player, deals damage in a '+' shape periodically.
    Tower { attack_interval: i32 },
}

#[derive(Clone, PartialEq, Debug)]
pub struct EnemyAi {
    personality: Personality,
}

impl EnemyAi {
    const fn new(personality: Personality) -> EnemyAi {
        EnemyAi { personality }
    }

    pub fn dummy() -> EnemyAi {
        EnemyAi {
            personality: Personality::Passive,
        }
    }

    pub fn process(
        &mut self,
        fighter: &mut Fighter,
        fighters: &mut [Fighter],
        level: &mut Level,
        rng: &mut Pcg32,
        log: &mut GameLog,
        round: u64,
    ) {
        if fighter.stats.health <= 0 {
            return;
        }

        let mut random_walk = |fighter: &mut Fighter, fighters: &mut [Fighter], level: &mut Level| {
            let d = (rng.next_u32() % 4) as i32;
            let (dx, dy) = match d {
                0 => (1, 0),
                1 => (-1, 0),
                2 => (0, 1),
                3 => (0, -1),
                _ => unreachable!(),
            };
            let new_x = fighter.x + dx;
            let new_y = fighter.y + dy;
            let enemy_in_way = fighters.iter().skip(1).find(|f| f.x == new_x && f.y == new_y).is_some();
            let door_in_way = level.get_terrain(new_x, new_y) == Terrain::Door;
            if !enemy_in_way && !door_in_way {
                fighter.step(dx, dy, fighters, level, rng, log, round);
            }
        };

        match self.personality {
            Personality::Passive => {}
            Personality::SelfDefense { ref mut was_attacked } => {
                if let Some((dx, dy)) = fighter.previously_hit_from {
                    if *was_attacked {
                        fighter.step(dx, dy, fighters, level, rng, log, round);
                        *was_attacked = false;
                        fighter.previously_hit_from = None;
                    } else {
                        *was_attacked = true;
                    }
                }
            }
            Personality::Skitterer => random_walk(fighter, fighters, level),
            Personality::Hunter { distance } => {
                let player = &fighters[0];
                let (dx, dy) = (player.x - fighter.x, player.y - fighter.y);
                let pd = ((dx * dx + dy * dy) as f32).sqrt();
                if pd <= distance && round % 3 < 2 {
                    if dy != 0 {
                        fighter.step(0, dy.signum(), fighters, level, rng, log, round);
                    } else {
                        fighter.step(dx.signum(), 0, fighters, level, rng, log, round);
                    }
                } else if pd > distance && round % 2 == 0 {
                    random_walk(fighter, fighters, level);
                }
            }
            Personality::Tower { attack_interval } => {
                // TODO: Implement Tower personality
            }
        }
    }
}

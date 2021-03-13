use crate::{Fighter, GameLog, Level, Terrain};
use rand_core::RngCore;
use rand_pcg::Pcg32;

pub const SLIME: EnemyAi = EnemyAi::new(Personality::SelfDefense { was_attacked: false });
pub const ROACH: EnemyAi = EnemyAi::new(Personality::Skitterer);
pub const ROCKMAN: EnemyAi = EnemyAi::new(Personality::Hunter { distance: 4.0 });
pub const SENTIENT_METAL: EnemyAi = EnemyAi::new(Personality::Tower { attack_interval: 4 });

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
    Tower { attack_interval: u64 },
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

        let mut random_walk = |rng: &mut Pcg32, fighter: &mut Fighter, fighters: &mut [Fighter], level: &mut Level| {
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
            let enemy_in_way = fighters
                .iter()
                .skip(1)
                .filter(|f| f.stats.health > 0)
                .find(|f| f.x == new_x && f.y == new_y)
                .is_some();
            let avoided = level.get_terrain(new_x, new_y).enemies_avoid();
            let would_move_behind_wall = dy > 0 && level.get_terrain(new_x, new_y + 1) == Terrain::Wall;
            if !enemy_in_way && !avoided && !would_move_behind_wall {
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
                } else if round % (1 + rng.next_u32() as u64 % 20) == 0 {
                    random_walk(rng, fighter, fighters, level);
                }
            }
            Personality::Skitterer => random_walk(rng, fighter, fighters, level),
            Personality::Hunter { distance } => {
                let player = &fighters[0];
                let (dx, dy) = (player.x - fighter.x, player.y - fighter.y);
                let pd = ((dx * dx + dy * dy) as f32).sqrt();
                if pd <= distance && round % 4 < 2 {
                    if dy != 0 {
                        fighter.step(0, dy.signum(), fighters, level, rng, log, round);
                    } else {
                        fighter.step(dx.signum(), 0, fighters, level, rng, log, round);
                    }
                } else if pd > distance && round % 2 == 0 {
                    random_walk(rng, fighter, fighters, level);
                }
            }
            Personality::Tower { attack_interval } => {
                if round % attack_interval == 0 {
                    fighter.cast_laser_cross(rng, fighters, level, log, round);
                } else {
                    // Run away from the player, avoid getting cornered (somewhat)
                    let player = &fighters[0];
                    let (dx, dy) = (player.x - fighter.x, player.y - fighter.y);
                    if dx.abs() < dy.abs() {
                        if level.get_terrain(fighter.x - dx.signum(), fighter.y).unwalkable() {
                            if level.get_terrain(fighter.x, fighter.y - dy.signum()).unwalkable() {
                                fighter.step(0, dy.signum(), fighters, level, rng, log, round);
                            } else {
                                fighter.step(0, -dy.signum(), fighters, level, rng, log, round);
                            }
                        } else {
                            fighter.step(-dx.signum(), 0, fighters, level, rng, log, round);
                        }
                    } else {
                        if level.get_terrain(fighter.x, fighter.y - dy.signum()).unwalkable() {
                            if level.get_terrain(fighter.x - dx.signum(), fighter.y).unwalkable() {
                                fighter.step(dx.signum(), 0, fighters, level, rng, log, round);
                            } else {
                                fighter.step(-dx.signum(), 0, fighters, level, rng, log, round);
                            }
                        } else {
                            fighter.step(0, -dy.signum(), fighters, level, rng, log, round);
                        }
                    }
                }
            }
        }
    }
}

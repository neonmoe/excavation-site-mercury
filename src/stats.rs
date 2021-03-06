use serde::{Deserialize, Serialize};

pub const DUMMY: Stats = Stats {
    max_health: 1,
    health: 1,
    arm: 1,
    leg: 1,
    finger: 1,
    flying: false,
    treasure: 0,
};

pub const PLAYER: Stats = Stats {
    max_health: 5,
    health: 5,
    arm: 10,
    leg: 10,
    finger: 10,
    flying: false,
    treasure: 0,
};

pub const SLIME: Stats = Stats {
    max_health: 4,
    health: 4,
    arm: 12,
    leg: 8,
    finger: 1,
    flying: false,
    treasure: 0,
};

pub const ROACH: Stats = Stats {
    max_health: 3,
    health: 3,
    arm: 10,
    leg: 13,
    finger: 8,
    flying: false,
    treasure: 0,
};

pub const ROCKMAN: Stats = Stats {
    max_health: 7,
    health: 7,
    arm: 10,
    leg: 14,
    finger: 5,
    flying: false,
    treasure: 0,
};

pub const SENTIENT_METAL: Stats = Stats {
    max_health: 9,
    health: 9,
    arm: 16,
    leg: 15,
    finger: 1,
    flying: true,
    treasure: 6,
};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Debug)]
pub enum StatIncrease {
    Arm,
    Leg,
    Finger,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    /// Cap for [Stats::health].
    pub max_health: i32,
    /// The amount of normal hits this creature can take.
    pub health: i32,
    /// The coordination and skill of the creature's weapon-swinging
    /// appendages.
    pub arm: i32,
    /// The agility and intuition of the creature, for dodging hits.
    pub leg: i32,
    /// The nimbleness of the creature's lockpicking and
    /// pickpocketing.
    pub finger: i32,
    /// True for creatures floating in air, and those who have
    /// acquired a flying apparatus.
    pub flying: bool,
    /// The amount of treasure this fighter drops when dead.
    pub treasure: i32,
}

impl Stats {
    pub fn apply_increase(&mut self, inc: StatIncrease) {
        match inc {
            StatIncrease::Arm => self.arm += 2,
            StatIncrease::Leg => self.leg += 2,
            StatIncrease::Finger => self.finger += 2,
        }
    }
}

pub const DUMMY: Stats = Stats {
    max_health: 1,
    health: 1,
    arm: 1,
    leg: 1,
    finger: 1,
    brain: 1,
};

pub const PLAYER: Stats = Stats {
    max_health: 5,
    health: 5,
    arm: 10,
    leg: 10,
    finger: 10,
    brain: 10,
};

#[derive(Clone, Debug)]
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
    /// The smarts and mental power of the creature, for use in
    /// operating machines and seeing through illusions.
    pub brain: i32,
}

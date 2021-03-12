use crate::{interface, Font, StatIncrease, Text};
use sdl2::pixels::Color;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    English,
    Debug,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Name {
    UserInput(String),
    Astronaut,
    Dummy,
    Slime,
    Roach,
    Rockman,
    SentientMetal,
}

impl Name {
    pub fn translated_to(&self, language: Language) -> String {
        if language == Language::Debug {
            return format!("{:?}", self);
        }

        match self {
            Name::UserInput(s) => s.clone(),
            Name::Astronaut => match language {
                Language::Debug => unreachable!(),
                Language::English => String::from("Astronaut"),
            },
            Name::Dummy => match language {
                Language::Debug => unreachable!(),
                Language::English => String::from("Dummy"),
            },
            Name::Slime => match language {
                Language::Debug => unreachable!(),
                Language::English => String::from("Slime"),
            },
            Name::Roach => match language {
                Language::Debug => unreachable!(),
                Language::English => String::from("Roach"),
            },
            Name::Rockman => match language {
                Language::Debug => unreachable!(),
                Language::English => String::from("Rock Man"),
            },
            Name::SentientMetal => match language {
                Language::Debug => unreachable!(),
                Language::English => String::from("Superior Metal Being"),
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum LocalizableString {
    SomeoneAttackedSomeone {
        attacker: Name,
        defender: Name,
        damage: i32,
        roll: i32,
        attacker_arm: i32,
        defender_leg: i32,
    },

    AttackMissed {
        attacker: Name,
        defender: Name,
        roll: i32,
        attacker_arm: i32,
        defender_leg: i32,
    },

    SomeoneWasIncapacitated(Name),

    FighterDescription {
        id: usize,
        name: Name,
        max_health: i32,
        health: i32,
        arm: i32,
        leg: i32,
        finger: i32,
        brain: i32,
    },

    GameOver {
        name: Name,
    },
    Victory,

    RestartButton,
    SubmitToLeaderboardsButton,
    LevelUpMessage(u32),
    StatInfo(StatIncrease),
    IncreaseStatButton(StatIncrease),
}

impl LocalizableString {
    pub fn localize(&self, language: Language) -> Vec<Text> {
        if language == Language::Debug {
            return vec![Text(Font::RegularUi, 12.0, Color::WHITE, format!("{:#?}", self))];
        }

        const NORMAL_FONT_SIZE: f32 = 16.0;
        const SMALLER_FONT_SIZE: f32 = 14.0;
        const BIGGER_FONT_SIZE: f32 = 18.0;
        match self {
            LocalizableString::SomeoneAttackedSomeone {
                attacker,
                defender,
                damage,
                roll,
                attacker_arm,
                defender_leg,
            } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi,
                        NORMAL_FONT_SIZE,
                        Color::WHITE,
                        format!(
                            "{att} hit {def} for {dmg} damage!\n",
                            att = attacker.translated_to(language),
                            def = defender.translated_to(language),
                            dmg = damage,
                        ),
                    ),
                    Text(
                        Font::RegularUi,
                        SMALLER_FONT_SIZE,
                        Color::RGB(0x99, 0x99, 0x99),
                        format!(
                            "Rolled {roll} + Arm {arm} - Leg {leg} = {diff}, leading to {bonus} bonus damage.\n",
                            roll = roll,
                            arm = attacker_arm,
                            leg = defender_leg,
                            diff = roll + attacker_arm - defender_leg,
                            bonus = damage - 1,
                        ),
                    ),
                ],
            },

            LocalizableString::AttackMissed {
                attacker,
                defender,
                roll,
                attacker_arm,
                defender_leg,
            } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi,
                        NORMAL_FONT_SIZE,
                        Color::RGB(0xEE, 0xEE, 0xEE),
                        format!(
                            "{att} struck {def}, but missed.\n",
                            att = attacker.translated_to(language),
                            def = defender.translated_to(language),
                        ),
                    ),
                    Text(
                        Font::RegularUi,
                        SMALLER_FONT_SIZE,
                        Color::RGB(0x99, 0x99, 0x99),
                        format!(
                            "Rolled {roll}, hitting would require {modf}, because Leg {leg} surpasses Arm {arm} by {modf}.\n",
                            roll = roll,
                            arm = attacker_arm,
                            leg = defender_leg,
                            modf = defender_leg - attacker_arm,
                        ),
                    ),
                ],
            },

            LocalizableString::SomeoneWasIncapacitated(name) => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![Text(
                    Font::RegularUi,
                    NORMAL_FONT_SIZE,
                    Color::WHITE,
                    format!("{} is incapacitated.\n", name.translated_to(language)),
                )],
            },

            LocalizableString::FighterDescription {
                id,
                name,
                max_health,
                health,
                arm,
                leg,
                finger,
                brain,
            } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi,
                        BIGGER_FONT_SIZE,
                        Color::WHITE,
                        format!(
                            "{}{}{}\n",
                            name.translated_to(language),
                            if *id <= 0 { " (that's you)" } else { "" },
                            if *health <= 0 { " (dead)" } else { "" },
                        ),
                    ),
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, format!("Health: ")),
                    Text(
                        Font::RegularUi,
                        20.0,
                        if *health <= *max_health / 3 {
                            interface::HEALTH_LOW
                        } else if *health <= *max_health * 2 / 3 {
                            interface::HEALTH_MEDIUM
                        } else {
                            interface::HEALTH_HIGH
                        },
                        format!("{}", health),
                    ),
                    Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE, format!("/{}", max_health)),
                    Text(
                        Font::RegularUi,
                        NORMAL_FONT_SIZE,
                        Color::WHITE,
                        format!(
                            "\nArm: {}\nLeg: {}\nFinger: {}\nBrain: {}\n",
                            arm, leg, finger, brain
                        ),
                    ),
                ],
            },

            LocalizableString::GameOver { name } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi,
                        BIGGER_FONT_SIZE,
                        Color::WHITE,
                        format!("{} was incapacitated.\n", name.translated_to(language)),
                    ),
                    Text(
                        Font::RegularUi,
                        NORMAL_FONT_SIZE,
                        Color::WHITE,
                        format!("\nBetter luck next time!\n"),
                    ),
                ],
            },

            LocalizableString::Victory => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi, BIGGER_FONT_SIZE, Color::WHITE, String::from("Treasure found!\n"),
                    ),
                    Text(
                        Font::RegularUi,
                        NORMAL_FONT_SIZE,
                        Color::WHITE,
                        format!("\nYou have delved a deep as it gets, congratulations!\n\
                                 Finish the run by selecting either button below.\n"),
                    ),
                ],
            },

            LocalizableString::RestartButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, String::from("Start over"))
                ],
            },
            LocalizableString::SubmitToLeaderboardsButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE, String::from("Submit to the leaderboards"))
                ],
            },

            LocalizableString::LevelUpMessage(current_level) => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, BIGGER_FONT_SIZE, Color::WHITE, String::from("Experience gained.\n\n")),
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, match current_level {
                        0 => String::from("Living blobs of coolant everywhere, giant roaches crawling in every corner, \
                                           what's next? The learning oppoturnities are endless, if nothing else.\n"),
                        1 => String::from("The darkness is starting to get to you, as you delve further away from \
                                           Sol's light. Treasure awaits.\n"),
                        2 => String::from("As you climb down the rope, the temperature gauge in your spacesuit \
                                           starts to climb, uncomfortably fast. Fortunate that the suit is \
                                           designed for extreme situations, it seems the motherload is near \
                                           the core of the asteroid. The depths await.\n"),
                        _ => String::from("[static noise]\n"),
                    })
                ],
            },

            LocalizableString::StatInfo(stat) => match language {
                Language::Debug => unreachable!(),
                Language::English =>match stat {
                    StatIncrease::Arm => vec![
                        Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                             String::from("Arm\n")),
                        Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE,
                             String::from("\nReflects your ability to smash heads in. \
                                           Each +1 is equivalent to rolling 1 better."))
                    ],
                    StatIncrease::Leg => vec![
                        Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                             String::from("Leg\n")),
                        Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE,
                             String::from("\nMakes you harder to hit. Each +1 is equivalent \
                                           to enemies rolling 1 worse."))
                    ],
                    StatIncrease::Finger => vec![
                        Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                             String::from("Finger\n")),
                        Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE,
                             String::from("\nAllows you to pick locks, to loot leftover \
                                           treasure from previous, security conscious miners."))
                    ],
                    StatIncrease::Brain => vec![
                        Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                             String::from("Brain\n")),
                        Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE,
                             String::from("\nMakes you smart. For all those logic puzzles \
                                           the ancients left for you to solve."))
                    ],
                }
            },

            LocalizableString::IncreaseStatButton(stat) => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, match stat {
                        StatIncrease::Arm => String::from("+2 to Arm"),
                        StatIncrease::Leg => String::from("+2 to Leg"),
                        StatIncrease::Finger => String::from("+2 to Finger"),
                        StatIncrease::Brain => String::from("+2 to Brain"),
                    })
                ],
            },
        }
    }
}

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
    Character(char, f32, Color),

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

    DoorUnlocked {
        roll_threshold: i32,
        roll: i32,
        finger: i32,
    },
    DoorUnlockingFailed {
        roll_threshold: i32,
        roll: i32,
        finger: i32,
    },

    FighterDescription {
        id: usize,
        name: Name,
        max_health: i32,
        health: i32,
        arm: i32,
        leg: i32,
        finger: i32,
    },

    GameOver {
        name: Name,
    },
    Victory,

    BigConfirmButton,
    EraseButton,
    NameInputInfo,
    RestartButton,
    QuitButton,
    SubmitToLeaderboardsButton,
    LevelUpMessage(u32),
    StatInfo(StatIncrease),
    IncreaseStatButton(StatIncrease),

    StatIncreaseByTraining {
        stat: StatIncrease,
        name: Name,
    },

    LeaderboardsHeader,
    LeaderboardsEmpty,
    LeaderboardsTitleName,
    LeaderboardsTitleTreasure,
    LeaderboardsTitleRounds,
    LeaderboardsName([char; 3]),
    LeaderboardsTreasure(i32),
    LeaderboardsRounds(Option<u64>),
    LeaderboardsSortByButton,
}

impl LocalizableString {
    pub fn localize(&self, language: Language) -> Vec<Text> {
        if let LocalizableString::Character(c, size, color) = self {
            return vec![Text(Font::RegularUi, *size, *color, format!("{}", c))];
        } else if language == Language::Debug {
            return vec![Text(Font::RegularUi, 12.0, Color::WHITE, format!("{:#?}", self))];
        }

        const NORMAL_FONT_SIZE: f32 = 16.0;
        const SMALLER_FONT_SIZE: f32 = 14.0;
        const BIGGER_FONT_SIZE: f32 = 18.0;
        const COMMENT_COLOR: Color = Color::RGB(0x99, 0x99, 0x99);
        match self {
            LocalizableString::Character(_, _, _) => unreachable!(),

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
                        Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                        format!(
                            "{att} hit {def} for {dmg} damage!\n",
                            att = attacker.translated_to(language),
                            def = defender.translated_to(language),
                            dmg = damage,
                        ),
                    ),
                    Text(
                        Font::RegularUi, SMALLER_FONT_SIZE, COMMENT_COLOR,
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
                        Font::RegularUi, NORMAL_FONT_SIZE, Color::RGB(0xEE, 0xEE, 0xEE),
                        format!(
                            "{att} struck {def}, but missed.\n",
                            att = attacker.translated_to(language),
                            def = defender.translated_to(language),
                        ),
                    ),
                    Text(
                        Font::RegularUi, SMALLER_FONT_SIZE, COMMENT_COLOR,
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
                    Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                    format!("{} is incapacitated.\n", name.translated_to(language)),
                )],
            },

            LocalizableString::DoorUnlocked {
                roll_threshold,
                roll,
                finger,
            } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                        format!("Door unlocked with a roll of {}.\n", roll),
                    ),
                    Text(
                        Font::RegularUi, SMALLER_FONT_SIZE, COMMENT_COLOR,
                        format!(
                            "The threshold for unlocking was {}, from Lock {} - Finger {}.\n",
                            roll_threshold - finger,
                            roll_threshold,
                            finger,
                        ),
                    ),
                ],
            },

            LocalizableString::DoorUnlockingFailed {
                roll_threshold,
                roll,
                finger,
            } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                        format!("Failed to open door with a roll of {}.\n", roll),
                    ),
                    Text(
                        Font::RegularUi, NORMAL_FONT_SIZE, COMMENT_COLOR,
                        format!(
                            "Unlocking{} would require a roll of {} (Lock {} - Finger {}).\n",
                            if roll_threshold - finger > 6 { " is impossible with current Finger, as it" } else { "" },
                            roll_threshold - finger,
                            roll_threshold,
                            finger,
                        ),
                    ),
                ],
            },

            LocalizableString::FighterDescription {
                id,
                name,
                max_health,
                health,
                arm,
                leg,
                finger,
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
                            "\nArm: {}\nLeg: {}\nFinger: {}\n",
                            arm, leg, finger
                        ),
                    ),
                ],
            },

            LocalizableString::GameOver { name } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(
                        Font::RegularUi, BIGGER_FONT_SIZE, Color::WHITE,
                        format!("{} was incapacitated.\n", name.translated_to(language)),
                    ),
                    Text(
                        Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
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
                        Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                        format!("\nYou have delved a deep as it gets, congratulations!\n\
                                 Finish the run by selecting either button below.\n"),
                    ),
                ],
            },

            LocalizableString::BigConfirmButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::BoldUi, BIGGER_FONT_SIZE, Color::WHITE, String::from("Confirm"))
                ],
            },
            LocalizableString::EraseButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, String::from("Erase"))
                ],
            },
            LocalizableString::NameInputInfo => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, BIGGER_FONT_SIZE, Color::WHITE, String::from(
                        "Enter a name or tag to represent you on the leaderboards. \
                         Only ASCII characters (A-Z) and digits (0-9) are accepted, sorry about that.\n"
                    ))
                ],
            },
            LocalizableString::RestartButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, String::from("New run"))
                ],
            },
            LocalizableString::QuitButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE, String::from("Quit"))
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
                                           Each +1 is equivalent to rolling 1 better.\n"))
                    ],
                    StatIncrease::Leg => vec![
                        Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                             String::from("Leg\n")),
                        Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE,
                             String::from("\nMakes you harder to hit. Each +1 is equivalent \
                                           to enemies rolling 1 worse.\n"))
                    ],
                    StatIncrease::Finger => vec![
                        Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::WHITE,
                             String::from("Finger\n")),
                        Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE,
                             String::from("\nAllows you to open locked doors. \
                                           Each +1 is equivalent to rolling 1 better when \
                                           opening locked doors.\n"))
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
                    })
                ],
            },

            LocalizableString::StatIncreaseByTraining { stat, name } => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, NORMAL_FONT_SIZE, Color::RGB(0x44, 0xDD, 0x44), match stat {
                        StatIncrease::Arm => format!(
                            "{}'s Arm improved by +1 from training.",
                            name.translated_to(language),
                        ),
                        StatIncrease::Leg => format!(
                            "{}'s Leg improved by +1. Regular walks are good for your health!",
                            name.translated_to(language),
                        ),
                        StatIncrease::Finger => format!(
                            "{}'s Finger improved by +1. Each lock makes the next one a little easier.",
                            name.translated_to(language),
                        ),
                    })
                ],
            },

            LocalizableString::LeaderboardsHeader => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::BoldUi, 24.0, Color::WHITE, String::from("Leaderboards"))
                ],
            },

            LocalizableString::LeaderboardsEmpty => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, 16.0, Color::WHITE, String::from("The leaderboards are empty.\nThe server is probably down."))
                ],
            },

            LocalizableString::LeaderboardsTitleName => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::BoldUi, 18.0, Color::WHITE, String::from("Name"))
                ],
            },
            LocalizableString::LeaderboardsTitleTreasure => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::BoldUi, 18.0, Color::WHITE, String::from("Treasure collected"))
                ],
            },
            LocalizableString::LeaderboardsTitleRounds => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::BoldUi, 18.0, Color::WHITE, String::from("Finish time (in-world)"))
                ],
            },

            LocalizableString::LeaderboardsName(chars) => match language {
                _ => vec![Text(Font::RegularUi, 18.0, Color::WHITE, format!("{}{}{}", chars[0], chars[1], chars[2]))],
            },
            LocalizableString::LeaderboardsTreasure(amount) => match language {
                _ => vec![Text(Font::RegularUi, 18.0, Color::WHITE, format!("{}", amount))],
            },
            LocalizableString::LeaderboardsRounds(rounds) => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    if let Some(rounds) = rounds {
                        Text(Font::RegularUi, 18.0, Color::WHITE, format!(
                            "{:02}:{:02}:{:02}", rounds / 60 / 60, rounds / 60, rounds
                        ))
                    } else {
                        Text(Font::RegularUi, 18.0, Color::WHITE, String::from("Died."))
                    }
                ],
            },

            LocalizableString::LeaderboardsSortByButton => match language {
                Language::Debug => unreachable!(),
                Language::English => vec![
                    Text(Font::RegularUi, SMALLER_FONT_SIZE, Color::WHITE, String::from("Sort by"))
                ],
            },
        }
    }
}

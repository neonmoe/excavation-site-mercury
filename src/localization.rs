use crate::{Font, Text};
use sdl2::pixels::Color;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Language {
    English,
    Debug,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Name {
    UserInput(String),
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
                            Color::RGB(0xEE, 0x55, 0x44)
                        } else if *health <= *max_health * 2 / 3 {
                            Color::RGB(0xEE, 0xAA, 0x22)
                        } else {
                            Color::RGB(0x66, 0xCC, 0x33)
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
        }
    }
}

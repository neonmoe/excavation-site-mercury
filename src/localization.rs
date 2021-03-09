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
}

impl LocalizableString {
    pub fn localize(&self, language: Language) -> Vec<Text> {
        if language == Language::Debug {
            return vec![Text(Font::RegularUi, 12.0, Color::WHITE, format!("{:#?}", self))];
        }

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
                Language::English => vec![Text(
                    Font::RegularUi,
                    16.0,
                    Color::WHITE,
                    format!(
                        "{att} hit {def} for {dmg} damage! Roll: {roll} (1d6 modified by {arm} arm - {leg} leg = {modf})\n",
                        att = attacker.translated_to(language),
                        def = defender.translated_to(language),
                        dmg = damage,
                        roll = roll,
                        arm = attacker_arm,
                        leg = defender_leg,
                        modf = attacker_arm - defender_leg,
                    ),
                )],
            },
        }
    }
}

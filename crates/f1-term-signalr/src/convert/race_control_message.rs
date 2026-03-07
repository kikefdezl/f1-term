use chrono::{DateTime, NaiveDateTime, Utc};
use f1_term_core::{
    flag::{Flag, FlagColor, FlagScope},
    race_control_message::{MessageCategory, RaceControlMessage},
};

use crate::parsing::race_control_messages::RawRaceControlMessage;

impl TryFrom<&RawRaceControlMessage> for RaceControlMessage {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &RawRaceControlMessage) -> Result<Self, Self::Error> {
        let timestamp = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::parse_from_str(&value.Utc, "%Y-%m-%dT%H:%M:%S")?,
            Utc,
        );

        let category = match value.Category.as_str() {
            "Flag" => {
                let color_str = value
                    .Flag
                    .as_ref()
                    .ok_or("Missing 'Flag' in Flag category")?;
                let color = match color_str.as_str() {
                    "GREEN" => FlagColor::Green,
                    "YELLOW" => FlagColor::Yellow,
                    "DOUBLE YELLOW" => FlagColor::DoubleYellow,
                    "RED" => FlagColor::Red,
                    "CLEAR" => FlagColor::Clear,
                    "CHEQUERED" => FlagColor::Chequered,
                    _ => return Err(format!("Unknown flag color: {}", color_str).into()),
                };

                let scope_str = value
                    .Scope
                    .as_ref()
                    .ok_or("Missing 'Scope' in Flag category")?;
                let scope = match scope_str.as_str() {
                    "Track" => FlagScope::Track,
                    "Sector" => {
                        let sector_num = value.Sector.ok_or("Missing 'Sector' for Sector scope")?;
                        FlagScope::Sector(sector_num)
                    }
                    _ => return Err(format!("Unknown flag scope: {}", scope_str).into()),
                };

                MessageCategory::Flag(Flag { color, scope })
            }
            "SafetyCar" => MessageCategory::SafetyCar,
            "Other" => {
                // Some messages containing flag information don't come categorized as "Flag".
                // We parse them here if they contain flag specific keywords to properly colorize the UI.
                if value.Message.contains("RED FLAG") {
                    MessageCategory::Flag(Flag {
                        color: FlagColor::Red,
                        scope: FlagScope::Track,
                    })
                } else {
                    MessageCategory::Other
                }
            }
            _ => return Err(format!("Unknown Category: {}", value.Category).into()),
        };

        Ok(RaceControlMessage {
            timestamp,
            category,
            message: value.Message.clone(),
        })
    }
}

pub fn convert_race_control_messages(
    raw_messages: &[RawRaceControlMessage],
) -> Result<Vec<RaceControlMessage>, Box<dyn std::error::Error>> {
    raw_messages
        .iter()
        .map(RaceControlMessage::try_from)
        .collect()
}

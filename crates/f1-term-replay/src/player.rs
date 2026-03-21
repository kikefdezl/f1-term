use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};
use std::time::Duration;

use f1_term_signalr::merge_patch::merge_patch;
use log::warn;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TimelineMessage {
    pub topic: String,
    pub delta: Value,
}

pub struct Player {
    pub current_time: Duration,
    pub duration: Duration,
    pub canonical_state: Value,
    pub timeline: BTreeMap<Duration, Vec<TimelineMessage>>,
    pub base_state: Value,
    pub is_playing: bool,
    pub seek_counter: u64,
}

impl Player {
    pub fn new(base_state: Value) -> Self {
        Self {
            current_time: Duration::ZERO,
            duration: Duration::ZERO,
            canonical_state: base_state.clone(),
            timeline: BTreeMap::new(),
            base_state,
            is_playing: true,
            seek_counter: 0,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_playing = !self.is_playing;
    }

    pub fn parse_stream(&mut self, topic: &str, stream_data: &str) {
        for line in stream_data.lines() {
            if line.len() < 12 {
                continue;
            }

            let time_str = &line[0..12]; // "00:00:03.031"
            let json_str = &line[12..];

            if let Some(duration) = parse_time(time_str) {
                if duration > self.duration {
                    self.duration = duration;
                }

                if let Ok(delta) = serde_json::from_str::<Value>(json_str) {
                    let msg = TimelineMessage {
                        topic: topic.to_string(),
                        delta,
                    };
                    self.timeline.entry(duration).or_default().push(msg);
                } else {
                    warn!("Failed to parse JSON for {}: {}", topic, json_str);
                }
            } else {
                warn!("Failed to parse time for {}: {}", topic, time_str);
            }
        }
    }

    pub fn tick(&mut self, dt: Duration) -> Vec<TimelineMessage> {
        if !self.is_playing {
            return Vec::new();
        }

        let prev_time = self.current_time;
        self.current_time += dt;
        if self.current_time > self.duration {
            self.current_time = self.duration;
            self.is_playing = false; // Auto-pause at end
        }

        self.apply_deltas(prev_time, self.current_time)
    }

    pub fn seek_by(&mut self, offset: i64) {
        let current_secs = self.current_time.as_secs_f64();
        let new_secs = current_secs + offset as f64;
        let new_time = if new_secs < 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(new_secs)
        };
        self.seek(new_time);
    }

    pub fn seek(&mut self, time: Duration) {
        self.current_time = time;
        if self.current_time > self.duration {
            self.current_time = self.duration;
        }

        self.canonical_state = self.base_state.clone();

        let _ = self.apply_deltas(Duration::ZERO, self.current_time);
        self.seek_counter += 1;
    }

    fn apply_deltas(&mut self, from_time: Duration, to_time: Duration) -> Vec<TimelineMessage> {
        let mut dispatched = Vec::new();

        let start_bound = if from_time == Duration::ZERO {
            Included(from_time)
        } else {
            Excluded(from_time)
        };

        let range = (start_bound, Included(to_time));

        for (_time, msgs) in self.timeline.range(range) {
            for msg in msgs {
                if !self.canonical_state.is_object() {
                    self.canonical_state = serde_json::json!({});
                }

                let canonical_obj = self.canonical_state.as_object_mut().unwrap();
                let topic_entry = canonical_obj
                    .entry(msg.topic.clone())
                    .or_insert_with(|| serde_json::json!({}));

                merge_patch(topic_entry, &msg.delta);
                dispatched.push(msg.clone());
            }
        }

        dispatched
    }
}

pub fn parse_time(s: &str) -> Option<Duration> {
    if s.len() < 12 {
        return None;
    }

    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return None;
    }

    let h: u64 = parts[0].parse().ok()?;
    let m: u64 = parts[1].parse().ok()?;

    let s_ms: Vec<&str> = parts[2].split('.').collect();
    if s_ms.len() != 2 {
        return None;
    }

    let s: u64 = s_ms[0].parse().ok()?;
    let ms: u32 = s_ms[1].parse().ok()?;

    Some(Duration::from_secs(h * 3600 + m * 60 + s) + Duration::from_millis(ms as u64))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_time() {
        assert_eq!(
            parse_time("00:00:13.093"),
            Some(Duration::from_millis(13093))
        );
        assert_eq!(
            parse_time("01:05:03.001"),
            Some(Duration::from_millis(3600000 + 300000 + 3001))
        );
    }

    #[test]
    fn test_player_parsing() {
        let base_state = json!({});
        let mut player = Player::new(base_state);

        let stream_data = "00:00:01.000{\"Value\": 1}\n00:00:02.000{\"Value\": 2}\n";
        player.parse_stream("TestTopic", stream_data);

        assert_eq!(player.duration, Duration::from_secs(2));
        assert_eq!(player.timeline.len(), 2);

        let dt = Duration::from_secs(1);
        let msgs = player.tick(dt);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].topic, "TestTopic");
        assert_eq!(msgs[0].delta, json!({"Value": 1}));

        let msgs2 = player.tick(dt);
        assert_eq!(msgs2.len(), 1);
        assert_eq!(msgs2[0].delta, json!({"Value": 2}));
    }

    #[test]
    fn test_player_seek() {
        let base_state = json!({"TestTopic": {"Initial": true}});
        let mut player = Player::new(base_state);

        let stream_data = "00:00:01.000{\"Value\": 1}\n00:00:02.000{\"Value\": 2}\n";
        player.parse_stream("TestTopic", stream_data);

        player.seek(Duration::from_secs(2));

        // Check if canonical state merged correctly
        assert_eq!(
            player.canonical_state,
            json!({"TestTopic": {"Initial": true, "Value": 2}})
        );

        // If we tick from here, we shouldn't get old messages
        let msgs = player.tick(Duration::from_secs(1));
        assert_eq!(msgs.len(), 0);
    }
}

use ratatui::style::Color;

// sectors
pub const SEGMENTS: &str = "▮"; // other options: ▮ ▰  ● ⬤
pub const SEGMENT_WIDTH: u16 = 1; // The render width of the character above
pub const COLOR_OVERALL_FASTEST: Color = Color::from_u32(0xB11DFB); // #B11DFB
pub const COLOR_PERSONAL_FASTEST: Color = Color::from_u32(0x33D176); // #33D176
pub const COLOR_SLOWER: Color = Color::Yellow;
pub const COLOR_IN_PIT: Color = Color::Blue;
pub const COLOR_ABORTED: Color = Color::Red;

// tyre compounds
pub const COLOR_SOFT: Color = Color::Red;
pub const COLOR_MEDIUM: Color = Color::Yellow;
pub const COLOR_HARD: Color = Color::White;
pub const COLOR_WET: Color = Color::Blue;
pub const COLOR_INTERMEDIATE: Color = Color::Green;
pub const COLOR_UNKNOWN: Color = Color::DarkGray;

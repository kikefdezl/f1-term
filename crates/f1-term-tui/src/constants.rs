use ratatui::style::Color;

pub const SEGMENTS: &str = "▰ "; // other options: ▮ ▰  ● ⬤
pub const SEGMENT_WIDTH: u16 = 2; // The render with of the character above
//
pub const COLOR_OVERALL_FASTEST: Color = Color::from_u32(0xB11DFB); // #B11DFB
pub const COLOR_PERSONAL_FASTEST: Color = Color::from_u32(0x33D176); // #33D176
pub const COLOR_SLOWER: Color = Color::Yellow;
pub const COLOR_IN_PIT: Color = Color::Blue;
pub const COLOR_ABORTED: Color = Color::Red;

pub mod live_timing;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ActivePage {
    #[default]
    LiveTiming,
}

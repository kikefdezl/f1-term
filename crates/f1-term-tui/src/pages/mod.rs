pub mod dashboard;
pub mod stints;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ActivePage {
    #[default]
    LiveTiming,
    Stints,
}

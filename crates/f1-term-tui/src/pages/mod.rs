pub mod dashboard;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ActivePage {
    #[default]
    LiveTiming,
}

pub mod live_timing;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PageType {
    #[default]
    LiveTiming,
}

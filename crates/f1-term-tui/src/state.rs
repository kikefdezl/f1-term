#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GapMode {
    #[default]
    ToFastest,
    ToPositionAhead,
}

impl GapMode {
    pub fn toggle(&mut self) {
        *self = match self {
            GapMode::ToFastest => GapMode::ToPositionAhead,
            GapMode::ToPositionAhead => GapMode::ToFastest,
        };
    }
}

#[derive(Default)]
pub struct AppState {
    pub exit: bool,
    pub gap_mode: GapMode,
}

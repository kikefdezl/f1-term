#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub color: FlagColor,
    pub scope: FlagScope,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagColor {
    Green,
    Yellow,
    DoubleYellow,
    Red,
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagScope {
    Track,
    Sector(u8),
}

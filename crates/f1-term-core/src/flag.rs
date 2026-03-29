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
    Chequered,
    Blue,
    BlackAndWhite,
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlagScope {
    Track,
    Sector(u8),
    Driver,
}

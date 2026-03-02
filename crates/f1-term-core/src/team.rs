#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TeamName {
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct TeamColor {
    pub u32: u32, // 0x00RRGGBB
}

#[derive(Clone, Debug)]
pub struct Team {
    pub name: TeamName,
    pub color: TeamColor,
}

use std::fmt::Display;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TeamName {
    pub value: String,
}

#[derive(Debug)]
pub struct TeamColor {
    pub u32: u32, // 0x00RRGGBB
}

#[derive(Debug)]
pub struct Team {
    pub name: TeamName,
    pub color: TeamColor,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: #{:X}", self.name.value, self.color.u32)
    }
}

#[derive(Debug)]
pub struct Args {
    pub replay: bool,
}

impl Args {
    pub fn parse() -> Self {
        let mut replay = false;
        for arg in std::env::args().skip(1) {
            if arg == "--replay" {
                replay = true;
            }
        }
        Self { replay }
    }
}

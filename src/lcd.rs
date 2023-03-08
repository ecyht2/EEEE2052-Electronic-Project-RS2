#[derive(Debug, PartialEq)]
pub enum LCDButtons {
    RIGHT,
    UP,
    DOWN,
    LEFT,
    SELECT,
}

impl LCDButtons {
    pub fn new(reading: u16) -> Result<Self, ()> {
        match reading {
            0..=500 => Ok(Self::RIGHT),
            501..=1000 => Ok(Self::UP),
            1001..=2500 => Ok(Self::DOWN),
            2501..=4000 => Ok(Self::LEFT),
            4001..=4096 => Ok(Self::SELECT),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
/// Buttons on the Arduino Uno LCD keypad shield.
pub enum LCDButtons {
    /// Right button.
    RIGHT,
    /// Up button.
    UP,
    /// Down button.
    DOWN,
    /// Left button.
    LEFT,
    /// Select button.
    SELECT,
}

impl LCDButtons {
    /// Gets the button being pressed from the value from the ADC.
    ///
    /// An error would be returned if the value given is > 4096. (Invalid comparator reading).
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

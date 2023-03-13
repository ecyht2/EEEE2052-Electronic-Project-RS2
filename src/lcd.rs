//! Liquid Crystal Display (LCD).
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
    /// An None would be returned if the value given is > 4096. (Invalid comparator reading).
    pub fn new(reading: u16) -> Option<Self> {
        match reading {
            0..=500 => Some(Self::RIGHT),
            501..=1000 => Some(Self::UP),
            1001..=2500 => Some(Self::DOWN),
            2501..=4000 => Some(Self::LEFT),
            4001..=4096 => Some(Self::SELECT),
            _ => None,
        }
    }
}

use std::fmt;

#[derive(Default)]
pub struct Keypad {
    pub keys: [bool; 16],
}

impl fmt::Debug for Keypad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..15 {
            try!(write!(f, "Key {:x} is {} | ", i, self.keys[i]));
        }

        writeln!(f, "Key {:x} is {}", 15, self.keys[15])
    }
}

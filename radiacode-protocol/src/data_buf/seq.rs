#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seq(u8);

impl Seq {
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn raw(self) -> u8 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn is_newer_than(self, other: Seq) -> bool {
        self != other && self.0.wrapping_sub(other.0) < 128
    }

    pub fn lost_since(self, expected: Seq) -> Option<u32> {
        if self == expected {
            return Some(0);
        }
        if !self.is_newer_than(expected) {
            return None;
        }
        Some(u32::from(self.0.wrapping_sub(expected.0)))
    }
}

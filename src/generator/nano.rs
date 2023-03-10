use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanosecond(pub i64);

impl Nanosecond {
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis * 1_000_000)
    }

    pub const fn into_millis(self) -> i64 {
        self.0.div_euclid(1_000_000)
    }
}

impl Sub<Self> for Nanosecond {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add<Self> for Nanosecond {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl From<Nanosecond> for Millisecond {
    fn from(val: Nanosecond) -> Self {
        Millisecond(val.into_millis())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Millisecond(pub i64);

impl Sub<Self> for Millisecond {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Add<Self> for Millisecond {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl From<Millisecond> for Nanosecond {
    fn from(val: Millisecond) -> Self {
        Nanosecond(val.0 * 1_000_000)
    }
}

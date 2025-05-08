//! A 4-bit value

use core::fmt;
use core::mem::transmute;

/// A 4-bit value
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Nibble {
    #[default]
    V0 = 0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
}

impl Nibble {
    pub const MIN: Self = Self::V0;

    pub const MAX: Self = Self::V15;

    #[inline]
    pub const fn new(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::V0),
            1 => Some(Self::V1),
            2 => Some(Self::V2),
            3 => Some(Self::V3),
            4 => Some(Self::V4),
            5 => Some(Self::V5),
            6 => Some(Self::V6),
            7 => Some(Self::V7),
            8 => Some(Self::V8),
            9 => Some(Self::V9),
            10 => Some(Self::V10),
            11 => Some(Self::V11),
            12 => Some(Self::V12),
            13 => Some(Self::V13),
            14 => Some(Self::V14),
            15 => Some(Self::V15),
            _ => None,
        }
    }

    #[inline]
    pub const unsafe fn new_unchecked(value: u8) -> Self {
        unsafe { transmute(value) }
    }

    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    #[inline]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        if (self as u8) < min as u8 {
            min
        } else if self as u8 > max as u8 {
            max
        } else {
            self
        }
    }

    // #[inline]
    // pub const fn checked_add(self, rhs: Self) -> Option<Self> {
    //     match (self as u8).checked_add(rhs as u8) {
    //         Some(v) => Self::new(v),
    //         None => None,
    //     }
    // }

    // #[inline]
    // pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
    //     match (self as u8).checked_sub(rhs as u8) {
    //         Some(v) => Self::new(v),
    //         None => None,
    //     }
    // }

    // #[inline]
    // pub const fn wrapping_add(self, rhs: Self) -> Self {
    //     unsafe { Self::new_unchecked((self as u8).wrapping_add(rhs as u8) & 0xf) }
    // }

    // #[inline]
    // pub const fn wrapping_sub(self, rhs: Self) -> Self {
    //     unsafe { Self::new_unchecked((self as u8).wrapping_sub(rhs as u8) & 0xf) }
    // }

    // #[inline]
    // pub const fn saturating_add(self, rhs: Self) -> Self {
    //     let lhs = self.clamp(Self::MIN, Self::MAX) as u8;
    //     let rhs = rhs.clamp(Self::MIN, Self::MAX) as u8;
    //     match lhs + rhs {
    //         result @ 0..=15 => unsafe { Self::new_unchecked(result) },
    //         _ => Self::MAX,
    //     }
    // }

    // #[inline]
    // pub const fn saturating_sub(self, rhs: Self) -> Self {
    //     let lhs = self.clamp(Self::MIN, Self::MAX) as u8;
    //     let rhs = rhs.clamp(Self::MIN, Self::MAX) as u8;
    //     unsafe { Self::new_unchecked(lhs.saturating_sub(rhs)) }
    // }
}

impl fmt::Display for Nibble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u8())
    }
}

impl fmt::Debug for Nibble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nibble({})", self.as_u8())
    }
}

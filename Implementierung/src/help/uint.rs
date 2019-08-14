use std::mem;
use std::ops::{Shl, Add, BitAnd};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::num::TryFromIntError;

/// Basierend auf folgender [Repository](https://github.com/thrill/thrill/blob/master/thrill/common/uint_types.hpp)
#[derive(Copy, Clone)]
pub struct UIntPair<T> {
    /// member containing lower significant integer value
    low: u32,

    /// member containing higher significant integer value
    high: T,
}

impl<T: Copy> UIntPair<T> {
    /// number of bits in the lower integer part, used a bit shift value.
    const LOW_BITS: usize = 8 * mem::size_of::<u32>();

    /// number of bits in the higher integer part, used a bit shift value.
    const HIGH_BITS: usize = 8 * mem::size_of::<T>();

    /// number of binary digits (bits) in UIntPair
    const DIGITS: usize = Self::LOW_BITS + Self::HIGH_BITS;

    /// number of bytes in UIntPair
    //const BYTES: usize = mem::size_of::<u32>() + mem::size_of::<T>();

    /// construct unit pair from lower and higher parts.
    pub fn new(l: u32, h: T) -> Self {
        Self {
            low: l,
            high: h
        }
    }
}

/// Ermöglicht die Konvertierung von u32 nach UIntPair.
impl<T: Int> From<u32> for UIntPair<T> {
    fn from(item: u32) -> Self {
        Self {
            low: item,
            high: T::MIN_VALUE
        }
    }
}

/// Ermöglicht die Konvertierung von i32 nach UIntPair.
impl<T: Int> From<i32> for UIntPair<T> {
    fn from(item: i32) -> Self {
        if item >= 0 {
            Self::from(item as u32)
        } else {
            Self {
                low: item as u32,
                high: T::MAX_VALUE
            }
        }
    }
}

/// Ermöglicht die Konvertierung von u16 nach UIntPair.
impl<T: Int> From<u16> for UIntPair<T> {
    fn from(item: u16) -> Self {
        Self::from(item as u32)
    }
}

/// Ermöglicht die Konvertierung von i16 nach UIntPair.
impl<T: Int> From<i16> for UIntPair<T> {
    fn from(item: i16) -> Self {
        Self::from(item as i32)
    }
}

/// Ermöglicht die Konvertierung von u8 nach UIntPair.
impl<T: Int> From<u8> for UIntPair<T> {
    fn from(item: u8) -> Self {
        Self::from(item as u32)
    }
}

/// Ermöglicht die Konvertierung von i8 nach UIntPair.
impl<T: Int> From<i8> for UIntPair<T> {
    fn from(item: i8) -> Self {
        Self::from(item as i32)
    }
}

/// Ermöglicht die Konvertierung von UIntPair nach u64.
impl<T: Int> From<UIntPair<T>> for u64 {
    fn from(item: UIntPair<T>) -> Self {
        let low_bits: u64 = (UIntPair::<T>::LOW_BITS as u8).into();
        item.high.into() << low_bits | (item.low as u64)
    }
}

/// Ermöglicht die Konvertierung von UIntPair nach i64.
impl<T: Int> From<UIntPair<T>> for i64 {
    fn from(item: UIntPair<T>) -> Self {
        u64::from(item) as i64
    }
}

/// addition operator right site UIntPair<T>
impl<T: Int> Add for UIntPair<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let add_low = (self.low as u64).wrapping_add(other.low as u64);
        let add_high = (add_low >> Self::LOW_BITS) as u8;
        Self {
            low: (add_low & u32::max_value() as u64) as u32,
            high: self.high + other.high + (T::from(add_high) & T::MAX_VALUE) 
        }
    }
}

/// addition operator right site u8
impl<T: Int> Add<u8> for UIntPair<T> {
    type Output = Self;

    fn add(self, other: u8) -> Self {
        self + Self::from(other)
    }
}

/// addition operator right site u16
impl<T: Int> Add<u16> for UIntPair<T> {
    type Output = Self;

    fn add(self, other: u16) -> Self {
        self + Self::from(other)
    }
}

/// addition operator right site u32
impl<T: Int> Add<u32> for UIntPair<T> {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        self + Self::from(other)
    }
}

/// addition operator right site u64
impl<T: Int> Add<u64> for UIntPair<T> {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        self + Self::from(other)
    }
}

/// addition operator left site u8
impl<T: Int> Add<UIntPair<T>> for u8 {
    type Output = UIntPair<T>;

    fn add(self, other: UIntPair<T>) -> UIntPair<T> {
        other + self
    }
}

/// addition operator left site u16
impl<T: Int> Add<UIntPair<T>> for u16 {
    type Output = UIntPair<T>;

    fn add(self, other: UIntPair<T>) -> UIntPair<T> {
        other + self
    }
}

/// addition operator left site u32
impl<T: Int> Add<UIntPair<T>> for u32 {
    type Output = UIntPair<T>;

    fn add(self, other: UIntPair<T>) -> UIntPair<T> {
        other + self
    }
}

/// addition operator left site u64
impl<T: Int> Add<UIntPair<T>> for u64 {
    type Output = Self;

    fn add(self, other: UIntPair<T>) -> Self {
        u64::from(other + self)
    }
}










/// Ermöglicht die Konvertierung von u64 nach UIntPair.
impl<T: Int> From<u64> for UIntPair<T> {
    fn from(item: u64) -> Self {
        assert!(item >> Self::DIGITS == 0, "You tried to convert a real u64 into a smaller value. You would lose information.");
        
        let low = item & u32::max_value() as u64;
        let high = (item >> Self::LOW_BITS) & T::MAX_VALUE.into();
        
        Self {
            low: low as u32,
            high: T::try_from(high).expect("From<u64> for UIntPair<T> ist schiefgelaufen.")
        }
    }
}


/// Stellt sicher, dass der Wert (in high) einen Maximal- und Minimalwert besitzt.
pub trait Int: Into<u64> + From<u8> + Copy + Shl<Output=Self> + Add<Output=Self> 
          + BitAnd<Output=Self> + Debug + TryFrom<u64, Error=TryFromIntError> {
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    
}

impl Int for u32 {
    const MAX_VALUE: Self = Self::max_value();
    const MIN_VALUE: Self = Self::min_value();
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
}

impl Int for u16 {
    const MAX_VALUE: Self = Self::max_value();
    const MIN_VALUE: Self = Self::min_value();
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
}

impl Int for u8 {
    const MAX_VALUE: Self = Self::max_value();
    const MIN_VALUE: Self = Self::min_value();
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::UIntPair;
    type u40 = UIntPair<u8>;

    #[test]
    fn test_add_random() {
        for i in 0..u32::max_value() {
            for j in 0..u32::max_value() {
                let x = u40::from(i);
                let y = u40::from(j);
                assert_eq!(i+j, u64::from(x+y) as u32);
            }
        }
    }

    #[test]
    fn test_add_borders() {
        let x = u40::from(0b1111111111111111111111111111111111111110 as u64);
        let y = 1;
  
        assert_eq!(0b1111111111111111111111111111111111111110+1,u64::from(y+x))
    }

    /// Checks the conversion from u8 to u40 
    #[test]
    fn test_from_u8() {
        for i in 0..u8::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }

    /// Checks the conversion from i8 to u40 
    #[test]
    fn test_from_i8() {
        for i in 0..i8::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }

    /// Checks the conversion from u16 to u40 
    #[test]
    fn test_from_u16() {
        for i in 0..u16::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }

    /// Checks the conversion from i16 to u40 
    #[test]
    fn test_from_i16() {
        for i in 0..i16::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }

        /// Checks the conversion from u32 to u40
    #[test]
    fn test_from_u32() {
        for i in 0..u32::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }

    /// Checks the conversion from i32 to u40 
    #[test]
    fn test_from_i32() {
        for i in 0..i32::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }

    /// Check all possible addition combinations
    #[test]
    fn test_all_addition() {
        let x: u8 = 25;
        let y: i8 = 20;
        let z: u16 = 30;
        let a: i16 = 20;
        let b: u32 = 40;
        let c: i32 = 30;
        let d: u64 = 80;
        let start = u40::from(0);
        assert_eq!(u64::from(start + x + y + z + a + b + c + d), 245);
        assert_eq!(d+(c + (b + (a + (z + (y+ (x + start)))))), 245);
    }

    

    
}











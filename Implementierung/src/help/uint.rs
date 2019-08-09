use std::mem;
use std::ops::Shl;

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
    pub fn new(l: u32, h: T) -> UIntPair<T> {
        UIntPair {
            low: l,
            high: h
        }
    }
}

/// Ermöglicht die Konvertierung von u32 nach UIntPair.
impl<T: Int> From<u32> for UIntPair<T> {
    fn from(item: u32) -> Self {
        UIntPair {
            low: item,
            high: T::MIN_VALUE
        }
    }
}

/// Ermöglicht die Konvertierung von i32 nach UIntPair.
impl<T: Int> From<i32> for UIntPair<T> {
    fn from(item: i32) -> Self {
        if item >= 0 {
            item.into()
        } else {
            UIntPair::<T> {
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

/// Ermöglicht die Konvertierung von u64 nach UIntPair.
impl<T: Int + From<u32>> From<u64> for UIntPair<T> {
    fn from(item: u64) -> Self {
        assert!(item >> Self::DIGITS == 0, "You tried to convert a real u64 into a smaller value. You would lost information.");
        
        let low = item & u32::max_value() as u64;
        let high = (item >> Self::LOW_BITS) & T::MAX_VALUE.into();
        
        UIntPair::<T> {
            low: low as u32,
            high: (high as u32).into()
        }
    }
}

/// Stellt sicher, dass der Wert (in high) einen Maximal- und Minimalwert besitzt.
pub trait Int: Into<u64> + From<u8> + Copy + Shl<Output=Self> {
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
}

impl Int for u32 {
    const MAX_VALUE: Self = Self::max_value();
    const MIN_VALUE: Self = Self::min_value();
}

impl Int for u16 {
    const MAX_VALUE: Self = Self::max_value();
    const MIN_VALUE: Self = Self::min_value();
}

impl Int for u8 {
    const MAX_VALUE: Self = Self::max_value();
    const MIN_VALUE: Self = Self::min_value();
}

#[cfg(test)]
mod tests {
    use super::UIntPair;
    type u40 = UIntPair<u8>;
    #[test]
    fn test_from_u32() {
        for i in 0..u32::max_value() {
            assert_eq!(u64::from(u40::from(i)), i as u64);
        }
    }
}











#![allow(dead_code)]
use std::mem;


/// Basierend auf folgender [Repository](https://github.com/thrill/thrill/blob/master/thrill/common/uint_types.hpp)
#[derive(Copy, Clone)]
pub struct UIntPair<T> {
    /// member containing lower significant integer value
    low: u32,

    /// member containing higher significant integer value
    high: Option<T>,
}

impl<T: Copy> UIntPair<T> {
    /// number of bits in the lower integer part, used a bit shift value.
    //const LOW_BITS: usize = 8 * mem::size_of::<u32>();

    /// number of bits in the higher integer part, used a bit shift value.
    //const HIGH_BITS: usize = 8 * mem::size_of::<T>();

    /// number of binary digits (bits) in UIntPair
    const DIGITS: usize = 8 * mem::size_of::<T>() + 8 * mem::size_of::<u32>();

    /// number of bytes in UIntPair
    //const BYTES: usize = mem::size_of::<u32>() + mem::size_of::<T>();

    /// construct unit pair from lower and higher parts.
    pub fn new(l: u32, h: T) -> UIntPair<T> {
        UIntPair {
            low: l,
            high: Some(h)
        }
    }
}

/// Ermöglicht die Konvertierung von u32 nach UIntPair.
impl<T: Copy> From<u32> for UIntPair<T> {
    fn from(item: u32) -> Self {
        UIntPair {
            low: item,
            high: None
        }
    }
}

/// Ermöglicht die Konvertierung von i32 nach UIntPair.
impl<T: Copy + Bounded> From<i32> for UIntPair<T> {
    fn from(item: i32) -> Self {
        if item >= 0 {
            item.into()
        } else {
            UIntPair::<T> {
                low: item as u32,
                high: Some(T::MAX_VALUE)
            }
        }
    }
}

/// Ermöglicht die Konvertierung von u16 nach UIntPair.
impl<T: Copy> From<u16> for UIntPair<T> {
    fn from(item: u16) -> Self {
        Self::from(item as u32)
    }
}

/// Ermöglicht die Konvertierung von i16 nach UIntPair.
impl<T: Copy + Bounded> From<i16> for UIntPair<T> {
    fn from(item: i16) -> Self {
        Self::from(item as i32)
    }
}

/// Ermöglicht die Konvertierung von u8 nach UIntPair.
impl<T: Copy> From<u8> for UIntPair<T> {
    fn from(item: u8) -> Self {
        Self::from(item as u32)
    }
}

/// Ermöglicht die Konvertierung von i8 nach UIntPair.
impl<T: Copy + Bounded> From<i8> for UIntPair<T> {
    fn from(item: i8) -> Self {
        Self::from(item as i32)
    }
}

/// Ermöglicht die Konvertierung von u64 nach UIntPair.
impl<T: Copy + Bounded + From<u32> + Into<u64>> From<u64> for UIntPair<T> {
    fn from(item: u64) -> Self {
        assert!(item >> Self::DIGITS == 0, "You tried to convert a real u64 into a smaller value. You would lost information.");
        let low = item & u32::max_value() as u64;
        let high = (item >> mem::size_of::<u32>()*8) & T::MAX_VALUE.into();
        
        UIntPair::<T> {
            low: low as u32,
            high: Some((high as u32).into())
        }
    }
}


  /* construct from an 64-bit unsigned integer
    UIntPair(const unsigned long long& a) // NOLINT
        : low_((Low)(a & low_max())),
          high_((High)((a >> low_bits) & high_max())) {
        // check for overflow
        assert((a >> (low_bits + high_bits)) == 0);
}*/

/// Stellt sicher, dass der Wert (in high) einen Maximal- und Minimalwert besitzt.
trait Bounded {
    const MAX_VALUE: Self;
    const MIN_VALUE: Self;
}

impl Bounded for u32 {
    const MAX_VALUE: u32 = u32::max_value();
    const MIN_VALUE: u32 = u32::min_value();
}








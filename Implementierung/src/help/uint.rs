use std::mem;
use std::ops::{Shl, Add, AddAssign, Sub, SubAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::num::TryFromIntError;
use std::cmp::Ordering;

/// Basierend auf folgender [Repository](https://github.com/thrill/thrill/blob/master/thrill/common/uint_types.hpp)
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct UIntPair<T> {
    /// member containing lower significant integer value
    low: u32,

    /// member containing higher significant integer value
    high: T,
}

impl<T: Int> UIntPair<T> {
    /// number of bits in the lower integer part, used a bit shift value.
    const LOW_BITS: usize = 8 * mem::size_of::<u32>();

    /// number of bits in the higher integer part, used a bit shift value.
    const HIGH_BITS: usize = 8 * mem::size_of::<T>();

    /// number of binary digits (bits) in UIntPair
    const DIGITS: usize = Self::LOW_BITS + Self::HIGH_BITS;

    /// number of bytes in UIntPair
    //const BYTES: usize = mem::size_of::<u32>() + mem::size_of::<T>();

    /// construct unit pair from lower and higher parts.
    pub fn new<E: Int + Into<Self>>(val: E) -> Self {
        val.into()
    }

    pub fn min_value() -> Self {
        Self {
            low: 0,
            high: T::MIN_VALUE
        }
    }

    pub fn max_value() -> Self {
        Self {
            low: u32::max_value(),
            high: T::MAX_VALUE
        }
    }

}

macro_rules! impl_UIntPair_traits {
    ($($int:ty)*) => {
        $(
            /// partial eq (==) with right site $int
            impl<T: Int> PartialEq<$int> for UIntPair<T> {
                fn eq(&self, other: &$int) -> bool {
                    self.eq(&Self::from(*other))
                }
            }

            /// partial eq (==) with left site $int 
            impl<T: Int> PartialEq<UIntPair<T>> for $int {
                fn eq(&self, other: &UIntPair<T>) -> bool {
                    other.eq(self)
                }
            }

            /// PartialOrd implementation (<=) for $int on the right site
            impl<T: Int> PartialOrd<$int> for UIntPair<T> {
                fn partial_cmp(&self, other: &$int) -> Option<Ordering> {
                    u64::from(*self).partial_cmp(&u64::from(*other))
                }
            }

            /// PartialOrd implementation (<=) for $int on the left site
            impl<T: Int> PartialOrd<UIntPair<T>> for $int {
                fn partial_cmp(&self, other: &UIntPair<T>) -> Option<Ordering> {
                    u64::from(*self).partial_cmp(&u64::from(*other))
                }
            }

            /// BitAnd (&) for right site $int
            impl<T: Int> BitAnd<$int> for UIntPair<T> {
                type Output = Self;

                // rhs is the "right-hand side" of the expression `a & b`
                fn bitand(self, rhs: $int) -> Self::Output {
                    Self {
                        low: self.low & rhs as u32,
                        high: self.high 
                    }
                }
            }

            /// BitAnd (&) for left site $int
            impl<T: Int> BitAnd<UIntPair<T>> for $int {
                type Output = UIntPair<T>;

                // rhs is the "right-hand side" of the expression `a & b`
                fn bitand(self, rhs: UIntPair<T>) -> Self::Output {
                    UIntPair {
                        low: self as u32 & rhs.low,
                        high: rhs.high 
                    }
                }
            }

            /// BitOr (|) for right site $int
            impl<T: Int> BitOr<$int> for UIntPair<T> {
                type Output = Self;

                fn bitor(self, rhs: $int) -> Self::Output {
                    Self {
                        low: self.low | rhs as u32,
                        high: self.high 
                    }
                }
            }

            /// BitOr (|) for left site $int
            impl<T: Int> BitOr<UIntPair<T>> for $int {
                type Output = UIntPair<T>;

                fn bitor(self, rhs: UIntPair<T>) -> Self::Output {
                    UIntPair {
                        low: self as u32 | rhs.low,
                        high: rhs.high 
                    }
                }
            }

            /// BitXor (^) for right site $int
            impl<T: Int> BitXor<$int> for UIntPair<T> {
                type Output = Self;

                fn bitxor(self, rhs: $int) -> Self::Output {
                    Self {
                        low: self.low ^ rhs as u32,
                        high: self.high 
                    }
                }
            }

            /// BitXor (^) for left site $int
            impl<T: Int> BitXor<UIntPair<T>> for $int {
                type Output = UIntPair<T>;

                fn bitxor(self, rhs: UIntPair<T>) -> Self::Output {
                    UIntPair {
                        low: self as u32 ^ rhs.low,
                        high: rhs.high 
                    }
                }
            }

            /// subtraction-assign operator right site $int
            impl<T: Int> SubAssign<$int> for UIntPair<T> {
                fn sub_assign(&mut self, other: $int) {
                    *self = *self - other;
                }
            }
            
            /// subtraction operator right site $int
            impl<T: Int> Sub<$int> for UIntPair<T> {
                type Output = Self;

                fn sub(self, other: $int) -> Self {
                    self - Self::from(other)
                }
            }

            /// addition operator left site $int
            impl<T: Int> Sub<UIntPair<T>> for $int {
                type Output = UIntPair<T>;

                fn sub(self, other: UIntPair<T>) -> UIntPair<T> {
                    UIntPair::<T>::from(self) - other
                }
            }

            /// addition operator right site $int
            impl<T: Int> Add<$int> for UIntPair<T> {
                type Output = Self;

                fn add(self, other: $int) -> Self {
                    self + Self::from(other)
                }
            }

            /// addition operator left site $int
            impl<T: Int> Add<UIntPair<T>> for $int {
                type Output = UIntPair<T>;

                fn add(self, other: UIntPair<T>) -> UIntPair<T> {
                    other + self
                }
            }

        )*
    }
}

impl_UIntPair_traits!(u8 u16 u32);

/// partial eq (==) with right site u64
impl<T: Int> PartialEq<u64> for UIntPair<T> {
    fn eq(&self, other: &u64) -> bool {
        self.eq(&Self::from(*other))
    }
}

/// partial eq (==) with left site u64
impl<T: Int> PartialEq<UIntPair<T>> for u64 {
    fn eq(&self, other: &UIntPair<T>) -> bool {
        other.eq(self)
    }
}

/// PartialOrd implementation (<=) for UIntPair<T> on the right site
impl<T: Int> PartialOrd for UIntPair<T> {
    fn partial_cmp(&self, other: &UIntPair<T>) -> Option<Ordering> {
        u64::from(*self).partial_cmp(&u64::from(*other))
    }
}

/// PartialOrd implementation (<=) for u64 on the right site
impl<T: Int> PartialOrd<u64> for UIntPair<T> {
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        u64::from(*self).partial_cmp(other)
    }
}

/// PartialOrd implementation (<=) for u64 on the left site
impl<T: Int> PartialOrd<UIntPair<T>> for u64 {
    fn partial_cmp(&self, other: &UIntPair<T>) -> Option<Ordering> {
        self.partial_cmp(&u64::from(*other))
    }
}

/// BitAnd (&) for right site UIntPair<T>
impl<T: Int> BitAnd for UIntPair<T> {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low & rhs.low,
            high: self.high & rhs.high
        }
    }
}

/// BitAnd (&) for right site u64
impl<T: Int> BitAnd<u64> for UIntPair<T> {
    type Output = u64;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: u64) -> Self::Output {
        u64::from(self) & rhs
    }
}

/// BitAnd (&) for left site u64
impl<T: Int> BitAnd<UIntPair<T>> for u64 {
    type Output = Self;

    // rhs is the "right-hand side" of the expression `a & b`
    fn bitand(self, rhs: UIntPair<T>) -> Self::Output {
        u64::from(rhs) & self
    }
}

/// BitOr (|) for right site UIntPair<T>
impl<T: Int> BitOr for UIntPair<T> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low | rhs.low,
            high: self.high | rhs.high
        }
    }
}

/// BitOr (|) for right site u64
impl<T: Int> BitOr<u64> for UIntPair<T> {
    type Output = u64;

    fn bitor(self, rhs: u64) -> Self::Output {
        u64::from(self) | rhs
    }
}

/// BitOr (|) for left site u64
impl<T: Int> BitOr<UIntPair<T>> for u64 {
    type Output = Self;

    fn bitor(self, rhs: UIntPair<T>) -> Self::Output {
        u64::from(rhs) | self
    }
}

/// BitXor (^) for right site UIntPair<T>
impl<T: Int> BitXor for UIntPair<T> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            low: self.low ^ rhs.low,
            high: self.high ^ rhs.high
        }
    }
}

/// BitXor (^) for right site u64
impl<T: Int> BitXor<u64> for UIntPair<T> {
    type Output = u64;

    fn bitxor(self, rhs: u64) -> Self::Output {
        u64::from(self) ^ rhs
    }
}

/// BitXor (^)for left site u64
impl<T: Int> BitXor<UIntPair<T>> for u64 {
    type Output = Self;

    fn bitxor(self, rhs: UIntPair<T>) -> Self::Output {
        u64::from(rhs) ^ self
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

/// bitAnd-assign operator right site with all possible types (except u64)
impl<T: Int, R: BitAnd<Self, Output=Self>> BitAndAssign<R> for UIntPair<T> {
    fn bitand_assign (&mut self, other: R) {
        *self = other & *self;
    }
}

/// bitAnd-assign operator right site with u64
impl<T: Int> BitAndAssign<u64> for UIntPair<T> {
    fn bitand_assign (&mut self, other: u64) {
        *self = Self::from(other & *self);
    }
}

/// bitOr-assign operator right site with all possible types (except u64)
impl<T: Int, R: BitOr<Self, Output=Self>> BitOrAssign<R> for UIntPair<T> {
    fn bitor_assign (&mut self, other: R) {
        *self = other | *self;
    }
}

/// bitOr-assign operator right site with u64
impl<T: Int> BitOrAssign<u64> for UIntPair<T> {
    fn bitor_assign (&mut self, other: u64) {
        *self = Self::from(other | *self);
    }
}

/// bitXor-assign operator right site with all possible types (except u64)
impl<T: Int, R: BitXor<Self, Output=Self>> BitXorAssign<R> for UIntPair<T> {
    fn bitxor_assign (&mut self, other: R) {
        *self = other ^ *self;
    }
}

/// bitXor-assign operator right site with u64
impl<T: Int> BitXorAssign<u64> for UIntPair<T> {
    fn bitxor_assign (&mut self, other: u64) {
        *self = Self::from(other ^ *self);
    }
}

/// addition-assign operator right site with all possible types (except u64)
impl<T: Int, R: Add<Self, Output=Self>> AddAssign<R> for UIntPair<T> {
    fn add_assign(&mut self, other: R) {
        *self = other + *self;
    }
}

/// addition-assign operator right site with all possible types (except u64)
impl<T: Int> AddAssign<u64> for UIntPair<T> {
    fn add_assign(&mut self, other: u64) {
        *self = Self::from(*self + other);
    }
}

/// subtraction-assign operator right site u64
impl<T: Int> SubAssign<u64> for UIntPair<T> {
    fn sub_assign(&mut self, other: u64) {
        *self = *self - other as u32;
    }
}

/// subtraction operator right site UIntPair<T>
impl<T: Int> Sub for UIntPair<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let sub_low = (self.low as u64).wrapping_sub(other.low as u64);
        let sub_high = (sub_low >> Self::LOW_BITS) as u8;
        Self {
            low: (sub_low & u32::max_value() as u64) as u32,
            high: self.high - other.high + (T::from(sub_high) & T::MAX_VALUE) 
        }
    }
}

/// subtraction operator right site u64
impl<T: Int> Sub<u64> for UIntPair<T> {
    type Output = u64;

    fn sub(self, other: u64) -> u64 {
        (self - Self::from(other)).into()
    }
}

/// addition operator left site u64
impl<T: Int> Sub<UIntPair<T>> for u64 {
    type Output = Self;

    fn sub(self, other: UIntPair<T>) -> Self {
        (UIntPair::<T>::from(self) - other).into()
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

/// addition operator right site u64
impl<T: Int> Add<u64> for UIntPair<T> {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        self + Self::from(other)
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
          + BitAnd<Output=Self> + Debug + TryFrom<u64, Error=TryFromIntError> + Sub<Output=Self> 
          + PartialEq + BitAnd<Output=Self> + BitOr<Output=Self> + BitXor<Output=Self> {
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

    /// subs and assign values
    #[test]
    fn test_sub_assign_random() {
        let mut b = u40::max_value();
        let mut c = u40::max_value();
        let mut d = u40::max_value();
        
        let mut right_val: u64 = u40::max_value().into();
        for i in 0..u16::max_value() {
            b -= i as u16;
            c -= i as u32;
            d -= i as u64;
            right_val -= i as u64;
        }

        assert_eq!(u64::from(b),right_val);
        assert_eq!(u64::from(c),right_val);
        assert_eq!(u64::from(d),right_val);
        
    }

      /// sub a lot of u32 values
   #[test]
    fn test_sub_random() {
        for i in 0..(u64::from(u40::max_value())) {
            for j in 0..(i+1) {
                let x = u40::from(i);
                let y = u40::from(j);
    
                assert_eq!(i+j, u64::from(x+y) as u64);
            }
        }
    }

    /// adds and assign a lot of u32 values
    #[test]
    fn test_add_assign_random() {
        let mut b = u40::from(0);
        let mut c = u40::from(0);
        let mut d = u40::from(0);
        
        let mut right_val: u64 = 0;
        for i in 0..u16::max_value() {
            b += i as u16;
            c += i as u32;
            d += i as u64;
            right_val += i as u64;
        }

        assert_eq!(u64::from(b),right_val);
        assert_eq!(u64::from(c),right_val);
        assert_eq!(u64::from(d),right_val);
        
    }

    /// adds a lot of u32 values
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

    /// Checks the Borders off the addition
    #[test]
    fn test_add_borders() {
        let x = u40::from(0b1111111111111111111111111111111111111110 as u64);
        let y = 1 as u32;
  
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
        let z: u16 = 30;
        let b: u32 = 40;
        let d: u64 = 80;
        let start = u40::from(0);
        assert_eq!(u64::from(start + x + z + b + d), 175);
        assert_eq!(d + (b + (z + (x + start))), 175);
    }

    /// Check all possible sub combinations
    #[test]
    fn test_all_subs() {
        let x: u8 = 25;
        let z: u16 = 30;
        let b: u32 = 40;
        let d: u64 = 80;
        let start = u40::from(1000);
        assert_eq!(u64::from(start - x - z - b - d), 825);
        assert_eq!(2000 as u64 - (start-x), 1025);
    }

    // TODO: Teste den ganzen BitAnd, BitOr, BitXor (+Assigns), PartialEq, Add und Sub Overflows, PartialOrd Kram
}
// TODO Bitshifts implementieren
#![allow(dead_code)]  
use uint::{u40, u48};


pub trait PredecessorSet<T> {
    fn insert(&mut self,element: T);
    fn delete(&mut self,element: T);
    fn predecessor(&self,number: T) -> Option<T>;
    fn successor(&self,number: T) -> Option<T>; // Optional
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>; 
    fn contains(&self, number: T) -> bool;
}


pub trait Splittable {
    fn split_integer_down(&self) -> (usize,u16,u16);
}

impl Splittable for u40 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        // TODO: Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 20) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = u64::from(*self) & 0xFFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u16 = (low >> 10) as u16 ;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u16 = (u64::from(*self) & 0x3FF) as u16;
        (i, j, k) 
    }
}

impl Splittable for u48 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        // TODO: Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 24) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = u64::from(*self) & 0xFFFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u16 = (low >> 12) as u16 ;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u16 = (u64::from(*self) & 0xFFF) as u16;
        (i, j, k) 
    }
}

impl Splittable for u32 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        let i: usize = (*self >> 16) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = *self & 0xFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u16 = (low >> 8) as u16;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u16 = (*self & 255) as u16;
        (i,j,k)
    }
}

impl Splittable for u64 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        let i: usize = (*self >> 32) as usize;
        // Die niedrigwertigsten 32 Bits element[32..63]
        let low = *self & 0xFFFFFFFF;
        // Bits 16 bis 32
        let j: u16 = (low >> 16) as u16;
        // Die niedrigwertigsten 16 Bits element[0..15]
        let k: u16 = (*self & 0xFFFF) as u16;
        (i,j,k)
    }
} 

pub enum PointerEnum<'a, T: 'a, E: 'a> {
    First(&'a mut T),
    Second(&'a mut E)
}

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein T oder ein E Objekt zeigt. Wichtig ist hierbei, dass T mit einem Vielfachen von 2 alligned werden muss!
pub struct Pointer<T,E> {
    pointer: *mut T,
    phantom: std::marker::PhantomData<E>,
}

impl<T: Clone,E: Clone> Clone for Pointer<T,E> {
    fn clone(&self) -> Self {
        if self.pointer.is_null() {
            Self::null()
        } else {
            match self.get() {
                PointerEnum::First(x) => Self::from_first(Box::new(x.clone())),
                PointerEnum::Second(x) => Self::from_second(x as *const E),
            }
        }
    }
}

impl<T,E> Drop for Pointer<T,E> {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }

        if (self.pointer as usize % 2) == 0 {
            unsafe { Box::from_raw(self.pointer) };
        } else {
            debug_assert!((self.pointer as usize % 2) == 1);

            unsafe { Box::from_raw((self.pointer as usize -1) as *mut E) };
        }
    }
}

impl<T,E> Pointer<T,E> {
    pub fn from_first(b: Box<T>) -> Self {
        let pointer = Box::into_raw(b);
        assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        assert!((pointer as usize % 2) == 0);

        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData
        }
    }

    pub fn from_second(b: *const E) -> Self {
        let pointer = b;
        assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        assert!((pointer as usize % 2) == 0);

        let pointer = (pointer as usize + 1) as *mut T;
        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData
        }
    }


    pub fn get(&self) -> PointerEnum<T,E> {
        if self.pointer.is_null() {
            panic!("Pointer<T> is null!");
        }

        if (self.pointer as usize % 2) == 0 {
            unsafe {PointerEnum::First(&mut (*self.pointer))}
        } else {
            debug_assert!((self.pointer as usize % 2) == 1);

            unsafe {PointerEnum::Second(&mut *((self.pointer as usize -1) as *mut E))}
        }
    }

    pub fn null() -> Self {
        Self {
            pointer: std::ptr::null_mut(),
            phantom: std::marker::PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }
}
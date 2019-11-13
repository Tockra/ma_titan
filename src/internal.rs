#![allow(dead_code)]  

pub trait PredecessorSet<T> {
    fn insert(&mut self, element: T);
    fn delete(&mut self, element: T);
    fn predecessor(&self, number: T) -> Option<T>;
    fn successor(&self, number: T) -> Option<T>; // Optional
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>;
    fn contains(&self, number: T) -> bool;
}

pub trait Splittable {
    fn split_integer_down(&self) -> (usize, u16, u16, u16);
}

impl Splittable for u64 {
    #[inline]
    fn split_integer_down(&self) -> (usize, u16, u16, u16)  {
        // Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 42) as usize;
        // Die niedrigwertigsten 24 Bits element[16..39]
        let low = u64::from(*self) & 4398046511103;
        // Bits 16 bis 23 element[8..15]
        let l: u16 = (low >> 28) as u16;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let low = low & 268435455;
        let j: u16 = (low >> 14) as u16;
        let k: u16 = (low & 16383) as u16;
        (i, l, j, k)
    }
}

pub enum PointerEnum<'a, T: 'a, E: 'a> {
    First(&'a mut T),
    Second(&'a mut E)
}

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein T oder ein E Objekt zeigt. Wichtig ist hierbei, dass T mit einem Vielfachen von 2 alligned werden muss!
pub struct Pointer<T, E> {
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

impl<T, E> Drop for Pointer<T, E> {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }

        if (self.pointer as usize % 2) == 0 {
            unsafe { Box::from_raw(self.pointer) };
        } else {
            debug_assert!((self.pointer as usize % 2) == 1);

            //unsafe { Box::from_raw((self.pointer as usize -1) as *mut E) };
        }
    }
}

impl<T, E> Pointer<T, E> {
    pub fn from_first(b: Box<T>) -> Self {
        let pointer = Box::into_raw(b);
        debug_assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        debug_assert!((pointer as usize % 2) == 0);

        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn from_second(b: *const E) -> Self {
        let pointer = b;
        assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        assert!((pointer as usize % 2) == 0);

        let pointer = (pointer as usize + 1) as *mut T;
        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn get(&self) -> PointerEnum<T, E> {
        if self.pointer.is_null() {
            panic!("Pointer<T> is null!");
        }

        if (self.pointer as usize % 2) == 0 {
            unsafe { PointerEnum::First(&mut (*self.pointer)) }
        } else {
            debug_assert!((self.pointer as usize % 2) == 1);

            unsafe { PointerEnum::Second(&mut *((self.pointer as usize - 1) as *mut E)) }
        }
    }

    pub fn null() -> Self {
        Self {
            pointer: std::ptr::null_mut(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }
}

/// Dies ist ein Wrapper um die Mphf-Hashfunktion. Es wird nicht die interne Implementierung verwendet, da
/// bei dieser das Gamma nicht beeinflusst werden kann.
use crate::default::build::GAMMA;
use boomphf::Mphf;

#[derive(Clone)]
pub struct MphfHashMap<K, V> {
    hash_function: Mphf<K>,
    objects: Box<[V]>,
}

impl<
        K: Into<u16>
            + std::marker::Send
            + std::marker::Sync
            + std::hash::Hash
            + std::fmt::Debug
            + Clone,
        V,
    > MphfHashMap<K, V>
{
    #[inline]
    pub fn new(keys: Box<[K]>, objects: Box<[V]>) -> Self {
        Self {
            hash_function: Mphf::new_parallel(GAMMA, &keys.to_vec(), None),
            objects: objects,
        }
    }

    /// Der zum `key` gehörende gehashte Wert wird aus der Datenstruktur ermittelt. Hierbei muss sichergestellt sein
    /// das zu `key` ein Schlüssel gehört. Anderenfalls sollte `try_hash` verwendet werden
    ///
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get(&self, key: &K) -> &V {
        let hash = self.hash_function.try_hash(key).unwrap() as usize;
        unsafe { self.objects.get_unchecked(hash) }
    }

    /// Der zum `key` gehörende gehashte Wert wird aus der Datenstruktur ermittelt. Hierbei muss sichergestellt sein
    /// das zu `key` ein Schlüssel gehört. Anderenfalls sollte `try_hash` verwendet werden
    ///
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> &mut V {
        let hash = self.hash_function.try_hash(key).unwrap() as usize;

        unsafe { self.objects.get_unchecked_mut(hash) }
    }
}

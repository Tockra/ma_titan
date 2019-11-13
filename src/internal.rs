use uint::{u40, u48};

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
    fn split_integer_down(&self) -> (usize, u8, u8, u8);
}

impl Splittable for u40 {
    #[inline]
    fn split_integer_down(&self) -> (usize, u8, u8, u8) {
        // Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 24) as usize;
        // Die niedrigwertigsten 24 Bits element[16..39]
        let low = u64::from(*self) & 0xFFFFFF;
        // Bits 16 bis 23 element[8..15]
        let l: u8 = (low >> 16) as u8;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let low = low & 0xFFFF;
        let j: u8 = (low >> 8) as u8;
        let k: u8 = (u64::from(*self) & 0xFF) as u8;
        (i, l, j, k)
    }
}

impl Splittable for u48 {
    #[inline]
    fn split_integer_down(&self) -> (usize, u8, u8, u8)  {
        // Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 24) as usize;
        // Die niedrigwertigsten 24 Bits element[16..39]
        let low = u64::from(*self) & 0xFFFFFF;
        // Bits 16 bis 23 element[8..15]
        let l: u8 = (low >> 16) as u8;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let low = low & 0xFFFF;
        let j: u8 = (low >> 8) as u8;
        let k: u8 = (u64::from(*self) & 0xFF) as u8;
        (i, l, j, k)
    }
}

impl Splittable for u64 {
    #[inline]
    fn split_integer_down(&self) -> (usize, u8, u8, u8)  {
        // Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 24) as usize;
        // Die niedrigwertigsten 24 Bits element[16..39]
        let low = u64::from(*self) & 0xFFFFFF;
        // Bits 16 bis 23 element[8..15]
        let l: u8 = (low >> 16) as u8;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let low = low & 0xFFFF;
        let j: u8 = (low >> 8) as u8;
        let k: u8 = (u64::from(*self) & 0xFF) as u8;
        (i, l, j, k)
    }
}

pub enum PointerEnum<'a, T: 'a, E: 'a> {
    First(&'a mut T),
    Second(&'a mut E)
}

pub struct Pointer2<T, E> {
    pointer: *mut T,
    phantom: std::marker::PhantomData<E>,
}

impl<T: Clone,E: Clone> Clone for Pointer2<T,E> {
    fn clone(&self) -> Self {
        if self.pointer.is_null() {
            Self::null()
        } else {
            match self.get() {
                PointerEnum::First(x) => Self::from_first(Box::new(x.clone())),
                PointerEnum::Second(x) => Self::from_second(Box::new(x.clone())),
            }
        }
    }
}

impl<T, E> Drop for Pointer2<T, E> {
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

impl<T, E> Pointer2<T, E> {
    pub fn from_first(b: Box<T>) -> Self {
        let pointer = Box::into_raw(b);
        debug_assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        debug_assert!((pointer as usize % 2) == 0);

        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn from_second(b: Box<E>) -> Self {
        let pointer = Box::into_raw(b);
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

use boomphf::Mphf;
use crate::default::build::GAMMA;
#[derive(Clone)]
pub struct MphfHashMap<K, V> {
    hash_function: Option<Mphf<K>>,
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
        if objects.len() > 1 {
            Self {
                hash_function: Some(Mphf::new_parallel(GAMMA, &keys.to_vec(), None)),
                objects: objects,
            }
        } else {
            Self {
                hash_function: None,
                objects: objects,
            }
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
        if !self.hash_function.is_none() {
            let hash = self.hash_function.as_ref().unwrap().try_hash(key).unwrap() as usize;
            unsafe { self.objects.get_unchecked(hash) }
        } else {
            unsafe { self.objects.get_unchecked(0) }
        }

    }

    /// Der zum `key` gehörende gehashte Wert wird aus der Datenstruktur ermittelt. Hierbei muss sichergestellt sein
    /// das zu `key` ein Schlüssel gehört. Anderenfalls sollte `try_hash` verwendet werden
    ///
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> &mut V {
        if !self.hash_function.is_none() {
            let hash = self.hash_function.as_ref().unwrap().try_hash(key).unwrap() as usize;
            unsafe { self.objects.get_unchecked_mut(hash) }
        } else {
            unsafe { self.objects.get_unchecked_mut(0) }
        }
    }
}

pub struct MphfHashMapThres<K,T> {
    pointer: Pointer2<MphfHashMap<K,T>,(Box<[K]>,Box<[T]>)>,
}

impl<K: Clone,T: Clone> Clone for MphfHashMapThres<K,T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone()
        }
    }
}

impl<K: Eq + std::fmt::Display + std::marker::Send + std::marker::Sync + std::hash::Hash + std::fmt::Debug + Into<u16> + Ord + Copy + std::hash::Hash,T> MphfHashMapThres<K,T> {
    pub fn new(keys: Box<[K]>, objects: Box<[T]>) -> Self {
        if keys.len() >= 1 {
            Self {
                pointer: Pointer2::from_second(Box::new((keys.to_vec().into_boxed_slice(),objects))),
            }
        } else {

            Self {
                pointer: Pointer2::from_first(Box::new(MphfHashMap::new(keys, objects))),
            }
        }

    }

    pub fn get_mut(&mut self, k: &K) -> &mut T {
        match self.pointer.get() {
            PointerEnum::Second((keys,values)) => {
                match keys.binary_search(k) {
                    Ok(x) => unsafe { values.get_unchecked_mut(x)},
                    _ => panic!("get in internal wurde mit ungültigem Schlüssel {} aufgerufen.", k),
                }
            },
            PointerEnum::First(x) => {
                x.get_mut(k)
            },
        }
    }

    pub fn get(&self, k: &K) -> &T {
        match self.pointer.get() {
            PointerEnum::Second((keys,values)) => {
                match keys.binary_search(k) {
                    Ok(x) => unsafe { values.get_unchecked(x)},
                    _ => panic!("get in internal wurde mit ungültigem Schlüssel {} aufgerufen.", k),
                }
            },
            PointerEnum::First(x) => {
                x.get(k)
            },
        }
    }
}
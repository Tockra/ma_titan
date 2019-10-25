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

pub enum PointerEnum<T: 'static, E: 'static> {
    First(&'static mut T),
    Second(&'static mut E)
}

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein T oder ein E Objekt zeigt. Wichtig ist hierbei, dass T mit einem Vielfachen von 2 alligned werden muss!
pub struct Pointer<T,E> {
    pointer: *mut T,
    phantom: std::marker::PhantomData<E>,
}

impl<T:'static + Clone,E:'static + Clone> Clone for Pointer<T,E> {
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
        debug_assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        debug_assert!((pointer as usize % 2) == 0);

        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData
        }
    }

    pub fn from_second(b: Box<E>) -> Self {
        let pointer = Box::into_raw(b);
        debug_assert!(std::mem::align_of::<T>() % 2 == 0 && std::mem::align_of::<E>() % 2 == 0);
        debug_assert!((pointer as usize % 2) == 0);

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

/// Dies ist ein Wrapper um die Mphf-Hashfunktion. Es wird nicht die interne Implementierung verwendet, da 
/// bei dieser das Gamma nicht beeinflusst werden kann. 
use crate::default::build::GAMMA;
use boomphf::Mphf;

#[derive(Clone)]
pub struct MphfHashMap<K,V> {
    hash_function: Mphf<K>,
    objects: Box<[V]>,
}

impl<K: Into<u16> + std::marker::Send + std::marker::Sync + std::hash::Hash + std::fmt::Debug + Clone,V> MphfHashMap<K,V> {
    pub fn new(keys: Box<[K]>, objects: Box<[V]>) -> Self {
        Self {
            hash_function: Mphf::new_parallel(GAMMA,&keys.to_vec(),None),
            objects: objects
        }
    }

       /// Mit Hilfe dieser Funktion kann die perfekte Hashfunktion verwendet werden. 
    /// Es muss beachtet werden, dass sichergestellt werden muss, dass der verwendete Key auch existiert!
    /// 
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn try_get(&self, key: K, lx_top: &[u64]) -> Option<&V> {
        let k: u16 = key.clone().into();
        let index = (k/64) as usize;
        let in_index_mask = 1<<(63-(k % 64));

        // Hier wird überprüft ob der Key zur Initialisierung bekannt war. Anderenfalls wird die Hashfunktion nicht ausgeführt.
        if (lx_top[index] & in_index_mask) != 0 {
            let hash = self.hash_function.try_hash(&key)? as usize;
            self.objects.get(hash)
        } else {
            None
        }
    }

    /// Der zum `key` gehörende gehashte Wert wird aus der Datenstruktur ermittelt. Hierbei muss sichergestellt sein
    /// das zu `key` ein Schlüssel gehört. Anderenfalls sollte `try_hash` verwendet werden
    /// 
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get(&mut self, key: &K) -> &mut V {
        let hash = self.hash_function.try_hash(key).unwrap() as usize;
        self.objects.get_mut(hash).unwrap()
    }

}

type HashMap<K,T> = MphfHashMap<K,T>;

pub struct MphfHashMapThres<K,T> {
    pointer: Pointer<HashMap<K,T>,(Box<[K]>,Box<[T]>)>,
}

impl<K:'static + Clone,T:'static + Clone> Clone for MphfHashMapThres<K,T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone()
        }
    }
}

extern crate stats_alloc;

/// DEBUG: Zur Evaluierung der Datenstruktur Nur in *_space Branches vorhanden
use std::sync::atomic::{AtomicUsize, Ordering};
pub static LEVEL_COUNT: AtomicUsize = AtomicUsize::new(0);
pub static HASH_MAPS_IN_BYTES: AtomicUsize = AtomicUsize::new(0);
pub static NUMBER_OF_KEYS: AtomicUsize = AtomicUsize::new(0);


use stats_alloc::Region;
use stats_alloc::{StatsAlloc};
use std::alloc::System;



impl<K:'static + Eq + std::fmt::Display + std::marker::Send + std::marker::Sync + std::hash::Hash + std::fmt::Debug + Into<u16> + Ord + Copy + std::hash::Hash,T: 'static> MphfHashMapThres<K,T> {
    pub fn new(GLOBAL: &'static StatsAlloc<System>, keys: Box<[K]>, objects: Box<[T]>) -> Self {
        LEVEL_COUNT.fetch_add(1, Ordering::SeqCst);
        NUMBER_OF_KEYS.fetch_add(keys.len(),Ordering::SeqCst);
        if keys.len() <= 161 {
            Self {
                pointer: Pointer::from_second(Box::new((keys.to_vec().into_boxed_slice(),objects))),
            }
        } else {
            let reg = Region::new(GLOBAL);

            let result = Self {
                pointer: Pointer::from_first(Box::new(HashMap::new(keys, objects))),
            };
            let change = reg.change();
            HASH_MAPS_IN_BYTES.fetch_add(change.bytes_current_used, Ordering::SeqCst);
            result
        }

    }

    pub fn get(&mut self, k: &K) -> &mut T {
        match self.pointer.get() {
            PointerEnum::Second((keys,values)) => {
                match keys.binary_search(k) {
                    Ok(x) => values.get_mut(x).unwrap(),
                    _ => panic!("get in internal wurde mit ungültigem Schlüssel {} aufgerufen.", k),
                }
            },
            PointerEnum::First(x) => {
                x.get(k)
            },
        }
    }

    pub fn try_get(&self, key: K, lx_top: &[u64]) -> Option<&T> {
        let k: u16 = key.clone().into();
        let index = (k/64) as usize;
        let in_index_mask = 1<<(63-(k % 64));

        // Hier wird überprüft ob der Key zur Initialisierung bekannt war. Anderenfalls wird die Hashfunktion nicht ausgeführt.
        if (lx_top[index] & in_index_mask) != 0 {
             match self.pointer.get() {
                PointerEnum::Second((keys,values)) => {
                    match keys.binary_search(&key) {
                        Ok(x) => values.get(x),
                        _ => None,
                    }
                },
                PointerEnum::First(x) => {
                    x.try_get(key,lx_top)
                },
             }
        } else {
            None
        }
    }
}
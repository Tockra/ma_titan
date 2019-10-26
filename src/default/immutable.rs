use uint::{u40, u48};

use crate::default::build::STreeBuilder;
use crate::internal::Splittable;
/// Die L2-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf eine
/// L3-Ebene zeigt.
pub type L2Ebene<T> = LevelPointer<L3Ebene<T>,T>;



/// Die L3-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf 
/// ein Indize der STree.element_list zeigt.
pub type L3Ebene<T> = LevelPointer<usize,T>;

use crate::internal::{self, PointerEnum};

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein usize-Objekt zeigt (Index aus Elementliste),
/// oder auf ein Levelobjekt
#[derive(Clone)]
pub struct LevelPointer<T: 'static,E: 'static> {
    pointer: internal::Pointer<Level<T,E>,usize>
}

impl<T,E> LevelPointer<T,E> {
    fn minimum(&self) -> usize {
        match self.pointer.get() {
            PointerEnum::First(l) => {
                (*l).minimum
            },

            PointerEnum::Second(e) => {
                *e
            }
        }
    }

    fn maximum(&self) -> usize {
        match self.pointer.get() {
            PointerEnum::First(l) => {
                (*l).maximum
            },

            PointerEnum::Second(e) => {
                *e
            }
        }
    }

    pub fn from_level(level_box: Box<Level<T,E>>) -> Self {
        Self {
            pointer: internal::Pointer::from_first(level_box)
        }
    }

    pub fn get(&self) -> PointerEnum<Level<T,E>, usize> {
        self.pointer.get()
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    pub fn from_null() -> Self {
        Self {
            pointer: internal::Pointer::null()
        }
    }

    pub fn from_usize(usize_box: Box<usize>) -> Self {
        Self {
            pointer: internal::Pointer::from_second(usize_box)
        }
    }

    pub fn change_to_usize(&mut self, usize_box: Box<usize>) {
        self.pointer = internal::Pointer::from_second(usize_box);
    }
}

/// Statische Predecessor-Datenstruktur. Sie verwendet perfektes Hashing und ein Array auf der Element-Listen-Ebene.
/// Sie kann nur sortierte und einmalige Elemente entgegennehmen.
#[derive(Clone)]
pub struct STree<T: 'static> {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2-Ebene je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    pub root_table: Box<[L2Ebene<T>]>,
    
    /// Das Root-Top-Array speichert für jeden Eintrag `root_table[i][x]`, der belegt ist, ein 1-Bit, sonst einen 0-Bit.
    /// Auch hier werden nicht 2^20 Einträge, sondern lediglich [u64;2^20/64] gespeichert.
    /// i steht dabei für die Ebene der root_tabelle. Ebene i+1 beinhaltet an Index [x] immer 64 Veroderungen aus Ebene i. 
    /// Somit gilt |root_table[i+1]| = |root_table[i]|/64  
    pub root_top: TopArray<T,usize>,

    /// Die Elementliste beinhaltet einen Vektor konstanter Länge mit jeweils allen gespeicherten Elementen in sortierter Reihenfolge.
    pub element_list: Box<[T]>,
}

/// Liste von Bitarrays zur Speicherung der LX-Top-Datenstrukturen
/// Zur Speicherplatzreduzierung werden die Längen der Arrays weggeworfen und zum Drop-Zeitpunkt erneut berechnet 
pub struct TopArray<T,V> {
    /// 2-dimensionales Array mit 
    data: Box<[*mut u64]>,
    
    // Länge der untersten Ebene. Kleiner Tradeoff zwischen Länge aller Ebenen Speichern und Level der tiefsten Ebene Speichern...
    lowest_len: usize,

    /// entspricht dem Nutzdatentyp (u40,u48 oder u64)
    phantom: std::marker::PhantomData<T>,

    /// Entspricht dem Key der gehasht wird. Wenn Wurzeltop, dann usize, wenn LXTop dann LXKey
    phantom_type: std::marker::PhantomData<V>,
}

impl<T,V> Drop for TopArray<T,V> {
    fn drop(&mut self) {
        // Hier muss editiert werden, wenn die Größen der L2- und L3- Level angepasst werden sollen
        let mut length = Self::get_length();

        // Solange Länge / 2^i > 256
        for (i,&ptr) in self.data.into_iter().enumerate() {
            length = length>>(i+1)*6;
            unsafe {
                Box::from_raw(std::slice::from_raw_parts_mut(ptr, length));
            } 
        }
    }
}

impl<T,V> Clone for TopArray<T,V> {
    fn clone(&self) -> Self {
        // Hier muss editiert werden, wenn die Größen der L2- und L3- Level angepasst werden sollen
        let mut length = Self::get_length();

        // Lege alle Rootarrays an
        let mut top_arrays = vec![];

        // Solange Länge / 2^i > 256
        for (i,&ptr) in self.data.iter().enumerate() {
            length = length>>(i+1)*6;
            let mut tmp = vec![];
            unsafe {
                for i in 0..length {
                    tmp.push(*ptr.add(i));
                }
            } 
            top_arrays.push(Box::into_raw(tmp.into_boxed_slice()) as *mut u64);
        }
        Self {
            data: top_arrays.into_boxed_slice(),
            lowest_len: length,
            phantom: std::marker::PhantomData,
            phantom_type: std::marker::PhantomData,
        }
    }
}

impl<T,V> TopArray<T,V> {
    #[inline]
    fn get_length() -> usize {
        if std::mem::size_of::<V>() == std::mem::size_of::<usize>() {
            1 << std::mem::size_of::<T>()*8/2
        } else if std::mem::size_of::<V>() == std::mem::size_of::<LXKey>() {
            1 << std::mem::size_of::<T>()*8/4
        } else {
            panic!("Ungültige Parameterkombination vom TopArray!")
        }
    }
}

impl<T,V> TopArray<T,V> {
    /// Erzeugt mehrere Ebenen für einen Bitvector der Länge length
    #[inline]
    pub fn new() -> Self {
        // Hier muss editiert werden, wenn die Größen der L2- und L3- Level angepasst werden sollen
        let mut length = Self::get_length();

        // Lege alle Rootarrays an
        let mut top_arrays = vec![];
        // Solange Länge / 64^i > 64
        while length >= 64 {
            length = length>>6;
            top_arrays.push(Box::into_raw(vec![0_u64;length].into_boxed_slice()) as *mut u64);
        }

        Self {
            data: top_arrays.into_boxed_slice(),
            lowest_len: length,
            phantom: std::marker::PhantomData,
            phantom_type: std::marker::PhantomData,
        }
    }

    #[inline]
    const fn get_bit_mask(in_index: usize) -> u64 {
        1<<63-in_index
    }

    /// Baut das Root-Top-Array mit Hilfe der sich in der Datenstruktur befindenden Werte.
    #[inline]
    pub fn set_bit(&mut self, bit: usize) {
        let mut index = bit/64;
        let mut in_index = bit%64;

        // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
        for i in 0..(self.data.len()) {
            // Aktueller in_index wird für Bitmaske verwendet
            let bit_mask = Self::get_bit_mask(in_index);
            let bit_window = unsafe { self.data.get_unchecked(i).add(index) };

            in_index = index%64;
            index = index/64;
    
            unsafe {
                *bit_window = *bit_window | bit_mask;
            }

        }
    }

    #[inline]
    pub fn is_set(&self, bit: usize) -> bool {
        let (index,in_index) = (bit/64,bit%64);
        let bit_mask = Self::get_bit_mask(in_index);
        let bit_window = unsafe { self.data.get_unchecked(0).add(index) };

        unsafe {
            *bit_window & bit_mask != 0
        }
    }

    #[inline]
    fn get_next_set_bit_translation(&self, index: usize, last_level: usize) -> usize {
        let mut index = index;
        for i in (0..(last_level)).rev() {
            let zeros_to_bit = unsafe {*self.data.get_unchecked(i).add(index)};
            index = index * 64 + zeros_to_bit.leading_zeros() as usize;
        }
        index
    }

    /// Diese Funktion as nächste Bit zurück, dass hinter `bit` gesetzt ist.
    #[inline]
    pub fn get_next_set_bit(&self, bit: usize) -> Option<usize> {
        let mut index = bit/64;
        let mut in_index = bit%64;

        // Steigt alle Ebenen des TopArrays herunter und prüft, ob in den 64-Bit Blöcken bereits das nachfolgende Bit liegt.
        for level in 0..(self.data.len()) {
            let bit_mask: u64 = u64::max_value().checked_shr(in_index as u32 + 1).unwrap_or(0); 
            let zeros_to_bit = unsafe {*self.data.get_unchecked(level).add(index) & bit_mask};

            if zeros_to_bit != 0 {
                let zeros = zeros_to_bit.leading_zeros() as usize;
                if zeros != 0 {
                     return Some(self.get_next_set_bit_translation(index * 64 + zeros, level));
                }
            }

            if level < self.data.len()-1 {
                in_index = index%64;
                index = index / 64 ;
            }

        }

        let bit_mask: u64 = u64::max_value().checked_shr(in_index as u32 + 1).unwrap_or(0); 
        let mut zeros_to_bit = unsafe {(*self.data.get_unchecked(self.data.len()-1).add(index))& bit_mask};

        for i in (index)..self.lowest_len {
            if zeros_to_bit != 0 {
                return Some(self.get_next_set_bit_translation(i * 64 + zeros_to_bit.leading_zeros() as usize, self.data.len()-1));
            }

            if i < self.lowest_len-1 {
                zeros_to_bit = unsafe {*self.data.get_unchecked(self.data.len()-1).add(i+1)};
            }
        }
        None
    }

        #[inline]
    fn get_prev_set_bit_translation(&self, index: usize, last_level: usize) -> usize {
        let mut index = index;
        for i in (0..(last_level)).rev() {
            let zeros_to_bit = unsafe {*self.data.get_unchecked(i).add(index)};
            index = index * 64 + 63 - zeros_to_bit.trailing_zeros() as usize;
        }
        index
    }

    /// Diese Funktion as nächste Bit zurück, dass vor `bit` gesetzt ist.
    #[inline]
    pub fn get_prev_set_bit(&self, bit: usize) -> Option<usize> {
        let mut index = bit/64;
        let mut in_index = bit%64;

        // Steigt alle Ebenen des TopArrays herunter und prüft, ob in den 64-Bit Blöcken bereits das Vorgänger Bit liegt.
        for level in 0..self.data.len() {
            let bit_mask: u64 = u64::max_value().checked_shl(64-in_index as u32).unwrap_or(0); 

            let zeros_to_bit = unsafe {*self.data.get_unchecked(level).add(index) & bit_mask};
            if zeros_to_bit != 0 {
                let zeros = zeros_to_bit.trailing_zeros();

                if zeros != 0 {
                    return Some(self.get_prev_set_bit_translation(index * 64 + 63 - zeros as usize, level)); 
                }

            }

            if level < self.data.len()-1 {
                in_index = index%64;
                index = index / 64 ;
            }
        }
        
        let bit_mask: u64 = u64::max_value().checked_shl(64-in_index as u32).unwrap_or(0); 
        let mut zeros_to_bit = unsafe {(*self.data.get_unchecked(self.data.len()-1).add(index))& bit_mask};

        for i in (0..(index+1)).rev() {
            if zeros_to_bit != 0 {
                return Some(self.get_prev_set_bit_translation(i * 64 + 63 - zeros_to_bit.trailing_zeros() as usize, self.data.len()-1));
            }

            if i > 0 {
                zeros_to_bit = unsafe {*self.data.get_unchecked(self.data.len()-1).add(i-1)};
            }
        }

        None
    }
}

/// Dieser Trait dient als Platzhalter für u40, u48 und u64. 
/// Er stellt sicher das der generische Parameter gewisse Traits implementiert und die New-Methode besitzt.
/// Zusätzlich wird die Größe des Root-Arrays in Form einer Funktion rückgebar gemacht.
pub trait Int: Ord + PartialOrd + From<u64> + Into<u64> + Copy + Splittable { 
    fn new(k: u64) -> Self {
        Self::from(k)
    }
    fn root_array_size() -> usize {
        1 << (std::mem::size_of::<Self>()*8/2)
    }
}

impl Int for u40 {

}

impl Int for u48 {

}

impl Int for u64 {

}

pub type LXKey = u16;
impl<T: Int> STree<T> {
    /// Gibt einen STree mit den in `elements` enthaltenen Werten zurück.
    ///
    /// # Arguments
    ///
    /// * `elements` - Eine Liste mit sortierten u40-Werten, die in die statische Datenstruktur eingefügt werden sollten. Kein Wert darf doppelt vorkommen! 
    #[inline]
    pub fn new(elements: Box<[T]>) -> Self {
        let mut builder = STreeBuilder::<T>::new(elements.clone());

        let root_top = builder.get_root_top();
        STree {
            root_table: builder.build(),
            root_top: root_top,
            element_list: elements,
        }
    }


    /// Gibt die Anzahl der in self enthaltenen Elemente zurück.
    #[inline]
    pub fn len(&self) -> usize {
        self.element_list.len()
    }

    /// Gibt das in der Datenstruktur gespeicherte Minimum zurück. Falls die Datenstruktur leer ist, wird None zurückgegeben.
    #[inline]
    pub fn minimum(&self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }
        Some(self.element_list[0])
    }

    /// Gibt das in der Datenstruktur gespeicherte Minimum zurück. Falls die Datenstruktur leer ist, wird None zurückgegeben.
    #[inline]
    pub fn maximum(&self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }
        Some(self.element_list[self.len() - 1])
    }

    /// Gibt das Maximum der übergebenen Ebene zurück.
    /// 
    /// # Arguments
    ///
    /// * `lx` - Referenz auf die Ebene, dessen Maximum zurückgegeben werden soll.
    #[inline]
    pub fn maximum_level<E>(&self, lx: &Level<E,T>) -> T {
        self.element_list[lx.maximum]
    }

    /// Gibt das Minimum der übergebenen Ebene zurück.
    /// 
    /// # Arguments
    ///
    /// * `lx` - Referenz auf die Ebene, dessen Minimum zurückgegeben werden soll.
    #[inline]
    pub fn minimum_level<E>(&self, lx: &Level<E, T>) -> T {
        self.element_list[lx.minimum]
    }



    /// Diese Methode gibt den Index INDEX des größten Elements zurück für das gilt element_list[INDEX]<=element>.
    /// Somit kann mit dieser Methode geprüft werden, ob ein Element in der Datenstruktur enthalten ist. Dann wird der Index dieses Elements zurückgegeben.
    /// Ist das Element nicht enthalten, wird der "Nachfolger" dieses Elements zurückgegeben.
    /// 
    /// # Arguments
    ///
    /// * `element` - Evtl. in der Datenstruktur enthaltener Wert, dessen Index zurückgegeben wird. Anderenfalls wird der Index des Vorgängers von `element` zurückgegeben.
    #[inline]
    pub fn locate_or_pred(&self, element: T) -> Option<usize> {
        // Paper z.1 
        if element < self.minimum().unwrap() {
            return None;
        } 

        let (i,j,k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || element < self.element_list[self.root_table[i].minimum()] {
            return self.root_top.get_prev_set_bit(i as usize)
                .map(|x| self.root_table[x].maximum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none() || element < self.element_list[third_level.unwrap().minimum()] {
                    let new_j = second_level.lx_top.get_prev_set_bit(j as usize);
                    return new_j
                        .and_then(|x| second_level.try_get(x as LXKey))
                        .map(|x| x.maximum());
                }

                // Paper z.7
                match third_level.unwrap().get() {
                    PointerEnum::First(l) => {
                        if l.lx_top.is_set(k as usize) {
                            return Some(*l.get(k));
                        } else {
                            // Paper z.8
                            let new_k = (*l).lx_top.get_prev_set_bit(k as usize);
                            return new_k
                                .map(|x| *(*l).try_get(x as LXKey).unwrap());
                        }

                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(*e);
                    }
                }
        
                
                
            },

            PointerEnum::Second(e) => {
                return Some(*e);
            }
        }


    }

    /// Diese Methode gibt den Index INDEX des kleinsten Elements zurück für das gilt element<=element_list[INDEX].
    /// Somit kann mit dieser Methode geprüft werden, ob ein Element in der Datenstruktur enthalten ist. Dann wird der Index dieses Elements zurückgegeben.
    /// Ist das Element nicht enthalten, wird der "Nachfolger" dieses Elements zurückgegeben.
    /// 
    /// # Arguments
    ///
    /// * `element` - Evtl. in der Datenstruktur enthaltener Wert, dessen Index zurückgegeben wird. Anderenfalls wird der Index des Nachfolgers von element zurückgegeben.
    #[inline]
    pub fn locate_or_succ(&self, element: T) -> Option<usize> {
        // Paper z.1 
        if element > self.maximum().unwrap() {
            return None;
        } 

        let (i,j,k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || self.element_list[self.root_table[i].maximum()] < element {
            return self.root_top.get_next_set_bit(i as usize)
                .map(|x| self.root_table[x].minimum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none() || self.element_list[third_level.unwrap().maximum()] < element {
                    let new_j = second_level.lx_top.get_next_set_bit(j as usize);
                    return new_j
                        .and_then(|x| second_level.try_get(x as LXKey))
                        .map(|x| x.minimum());
                }

                // Paper z.7
                match third_level.unwrap().get() {
                    PointerEnum::First(l) => {
                         
                        if l.lx_top.is_set(k as usize) {
                            return Some(*l.get(k));
                        } else {
                            // Paper z.8
                            let new_k = (*l).lx_top.get_next_set_bit(k as usize);
                            return new_k
                                .map(|x| *(*l).try_get(x as LXKey).unwrap());
                        };
                                        
  
                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(*e);
                    }
                }
        
                
                
            },

            PointerEnum::Second(e) => {
                return Some(*e);
            }
        }
        
    }
}

#[allow(dead_code)]
/// Diese Datenstruktur dient als naive Hashmap. Sie speichert eine Lookuptable und die Daten
pub struct LookupTableSmall<E> {
    /// (ehemaliges Array mit len= objects.len())
    table: *mut u8,
    objects: Box<[E]>,
}

impl<E> Drop for LookupTableSmall<E> {
    fn drop(&mut self) {
        unsafe {
            let mut len = 0_usize;
            let max_index = self.objects.len()-1;
            let mut curr_value = self.table;
            while *curr_value as usize != max_index {
                curr_value = curr_value.offset(1);
                len += 1;
            }
            Box::from_raw(std::slice::from_raw_parts_mut(self.table, len));
        }
    }
}

impl<E: Clone> Clone for LookupTableSmall<E> {
    fn clone(&self) -> Self {
        let mut new_lookup = vec![];
        let max_index = self.objects.len()-1;
        let mut curr_value = self.table;
        unsafe {
            while *curr_value as usize != max_index {
                new_lookup.push(*curr_value);
                curr_value = curr_value.offset(1);
            }
        }

        Self {
            table: Box::into_raw(new_lookup.into_boxed_slice()) as *mut u8,
            objects: self.objects.clone()
        }
    }
}

#[allow(dead_code)]
impl<E> LookupTableSmall<E> {
    
    /// Vorbindung: keys sind sortiert. Weiterhin gilt keys.len() == objects.len() und  keys.len() > 0
    /// Nachbedingung : keys[i] -> objects[i]
    pub fn new(keys: &[u8], objects: Box<[E]>) -> Self {
        debug_assert!(keys.len() == objects.len());

        // benötigt die Eigenschaft, dass die keys sortiert sind
        let mut lookup_table = vec![0_u8;keys[keys.len()-1] as usize + 1];
        for (i,&k) in keys.into_iter().enumerate() {
            lookup_table[k as usize] = i as u8;
        }
        Self {
            table: Box::into_raw(lookup_table.into_boxed_slice()) as *mut u8,
            objects: objects,
        }
    }

    pub fn get(&self, key: &u8) -> &E {
        unsafe {
            self.objects.get_unchecked(*self.table.offset(*key as isize) as usize)
        }
    }

    pub fn get_mut(&mut self, key: &u8) -> &mut E {
        unsafe {
            self.objects.get_unchecked_mut(*self.table.offset(*key as isize) as usize)
        }
    }
}

/// Diese Datenstruktur dient als naive Hashmap. Sie speichert eine Lookuptable und die Daten
pub struct LookupTable<E> {
    /// (ehemaliges Array mit len= objects.len())
    table: *mut u16,
    objects: Box<[E]>,
}

impl<E> Drop for LookupTable<E> {
    fn drop(&mut self) {
        unsafe {
            let mut len = 0_usize;
            let max_index = self.objects.len()-1;
            let mut curr_value = self.table;
            while *curr_value as usize != max_index {
                curr_value = curr_value.offset(1);
                len += 1;
            }
            Box::from_raw(std::slice::from_raw_parts_mut(self.table, len));
        }
    }
}


impl<E: Clone> Clone for LookupTable<E> {
    fn clone(&self) -> Self {
        let mut new_lookup = vec![];
        let max_index = self.objects.len()-1;
        let mut curr_value = self.table;
        unsafe {
            while *curr_value as usize != max_index {
                new_lookup.push(*curr_value);
                curr_value = curr_value.offset(1);
            }
        }

        Self {
            table: Box::into_raw(new_lookup.into_boxed_slice()) as *mut u16,
            objects: self.objects.clone()
        }
    }
}


impl<E> LookupTable<E> {
    /// Vorbindung: keys sind sortiert. Weiterhin gilt keys.len() == objects.len() und  keys.len() > 0
    /// Nachbedingung : keys[i] -> objects[i]
    pub fn new(keys: &[u16], objects: Box<[E]>) -> Self {

        // benötigt die Eigenschaft, dass die keys sortiert sind
        let mut lookup_table = vec![0_u16;keys[keys.len()-1] as usize + 1];
        for (i,&k) in keys.into_iter().enumerate() {
            lookup_table[k as usize] = i as u16;
        }
        Self {
            table: Box::into_raw(lookup_table.into_boxed_slice()) as *mut u16,
            objects: objects,
        }
    }

    pub fn get(&self, key: &u16) -> &E {
        unsafe {
            self.objects.get_unchecked(*self.table.offset(*key as isize) as usize)
        }
    }

    pub fn get_mut(&mut self, key: &u16) -> &mut E {
        unsafe {
            self.objects.get_unchecked_mut(*self.table.offset(*key as isize) as usize)
        }
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays. 
#[derive(Clone)]
#[repr(align(4))]
pub struct Level<T, E> {
    /// Perfekte Hashmap, die immer (außer zur Inialisierung) gesetzt ist. 
    hash_map: LookupTable<T>,

    /// Speichert einen Zeiger auf den Index des Maximum dieses Levels
    pub maximum: usize,

    /// Speichert einen Zeiger auf den Index des Minimums dieses Levels
    pub minimum: usize,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64]. 
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    lx_top: TopArray<E,u16>,
}

impl<T,E> Level<T,E> {
    /// Gibt ein Level<T> mit Origin-Key j zurück. Optional kann eine Schlüsselliste übergeben werden, für welche dann
    /// eine perfekte Hashfunktion generiert wird.
    ///
    /// # Arguments
    ///
    /// * `j` - Falls eine andere Ebene auf diese mittels Hashfunktion zeigt, muss der verwendete key gespeichert werden. 
    /// * `keys` - Eine Liste mit allen Schlüsseln, die mittels perfekter Hashfunktion auf die nächste Ebene zeigen.
    #[inline]
    pub fn new(lx_top: TopArray<E, u16>, objects: Box<[T]>, keys: Box<[LXKey]>, minimum: usize, maximum: usize) -> Level<T,E> {
        Level {
            hash_map: LookupTable::new(&keys, objects),
            minimum: minimum,
            maximum: maximum,
            lx_top: lx_top,
        } 
    }

    /// Mit Hilfe dieser Funktion kann die perfekte Hashfunktion verwendet werden. 
    /// Es muss beachtet werden, dass sichergestellt werden muss, dass der verwendete Key auch existiert!
    /// 
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn try_get(&self, key: LXKey) -> Option<&T> {
        if self.lx_top.is_set(key as usize) {
            Some(self.hash_map.get(&key))
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
    pub fn get(&mut self, key: LXKey) -> &mut T {
        self.hash_map.get_mut(&key)
    }
}

#[cfg(test)]
mod tests {
    use uint::{u40,u48};
    use super::{STree};
    use crate::internal::{PointerEnum,Splittable};

    /// Größe der LX-Top-Arrays 40 Bit
    const LX_ARRAY_SIZE_U40: usize = 1 << 10;

    /// Größe der LX-Top-Arrays 48 Bit
    const LX_ARRAY_SIZE_U48: usize = 1 << 12;

    // u64 Tests werden ausgespart, da der STree (leer) nach Initialisierung 2^32 * 8 Byte = 34 Gbyte RAM benötigt
    // Diese Tests sind nicht auf gängigen Laptop ausführbar. (Zukunft, ich rede von 2019 :p).

    /// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
    #[test]
    fn test_u40_new_hashfunctions() {

        // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
        // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
        let mut data: Vec<u40> = vec![u40::new(0);LX_ARRAY_SIZE_U40];
        
        for i in 0..data.len() {
            data[i] = u40::new(i as u64);
        }
 
        let check = data.clone();
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u40::new(0));
        assert_eq!(data_structure.maximum().unwrap(),u40::new(check.len() as u64 - 1));
        for val in check {
            let (i,j,k) = Splittable::split_integer_down(&val);

            match data_structure.root_table[i].get() {
                PointerEnum::First(l) => {
                    let second_level = l.get(j);
                    let saved_val = match second_level.get() {
                        PointerEnum::First(l) => {
                            *(*l).get(k)
                        },
                        PointerEnum::Second(e) => {
                            *e
                        }
                    };
                    assert_eq!(data_structure.element_list[saved_val],val);
                },

                PointerEnum::Second(e) => {
                    assert_eq!(data_structure.element_list[*e],val);
                }
            };

        }
    }


    /// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
    #[test]
    fn test_u48_new_hashfunctions() {
        // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
        // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
        let mut data: Vec<u48> = vec![u48::new(0);LX_ARRAY_SIZE_U48];
        
        for i in 0..data.len() {
            data[i] = u48::new(i as u64);
        }
 
        let check = data.clone();
        let data_structure: STree<u48> = STree::new(data.into_boxed_slice());

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u48::new(0));
        assert_eq!(data_structure.maximum().unwrap(),u48::new(check.len() as u64 - 1));
        for val in check {
            let (i,j,k) = Splittable::split_integer_down(&val);
            match data_structure.root_table[i].get() {
                PointerEnum::First(l) => {
                    let second_level = l.get(j);
                    let saved_val = match second_level.get() {
                        PointerEnum::First(l) => {
                            *(*l).get(k)
                        },
                        PointerEnum::Second(e) => {
                            *e
                        }
                    };
                    assert_eq!(data_structure.element_list[saved_val],val);
                },

                PointerEnum::Second(e) => {
                    assert_eq!(data_structure.element_list[*e],val);
                }
            };

        }
    }
    
    /// Die Top-Arrays werden geprüft. Dabei wird nur grob überprüft, ob sinnvolle Werte gesetzt wurden.
    /// Dieser Test ist ein Kandidat zum Entfernen oder Erweitern.
    #[test]
    fn test_u40_top_arrays() {
        let data: Vec<u40> = vec![u40::new(0b00000000000000000000_1010010010_0101010101),u40::new(0b00000000000000000000_1010010010_0101010111),u40::new(0b11111111111111111111_1010010010_0101010101_u64)];
        let check = data.clone();
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u40::new(0b00000000000000000000_1010010010_0101010101));
        assert_eq!(data_structure.maximum().unwrap(),u40::new(0b11111111111111111111_1010010010_0101010101_u64));

        for val in check {
            let (i,j,k) = Splittable::split_integer_down(&val);
            if data_structure.root_table[i].minimum() != data_structure.root_table[i].maximum() {
                let second_level = match data_structure.root_table[i].get() {
                        PointerEnum::First(l) => {
                            l.get(j)
                        },
                        _ => {
                            panic!("Das sollte nicht geschehen");
                        }
                };
                if second_level.minimum() != second_level.maximum() {
                    let saved_val = match second_level.get() {
                        PointerEnum::First(l) => {
                            l.get(k)
                        },
                        _ => {
                            panic!("Das sollte nicht geschehen");
                        }
                    };
                    assert_eq!(data_structure.element_list[*saved_val],val);
                } else {
                    assert_eq!(data_structure.element_list[second_level.minimum()],val);
                }

            } else {
                assert_eq!(data_structure.element_list[data_structure.root_table[i].minimum()],val);
            }

        }
        
    }

    /// Die Top-Arrays werden geprüft. Dabei wird nur grob überprüft, ob sinnvolle Werte gesetzt wurden.
    /// Dieser Test ist ein Kandidat zum Entfernen oder Erweitern.
    #[test]
    fn test_u48_top_arrays() {
        let data: Vec<u48> = vec![u48::new(0b10010010_00000000000000000000_1010010010_0101010101_u64),u48::new(0b10010010_00000000000000000000_1010010010_0101010111_u64),u48::new(0b11111111_11111111111111111111_1010010010_0101010101_u64)];
        let check = data.clone();
        let data_structure: STree<u48> = STree::new(data.into_boxed_slice());

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u48::new(0b10010010_00000000000000000000_1010010010_0101010101_u64));
        assert_eq!(data_structure.maximum().unwrap(),u48::new(0b11111111_11111111111111111111_1010010010_0101010101_u64));

        for val in check {
            let (i,j,k) = Splittable::split_integer_down(&val);
            if data_structure.root_table[i].minimum() != data_structure.root_table[i].maximum() {
                let second_level = match data_structure.root_table[i].get() {
                        PointerEnum::First(l) => {
                            l.get(j)
                        },
                        _ => {
                            panic!("Das sollte nicht geschehen");
                        }
                };
                if second_level.minimum() != second_level.maximum() {
                    let saved_val = match second_level.get() {
                        PointerEnum::First(l) => {
                            l.get(k)
                        },
                        _ => {
                            panic!("Das sollte nicht geschehen");
                        }
                    };
                    assert_eq!(data_structure.element_list[*saved_val],val);
                } else {
                    assert_eq!(data_structure.element_list[second_level.minimum()],val);
                }

            } else {
                assert_eq!(data_structure.element_list[data_structure.root_table[i].minimum()],val);
            }

        }

        
    }


    /// Die locate_or_succ-Funktion wird getestet. Dabei werden beliebige Werte in ein STree gegeben und anschließend wird
    /// `locate_or_succ(x) mit allen x zwischen STree.min() und STree.max() getestet.
    #[test]
    fn test_u40_locate_or_succ_bruteforce() {
        let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
        let mut data: Vec<u40> = vec![];
        for val in data_v1.iter() {
            data.push(u40::new(*val));
        }
        
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());
        for (index,_) in data_v1.iter().enumerate() {
            if index < data_v1.len()-1 {
                for i in data_v1[index]+1..data_v1[index+1]+1 {
                    let locate = data_structure.locate_or_succ(u40::new(i)).unwrap();
                    assert_eq!(data_structure.element_list[locate], u40::new(data_v1[index+1]));
                }
            }
        }
    }

    /// Die locate_or_succ-Funktion wird getestet. Dabei werden beliebige Werte in ein STree gegeben und anschließend wird
    /// `locate_or_succ(x) mit allen x zwischen STree.min() und STree.max() getestet.
    #[test]
    fn test_u48_locate_or_succ_bruteforce() {
        let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983,1865983];
        let mut data: Vec<u48> = vec![];
        for val in data_v1.iter() {
            data.push(u48::new(*val));
        }
        
        let data_structure: STree<u48> = STree::new(data.into_boxed_slice());
        for (index,_) in data_v1.iter().enumerate() {
            if index < data_v1.len()-1 {
                for i in data_v1[index]+1..data_v1[index+1]+1 {
                    let locate = data_structure.locate_or_succ(u48::new(i)).unwrap();
                    assert_eq!(data_structure.element_list[locate], u48::new(data_v1[index+1]));
                }
            }
        }
    }

    /// # Äquivalenzklassentest mit Bruteforce
    /// `locate_or_succ` wird getestet. Dabei werden in jeder Ebene die gesuchten Elemente einmal im Minimum, im Maximum und irgendwo dazwischen liegen.
    #[test]
    fn test_u40_locate_or_succ_eqc_bruteforce_test() {
        let data_raw: Vec<u64> = vec![
            0b00000000000000000000_0000000000_0000000001,
            0b00000000000000000000_0000000000_0000111000,
            0b00000000000000000000_0000000000_1111111111,

            0b00000000000000000000_0001110000_0000000000,
            0b00000000000000000000_0001110000_0000111000,
            0b00000000000000000000_0001110000_1111111111,

            0b00000000000000000000_1111111111_0000000000,
            0b00000000000000000000_1111111111_0000111000,
            0b00000000000000000000_1111111111_1111111111,

            0b00000000001111000000_0000000000_0000000000,
            0b00000000001111000000_0000000000_0000111000,
            0b00000000001111000000_0000000000_1111111111,

            0b00000000001111000000_0001110000_0000000000,
            0b00000000001111000000_0001110000_0000111000,
            0b00000000001111000000_0001110000_1111111111,

            0b00000000001111000000_1111111111_0000000000,
            0b00000000001111000000_1111111111_0000111000,
            0b00000000001111000000_1111111111_1111111111,

            0b11111111111111111111_0000000000_0000000000,
            0b11111111111111111111_0000000000_0000111000,
            0b11111111111111111111_0000000000_1111111111,

            0b11111111111111111111_0001110000_0000000000,
            0b11111111111111111111_0001110000_0000111000,
            0b11111111111111111111_0001110000_1111111111,

            0b11111111111111111111_1111111111_0000000000,
            0b11111111111111111111_1111111111_0000111000,
            0b11111111111111111111_1111111111_1111111110,
            
        ];

        let mut data: Vec<u40> = vec![];
        for val in data_raw.iter() {
            data.push(u40::new(*val));
        }
        let data_structure: STree<u40> = STree::new(data.clone().into_boxed_slice());
        assert_eq!(data_structure.locate_or_succ(u40::new(0b11111111111111111111_1111111111_1111111111_u64)), None);
        
        for (i,&elem) in data.iter().enumerate() {
            if i > 0 {
                for j in 0..16877216 {
                    if u64::from(elem)>=j as u64 {
                        let index = elem - u40::new(j);
                        if index > data_structure.element_list[i-1] {
                            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(index).unwrap() as usize], elem);
                        }
                    }
                }
            } else {
                assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem).unwrap() as usize], elem);
                assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem-u40::new(1)).unwrap() as usize], elem);
            }
        }
    }

    #[test]
    fn test_u48_locate_or_succ_eqc_bruteforce_test() {
        let data_raw: Vec<u64> = vec![
            0b000000000000000000000000_000000000000_000000000001,
            0b000000000000000000000000_000000000000_000001110000,
            0b000000000000000000000000_000000000000_111111111111,

            0b000000000000000000000000_000001110000_000000000000,
            0b000000000000000000000000_000001110000_000001110000,
            0b000000000000000000000000_000001110000_111111111111,

            0b000000000000000000000000_111111111111_000000000001,
            0b000000000000000000000000_111111111111_000001110000,
            0b000000000000000000000000_111111111111_111111111111,

            0b000000001100000000000011_000000000000_000000000001,
            0b000000001100000000000011_000000000000_000001110000,
            0b000000001100000000000011_000000000000_111111111111,

            0b000000001100000000000011_000001110000_000000000000,
            0b000000001100000000000011_000001110000_000001110000,
            0b000000001100000000000011_000001110000_111111111111,

            0b000000001100000000000011_111111111111_000000000001,
            0b000000001100000000000011_111111111111_000001110000,
            0b000000001100000000000011_111111111111_111111111111,


            0b111111111111111111111111_000000000000_000000000001,
            0b111111111111111111111111_000000000000_000001110000,
            0b111111111111111111111111_000000000000_111111111111,

            0b111111111111111111111111_000001110000_000000000000,
            0b111111111111111111111111_000001110000_000001110000,
            0b111111111111111111111111_000001110000_111111111111,

            0b111111111111111111111111_111111111111_000000000001,
            0b111111111111111111111111_111111111111_000001110000,
            0b111111111111111111111111_111111111111_111111111110,
        ];

        let mut data: Vec<u48> = vec![];
        for val in data_raw.iter() {
            data.push(u48::new(*val));
        }
        let data_structure: STree<u48> = STree::new(data.clone().into_boxed_slice());
        assert_eq!(data_structure.locate_or_succ(u48::new(0b111111111111111111111111_111111111111_111111111111_u64)), None);
        
        for (i,&elem) in data.iter().enumerate() {
            if i > 0 {
                for j in 0..16877216 {
                    if u64::from(elem)>=j as u64 {
                        let index = elem - u48::new(j);
                        if index > data_structure.element_list[i-1] {
                            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(index).unwrap() as usize], elem);
                        }
                    }
                }
            } else {
                assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem).unwrap() as usize], elem);
                assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem-u48::new(1)).unwrap() as usize], elem);
            }
        }
    }

    /// Die locate_or_pred-Funktion wird getestet. Dabei werden beliebige (fest gewählte) Werte in ein STree gegeben und anschließend wird
    /// `locate_or_pred(x) mit allen x zwischen STree.min() und STree.max() getestet.
    #[test]
    fn test_u40_locate_or_pred_bruteforce() {
        let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
        let mut data: Vec<u40> = vec![];
        for val in data_v1.iter() {
            data.push(u40::new(*val));
        }
        
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());
        assert_eq!(u40::new(1065983), data_structure.element_list[data_structure.locate_or_pred(u40::new(1065983)).unwrap()]);
        for (index,_) in data_v1.iter().enumerate().rev() {
            if index > 0 {
                for i in (data_v1[index-1]..data_v1[index]).rev() {
                    let locate = data_structure.locate_or_pred(u40::new(i)).unwrap();
                    assert_eq!(u40::new(data_v1[index-1]), data_structure.element_list[locate]);
                }
            }
        }
    }

        /// Die locate_or_pred-Funktion wird getestet. Dabei werden beliebige (fest gewählte) Werte in ein STree gegeben und anschließend wird
    /// `locate_or_pred(x) mit allen x zwischen STree.min() und STree.max() getestet.
    #[test]
    fn test_u48_locate_or_pred_bruteforce() {
        let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
        let mut data: Vec<u48> = vec![];
        for val in data_v1.iter() {
            data.push(u48::new(*val));
        }
        
        let data_structure: STree<u48> = STree::new(data.into_boxed_slice());
        assert_eq!(u48::new(1065983), data_structure.element_list[data_structure.locate_or_pred(u48::new(1065983)).unwrap()]);
        for (index,_) in data_v1.iter().enumerate().rev() {
            if index > 0 {
                for i in (data_v1[index-1]..data_v1[index]).rev() {
                    let locate = data_structure.locate_or_pred(u48::new(i)).unwrap();
                    assert_eq!(u48::new(data_v1[index-1]), data_structure.element_list[locate]);
                }
            }
        }
    }

    use num::Bounded;
     /// # Äquivalenzklassentest mit Bruteforce
    /// `locate_or_pred` wird getestet. Dabei werden in jeder Ebene die gesuchten Elemente einmal im Minimum, im Maximum und irgendwo dazwischen liegen.
    #[test]
    fn test_u40_locate_or_pred_eqc_bruteforce_test() {
        let data_raw: Vec<u64> = vec![
            0b00000000000000000000_0000000000_0000000001,
            0b00000000000000000000_0000000000_0000111000,
            0b00000000000000000000_0000000000_1111111111,

            0b00000000000000000000_0001110000_0000000000,
            0b00000000000000000000_0001110000_0000111000,
            0b00000000000000000000_0001110000_1111111111,

            0b00000000000000000000_1111111111_0000000000,
            0b00000000000000000000_1111111111_0000111000,
            0b00000000000000000000_1111111111_1111111111,

            0b00000000001111000000_0000000000_0000000000,
            0b00000000001111000000_0000000000_0000111000,
            0b00000000001111000000_0000000000_1111111111,

            0b00000000001111000000_0001110000_0000000000,
            0b00000000001111000000_0001110000_0000111000,
            0b00000000001111000000_0001110000_1111111111,

            0b00000000001111000000_1111111111_0000000000,
            0b00000000001111000000_1111111111_0000111000,
            0b00000000001111000000_1111111111_1111111111,

            0b11111111111111111111_0000000000_0000000000,
            0b11111111111111111111_0000000000_0000111000,
            0b11111111111111111111_0000000000_1111111111,

            0b11111111111111111111_0001110000_0000000000,
            0b11111111111111111111_0001110000_0000111000,
            0b11111111111111111111_0001110000_1111111111,

            0b11111111111111111111_1111111111_0000000000,
            0b11111111111111111111_1111111111_0000111000,
            0b11111111111111111111_1111111111_1111111110,
            
        ];

        let mut data: Vec<u40> = vec![];
        for val in data_raw.iter() {
            data.push(u40::new(*val));
        }
        let data_structure: STree<u40> = STree::new(data.clone().into_boxed_slice());
        assert_eq!(data_structure.locate_or_pred(u40::new(0)), None);

        for (i,&elem) in data.iter().enumerate().rev() {
            if i < data.len()-1 {
                for j in 0..16877216 {
                    if u40::max_value() > elem && u40::new(j) < u40::max_value() - elem {
                        let index = elem + u40::new(j);
                        if index < data_structure.element_list[i+1] {
                            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(index).unwrap() as usize], elem);
                        }
                    }
                }
            } else {
                assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem).unwrap() as usize], elem);
                assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem+u40::new(1)).unwrap() as usize], elem);
            }
        }
    }

#[test]
    fn test_u48_locate_or_pred_eqc_bruteforce_test() {
        let data_raw: Vec<u64> = vec![
            0b00000000000000000000_0000000000_0000000001,
            0b00000000000000000000_0000000000_0000111000,
            0b00000000000000000000_0000000000_1111111111,

            0b00000000000000000000_0001110000_0000000000,
            0b00000000000000000000_0001110000_0000111000,
            0b00000000000000000000_0001110000_1111111111,

            0b00000000000000000000_1111111111_0000000000,
            0b00000000000000000000_1111111111_0000111000,
            0b00000000000000000000_1111111111_1111111111,

            0b00000000001111000000_0000000000_0000000000,
            0b00000000001111000000_0000000000_0000111000,
            0b00000000001111000000_0000000000_1111111111,

            0b00000000001111000000_0001110000_0000000000,
            0b00000000001111000000_0001110000_0000111000,
            0b00000000001111000000_0001110000_1111111111,

            0b00000000001111000000_1111111111_0000000000,
            0b00000000001111000000_1111111111_0000111000,
            0b00000000001111000000_1111111111_1111111111,

            0b11111111111111111111_0000000000_0000000000,
            0b11111111111111111111_0000000000_0000111000,
            0b11111111111111111111_0000000000_1111111111,

            0b11111111111111111111_0001110000_0000000000,
            0b11111111111111111111_0001110000_0000111000,
            0b11111111111111111111_0001110000_1111111111,

            0b11111111111111111111_1111111111_0000000000,
            0b11111111111111111111_1111111111_0000111000,
            0b11111111111111111111_1111111111_1111111110,
        ];

        let mut data: Vec<u48> = vec![];
        for val in data_raw.iter() {
            data.push(u48::new(*val));
        }
        let data_structure: STree<u48> = STree::new(data.clone().into_boxed_slice());
        assert_eq!(data_structure.locate_or_pred(u48::new(0)), None);

        for (i,&elem) in data.iter().enumerate().rev() {
            if i < data.len()-1 {
                for j in 0..16877216 {
                    if u48::max_value() > elem && u48::new(j) < u48::max_value() - elem {
                        let index = elem + u48::new(j);
                        if index < data_structure.element_list[i+1] {
                            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(index).unwrap() as usize], elem);
                        }
                    }
                }
            } else {
                assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem).unwrap() as usize], elem);
                assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem+u48::new(1)).unwrap() as usize], elem);
            }
        }
    }

    use rand_distr::{Distribution, Uniform};
    use crate::default::immutable::TopArray;
    /*#[test]*/
    /// Fügt einige Bits in eine ArrayTop-Struktur und prüft anschließend, ob die Bits gesetted sind.
    /// (Deaktiviert, da der Test sehr lange dauert)
    fn test_top_array_set_bit() {
        let between = Uniform::from(0u64..(1<<10));
        let mut rng = rand::thread_rng();

        let mut bits_set: Vec<usize> = vec![];
        for _ in 0..230 {
            bits_set.push(between.sample(&mut rng) as usize);
        }
        bits_set.sort();
        bits_set.dedup();

        let mut lxtop = TopArray::<u40, u16>::new();

        for &i in bits_set.iter() {
            lxtop.set_bit(i);
        }

        for i in 0..(1<<10) {
            assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
        }

        for i in 0..bits_set.len()-1 {
            assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
        }

        for i in 1..bits_set.len() {
            assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
        }

        let between = Uniform::from(0u64..(1<<12));
        let mut bits_set: Vec<usize> = vec![];
        for _ in 0..230 {
            bits_set.push(between.sample(&mut rng) as usize);
        }
        bits_set.sort();
        bits_set.dedup();
        
        let mut lxtop = TopArray::<u48, u16>::new();
        
        for &i in bits_set.iter() {
            lxtop.set_bit(i);
        }

        for i in 0..(1<<12) {
            assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
        }

        for i in 0..bits_set.len()-1 {
            assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
        }

        for i in 1..bits_set.len() {
            assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
        } 

        let between = Uniform::from(0u64..(1<<16));
        let mut bits_set: Vec<usize> = vec![];
        for _ in 0..20000 {
            bits_set.push(between.sample(&mut rng) as usize);
        }
        bits_set.sort();
        bits_set.dedup();
        let mut lxtop = TopArray::<u64, u16>::new();

        for &i in bits_set.iter() {
            lxtop.set_bit(i);
        }

        for i in 0..(1<<16) {
            assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
        }

        for i in 0..bits_set.len()-1 {
            assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
        }

        for i in 1..bits_set.len() {
            assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
        }
        
        let between = Uniform::from(0u64..(1<<20));
        let mut bits_set: Vec<usize> = vec![];
        for _ in 0..20000 {
            bits_set.push(between.sample(&mut rng) as usize);
        }
        bits_set.sort();
        bits_set.dedup();
        let mut lxtop = TopArray::<u40, usize>::new();

        for &i in bits_set.iter() {
            lxtop.set_bit(i);
        }

        for i in 0..(1<<20) {
            assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
        }

        for i in 0..bits_set.len()-1 {
            assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
        }

        for i in 1..bits_set.len() {
            assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
        }

        let between = Uniform::from(0u64..(1<<22));
        let mut bits_set: Vec<usize> = vec![];
        for _ in 0..20000 {
            bits_set.push(between.sample(&mut rng) as usize);
        }
        bits_set.sort();
        bits_set.dedup();
        let mut lxtop = TopArray::<u48, usize>::new();

        for &i in bits_set.iter() {
            lxtop.set_bit(i);
        }

        for i in 0..(1<<22) {
            assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
        }

        for i in 0..bits_set.len()-1 {
            assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
        }

        for i in 1..bits_set.len() {
            assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
        }

        let between = Uniform::from(0u64..(1<<32));
        let mut bits_set: Vec<usize> = vec![];
        for _ in 0..20000 {
            bits_set.push(between.sample(&mut rng) as usize);
        }
        println!("Did it");
        bits_set.sort();
        bits_set.dedup();
        println!("Did sort");
        let mut lxtop = TopArray::<u64, usize>::new();

        for &i in bits_set.iter() {
            lxtop.set_bit(i);
        }
        println!("Did set");

        // Abgespeckt da das verdammt lange dauert!
        for i in 0..(1<<22) {
            assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
        }

        for i in 0..bits_set.len()-1 {
            assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
        }

        for i in 1..bits_set.len() {
            assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
        }
    }
}
use crate::default::build::STreeBuilder;

use crate::internal::{MphfHashMapThres, Splittable};
pub type L1Ebene<T> = LevelPointer<L2Ebene<T>, T>;
/// Die L2-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf eine
/// L3-Ebene zeigt.
pub type L2Ebene<T> = LevelPointer<L3Ebene<T>, T>;

/// Die L3-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf
/// ein Indize der STree.element_list zeigt.
pub type L3Ebene<T> = LevelPointer<*const T, T>;

use crate::internal::{self, PointerEnum};

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein usize-Objekt zeigt (Index aus Elementliste),
/// oder auf ein Levelobjekt
#[derive(Clone)]
pub struct LevelPointer<T, E> {
    pointer: internal::Pointer<Level<T, E>, E>,
}

impl<T, E> LevelPointer<T, E> {
    pub fn minimum(&self) -> &E {
        match self.pointer.get() {
            PointerEnum::First(l) => unsafe { &*l.minimum },

            PointerEnum::Second(e) => &*e,
        }
    }

    pub fn maximum(&self) -> &E {
        match self.pointer.get() {
            PointerEnum::First(l) => unsafe { &*l.maximum },

            PointerEnum::Second(e) => &*e,
        }
    }

    pub fn from_level(level_box: Box<Level<T, E>>) -> Self {
        Self {
            pointer: internal::Pointer::from_first(level_box),
        }
    }

    pub fn get(&self) -> PointerEnum<Level<T, E>, E> {
        self.pointer.get()
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    pub fn from_null() -> Self {
        Self {
            pointer: internal::Pointer::null(),
        }
    }

    pub fn from_usize(usize_box: *const E) -> Self {
        Self {
            pointer: internal::Pointer::from_second(usize_box),
        }
    }

    pub fn change_to_usize(&mut self, usize_box: *const E) {
        self.pointer = internal::Pointer::from_second(usize_box);
    }
}

/// Statische Predecessor-Datenstruktur. Sie verwendet perfektes Hashing und ein Array auf der Element-Listen-Ebene.
/// Sie kann nur sortierte und einmalige Elemente entgegennehmen.
#[derive(Clone)]
pub struct STree<T> {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2-Ebene je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    pub root_table: Box<[L1Ebene<T>]>,

    /// Das Root-Top-Array speichert für jeden Eintrag `root_table[i][x]`, der belegt ist, ein 1-Bit, sonst einen 0-Bit.
    /// Auch hier werden nicht 2^20 Einträge, sondern lediglich [u64;2^20/64] gespeichert.
    /// i steht dabei für die Ebene der root_tabelle. Ebene i+1 beinhaltet an Index [x] immer 64 Veroderungen aus Ebene i.
    /// Somit gilt |root_table[i+1]| = |root_table[i]|/64  
    pub root_top: TopArray<T, usize>,

    /// Die Elementliste beinhaltet einen Vektor konstanter Länge mit jeweils allen gespeicherten Elementen in sortierter Reihenfolge.
    pub element_list: Box<[T]>,
}

/// Liste von Bitarrays zur Speicherung der LX-Top-Datenstrukturen
/// Zur Speicherplatzreduzierung werden die Längen der Arrays weggeworfen und zum Drop-Zeitpunkt erneut berechnet
pub struct TopArray<T, V> {
    /// 2-dimensionales Array mit
    data: Box<[*mut u64]>,

    // Länge der untersten Ebene. Kleiner Tradeoff zwischen Länge aller Ebenen Speichern und Level der tiefsten Ebene Speichern...
    lowest_len: usize,

    /// entspricht dem Nutzdatentyp (u40,u48 oder u64)
    phantom: std::marker::PhantomData<T>,

    /// Entspricht dem Key der gehasht wird. Wenn Wurzeltop, dann usize, wenn LXTop dann LXKey
    phantom_type: std::marker::PhantomData<V>,
}

impl<T, V> Drop for TopArray<T, V> {
    fn drop(&mut self) {
        // Hier muss editiert werden, wenn die Größen der L2- und L3- Level angepasst werden sollen
        let mut length = Self::get_length();

        // Solange Länge / 2^i > 256
        for (i, &ptr) in self.data.into_iter().enumerate() {
            length = length >> (i + 1) * 6;
            unsafe {
                Box::from_raw(std::slice::from_raw_parts_mut(ptr, length));
            }
        }
    }
}

impl<T, V> Clone for TopArray<T, V> {
    fn clone(&self) -> Self {
        // Hier muss editiert werden, wenn die Größen der L2- und L3- Level angepasst werden sollen
        let mut length = Self::get_length();

        // Lege alle Rootarrays an
        let mut top_arrays = vec![];

        // Solange Länge / 2^i > 256
        for (i, &ptr) in self.data.iter().enumerate() {
            length = length >> (i + 1) * 6;
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

impl<T, V> TopArray<T, V> {
    #[inline]
    fn get_length() -> usize {
        if std::mem::size_of::<V>() == std::mem::size_of::<usize>() {
            1 << 22
        } else if std::mem::size_of::<V>() == std::mem::size_of::<LXKey>() {
            1 << 14
        } else {
            panic!("Ungültige Parameterkombination vom TopArray!")
        }
    }
}

impl<T, V> TopArray<T, V> {
    /// Erzeugt mehrere Ebenen für einen Bitvector der Länge length
    #[inline]
    pub fn new() -> Self {
        // Hier muss editiert werden, wenn die Größen der L2- und L3- Level angepasst werden sollen
        let mut length = Self::get_length();

        // Lege alle Rootarrays an
        let mut top_arrays = vec![];
        // Solange Länge / 64^i > 64
        while length >= 64 {
            length = length >> 6;
            top_arrays.push(Box::into_raw(vec![0_u64; length].into_boxed_slice()) as *mut u64);
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
        1 << 63 - in_index
    }

    /// Baut das Root-Top-Array mit Hilfe der sich in der Datenstruktur befindenden Werte.
    #[inline]
    pub fn set_bit(&mut self, bit: usize) {
        let mut index = bit / 64;
        let mut in_index = bit % 64;

        // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
        for i in 0..(self.data.len()) {
            // Aktueller in_index wird für Bitmaske verwendet
            let bit_mask = Self::get_bit_mask(in_index);
            let bit_window = unsafe { self.data.get_unchecked(i).add(index) };

            in_index = index % 64;
            index = index / 64;

            unsafe {
                *bit_window = *bit_window | bit_mask;
            }
        }
    }

    #[inline]
    pub fn is_set(&self, bit: usize) -> bool {
        let (index, in_index) = (bit / 64, bit % 64);
        let bit_mask = Self::get_bit_mask(in_index);
        let bit_window = unsafe { self.data.get_unchecked(0).add(index) };

        unsafe { *bit_window & bit_mask != 0 }
    }

    #[inline]
    fn get_next_set_bit_translation(&self, index: usize, last_level: usize) -> usize {
        let mut index = index;
        for i in (0..(last_level)).rev() {
            let zeros_to_bit = unsafe { *self.data.get_unchecked(i).add(index) };
            index = index * 64 + zeros_to_bit.leading_zeros() as usize;
        }
        index
    }

    /// Diese Funktion as nächste Bit zurück, dass hinter `bit` gesetzt ist.
    #[inline]
    pub fn get_next_set_bit(&self, bit: usize) -> Option<usize> {
        let mut index = bit / 64;
        let mut in_index = bit % 64;

        // Steigt alle Ebenen des TopArrays herunter und prüft, ob in den 64-Bit Blöcken bereits das nachfolgende Bit liegt.
        for level in 0..(self.data.len()) {
            let bit_mask: u64 = u64::max_value()
                .checked_shr(in_index as u32 + 1)
                .unwrap_or(0);
            let zeros_to_bit = unsafe { *self.data.get_unchecked(level).add(index) & bit_mask };

            if zeros_to_bit != 0 {
                let zeros = zeros_to_bit.leading_zeros() as usize;
                if zeros != 0 {
                    return Some(self.get_next_set_bit_translation(index * 64 + zeros, level));
                }
            }

            if level < self.data.len() - 1 {
                in_index = index % 64;
                index = index / 64;
            }
        }

        let bit_mask: u64 = u64::max_value()
            .checked_shr(in_index as u32 + 1)
            .unwrap_or(0);
        let mut zeros_to_bit =
            unsafe { (*self.data.get_unchecked(self.data.len() - 1).add(index)) & bit_mask };

        for i in (index)..self.lowest_len {
            if zeros_to_bit != 0 {
                return Some(self.get_next_set_bit_translation(
                    i * 64 + zeros_to_bit.leading_zeros() as usize,
                    self.data.len() - 1,
                ));
            }

            if i < self.lowest_len - 1 {
                zeros_to_bit = unsafe { *self.data.get_unchecked(self.data.len() - 1).add(i + 1) };
            }
        }
        None
    }

    #[inline]
    fn get_prev_set_bit_translation(&self, index: usize, last_level: usize) -> usize {
        let mut index = index;
        for i in (0..(last_level)).rev() {
            let zeros_to_bit = unsafe { *self.data.get_unchecked(i).add(index) };
            index = index * 64 + 63 - zeros_to_bit.trailing_zeros() as usize;
        }
        index
    }

    /// Diese Funktion as nächste Bit zurück, dass vor `bit` gesetzt ist.
    #[inline]
    pub fn get_prev_set_bit(&self, bit: usize) -> Option<usize> {
        let mut index = bit / 64;
        let mut in_index = bit % 64;

        // Steigt alle Ebenen des TopArrays herunter und prüft, ob in den 64-Bit Blöcken bereits das Vorgänger Bit liegt.
        for level in 0..self.data.len() {
            let bit_mask: u64 = u64::max_value()
                .checked_shl(64 - in_index as u32)
                .unwrap_or(0);

            let zeros_to_bit = unsafe { *self.data.get_unchecked(level).add(index) & bit_mask };
            if zeros_to_bit != 0 {
                let zeros = zeros_to_bit.trailing_zeros();

                if zeros != 0 {
                    return Some(
                        self.get_prev_set_bit_translation(index * 64 + 63 - zeros as usize, level),
                    );
                }
            }

            if level < self.data.len() - 1 {
                in_index = index % 64;
                index = index / 64;
            }
        }

        let bit_mask: u64 = u64::max_value()
            .checked_shl(64 - in_index as u32)
            .unwrap_or(0);
        let mut zeros_to_bit =
            unsafe { (*self.data.get_unchecked(self.data.len() - 1).add(index)) & bit_mask };

        for i in (0..(index + 1)).rev() {
            if zeros_to_bit != 0 {
                return Some(self.get_prev_set_bit_translation(
                    i * 64 + 63 - zeros_to_bit.trailing_zeros() as usize,
                    self.data.len() - 1,
                ));
            }

            if i > 0 {
                zeros_to_bit = unsafe { *self.data.get_unchecked(self.data.len() - 1).add(i - 1) };
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
        1 << (22)
    }
}

impl Int for u64 {}

pub type LXKey = u16;
impl<T: Int> STree<T> {
    /// Gibt einen STree mit den in `elements` enthaltenen Werten zurück.
    ///
    /// # Arguments
    ///
    /// * `elements` - Eine Liste mit sortierten u40-Werten, die in die statische Datenstruktur eingefügt werden sollten. Kein Wert darf doppelt vorkommen!
    #[inline]
    pub fn new(elements: Box<[T]>) -> Self {
        let mut builder = STreeBuilder::<T>::new(elements);

        let root_top = builder.get_root_top();
        STree {
            root_table: builder.build(),
            root_top: root_top,
            element_list: std::mem::replace(&mut builder.elements, Box::new([])),
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
    pub fn maximum_level<E>(&self, lx: &Level<E, T>) -> T {
        unsafe { *lx.maximum }
    }

    /// Gibt das Minimum der übergebenen Ebene zurück.
    ///
    /// # Arguments
    ///
    /// * `lx` - Referenz auf die Ebene, dessen Minimum zurückgegeben werden soll.
    #[inline]
    pub fn minimum_level<E>(&self, lx: &Level<E, T>) -> T {
        unsafe { *lx.minimum }
    }

    /// Diese Methode gibt den Index INDEX des größten Elements zurück für das gilt element_list[INDEX]<=element>.
    /// Somit kann mit dieser Methode geprüft werden, ob ein Element in der Datenstruktur enthalten ist. Dann wird der Index dieses Elements zurückgegeben.
    /// Ist das Element nicht enthalten, wird der "Nachfolger" dieses Elements zurückgegeben.
    ///
    /// # Arguments
    ///
    /// * `element` - Evtl. in der Datenstruktur enthaltener Wert, dessen Index zurückgegeben wird. Anderenfalls wird der Index des Vorgängers von `element` zurückgegeben.
    #[inline]
    pub fn locate_or_pred(&self, element: T) -> Option<&T> {
        // Paper z.1
        if element < self.minimum().unwrap() {
            return None;
        }
    

        let (i, l, j, k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || element < *self.root_table[i].minimum()
        {
            return self
                .root_top
                .get_prev_set_bit(i as usize)
                .map(|x| self.root_table[x].maximum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l1_object) => {
                let l2_object = l1_object.try_get(l);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if l2_object.is_none()
                    || element < *l2_object.unwrap().minimum()
                {
                    let new_l = l1_object.lx_top.get_prev_set_bit(l as usize);
                    return unsafe { new_l
                        .and_then(|x| l1_object.try_get(x as LXKey))
                        .map(|x| &*(x.maximum() as *const T)) };
                }

                // Paper z.7
                match l2_object.unwrap().get() {
                    PointerEnum::First(l2_object) => {
                        let l3_object = l2_object.try_get(j);
                        if l3_object.is_none()
                            || element < *l3_object.unwrap().minimum()
                        {
                            let new_j = l2_object.lx_top.get_prev_set_bit(j as usize);
                            return unsafe { new_j
                                .and_then(|x| l2_object.try_get(x as LXKey))
                                .map(|x| &*(x.maximum() as *const T)) };
                        }

                        match l3_object.unwrap().get() {
                            PointerEnum::First(l3_object) => {
                                if l3_object.lx_top.is_set(k as usize) {
                                    return unsafe { Some(&**l3_object.get(k)) };
                                } else {
                                    // Paper z.8
                                    let new_k = l3_object.lx_top.get_prev_set_bit(k as usize);
                                    return unsafe { new_k.map(|x| &**l3_object.try_get(x as LXKey).unwrap()) };
                                }
                            }
                            // Paper z.7
                            PointerEnum::Second(e) => {
                                return Some(e);
                            }
                        }
                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(e);
                    }
                }
            }
            PointerEnum::Second(e) => {
                return Some(e);
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
    pub fn locate_or_succ(&self, element: T) -> Option<&T> {
        // Paper z.1
        if element > self.maximum().unwrap() {
            return None;
        }

        let (i, l, j, k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || *self.root_table[i].maximum() < element
        {
            return self
                .root_top
                .get_next_set_bit(i as usize)
                .map(|x| self.root_table[x].minimum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l1_object) => {
                let l2_object = l1_object.try_get(l);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if l2_object.is_none()
                    || *l2_object.unwrap().maximum() < element
                {
                    let new_l = l1_object.lx_top.get_next_set_bit(l as usize);
                    return unsafe { new_l
                        .and_then(|x| l1_object.try_get(x as LXKey))
                        .map(|x| &*(x.minimum() as *const T)) };
                }

                // Paper z.7
                match l2_object.unwrap().get() {
                    PointerEnum::First(l2_object) => {
                        let l3_object = l2_object.try_get(j);

                        if l3_object.is_none()
                            || *l3_object.unwrap().maximum() < element
                        {
                            let new_j = l2_object.lx_top.get_next_set_bit(j as usize);
                            return unsafe { new_j
                                .and_then(|x| l2_object.try_get(x as LXKey))
                                .map(|x| &*(x.minimum() as *const T)) };
                        }

                        match l3_object.unwrap().get() {
                            PointerEnum::First(l3_object) => {
                                if l3_object.lx_top.is_set(k as usize) {
                                    return unsafe { Some(&**l3_object.get(k)) };
                                } else {
                                    // Paper z.8
                                    let new_k = l3_object.lx_top.get_next_set_bit(k as usize);
                                    return unsafe { new_k.map(|x| &**l3_object.try_get(x as LXKey).unwrap()) };
                                };
                            }
                            // Paper z.7
                            PointerEnum::Second(e) => {
                                return Some(e);
                            }
                        }

                    }
                    PointerEnum::Second(e) => {
                        return Some(e);
                    }
                }
            }

            PointerEnum::Second(e) => {
                return Some(e);
            }
        }
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays.
#[derive(Clone)]
#[repr(align(4))]
pub struct Level<T, E> {
    /// Perfekte Hashmap, die immer (außer zur Inialisierung) gesetzt ist.
    pub hash_map: MphfHashMapThres<LXKey, T>,

    /// Speichert einen Zeiger auf den Index des Maximum dieses Levels
    pub maximum: *const E,

    /// Speichert einen Zeiger auf den Index des Minimums dieses Levels
    pub minimum: *const E,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64].
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    lx_top: TopArray<E,LXKey>,
}

impl<T, E> Level<T, E> {
    /// Gibt ein Level<T> mit Origin-Key j zurück. Optional kann eine Schlüsselliste übergeben werden, für welche dann
    /// eine perfekte Hashfunktion generiert wird.
    ///
    /// # Arguments
    ///
    /// * `j` - Falls eine andere Ebene auf diese mittels Hashfunktion zeigt, muss der verwendete key gespeichert werden.
    /// * `keys` - Eine Liste mit allen Schlüsseln, die mittels perfekter Hashfunktion auf die nächste Ebene zeigen.
    #[inline]
    pub fn new(lx_top: TopArray<E, u16>, objects: Box<[T]>, keys: Box<[LXKey]>, minimum: *const E, maximum: *const E) -> Level<T,E> {
        Level {
            hash_map: MphfHashMapThres::new(keys, objects),
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
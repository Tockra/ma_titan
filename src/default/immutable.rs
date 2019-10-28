use uint::{u40, u48};

use crate::internal::{Splittable};
use crate::default::build::insert_l3_level;
/// Die L2-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf eine
/// L3-Ebene zeigt.
pub type L2Ebene<T> = LevelPointer<L3Ebene<T>, T>;

/// Die L3-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf
/// ein Indize der STree.element_list zeigt.
pub type L3Ebene<T> = LevelPointer<usize, T>;

use crate::internal::{self, PointerEnum};

type HashMap<T,K> = fnv::FnvHashMap<T,K>;

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein usize-Objekt zeigt (Index aus Elementliste),
/// oder auf ein Levelobjekt
#[derive(Clone)]
pub struct LevelPointer<T, E> {
    pointer: internal::Pointer<Level<T, E>, usize>,
}

impl<T, E> LevelPointer<T, E> {
    pub fn minimum(&self) -> usize {
        match self.pointer.get() {
            PointerEnum::First(l) => (*l).minimum,

            PointerEnum::Second(e) => *e,
        }
    }

    pub fn maximum(&self) -> usize {
        match self.pointer.get() {
            PointerEnum::First(l) => (*l).maximum,

            PointerEnum::Second(e) => *e,
        }
    }

    pub fn from_level(level_box: Box<Level<T, E>>) -> Self {
        Self {
            pointer: internal::Pointer::from_first(level_box),
        }
    }

    pub fn get(&self) -> PointerEnum<Level<T, E>, usize> {
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

    pub fn from_usize(usize_box: Box<usize>) -> Self {
        Self {
            pointer: internal::Pointer::from_second(usize_box),
        }
    }

    pub fn change_to_usize(&mut self, usize_box: Box<usize>) {
        self.pointer = internal::Pointer::from_second(usize_box);
    }
}

/// Statische Predecessor-Datenstruktur. Sie verwendet perfektes Hashing und ein Array auf der Element-Listen-Ebene.
/// Sie kann nur sortierte und einmalige Elemente entgegennehmen.
#[derive(Clone)]
pub struct STree<T> {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2-Ebene je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    pub root_table: Box<[L2Ebene<T>]>,

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
            1 << std::mem::size_of::<T>() * 8 / 2
        } else if std::mem::size_of::<V>() == std::mem::size_of::<LXKey>() {
            1 << std::mem::size_of::<T>() * 8 / 4
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
        1 << (std::mem::size_of::<Self>() * 8 / 2)
    }
}

impl Int for u40 {}

impl Int for u48 {}

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
        if elements.len() == 0 {
            panic!("Ein statischer STree muss mindestens 1 Element beinhalten.");
        }
        let mut root_table: Box<[L2Ebene<T>]> = vec![LevelPointer::from_null(); T::root_array_size()].into_boxed_slice();
        let mut root_top = TopArray::<T,usize>::new();

        for (index,elem) in elements.iter().enumerate() {
            let (i,j,k) = Splittable::split_integer_down(elem);
            root_top.set_bit(i as usize);
 
            if root_table[i].is_null() {
                root_table[i] = LevelPointer::from_usize(Box::new(index));
            } else {
                match root_table[i].get() {
                    PointerEnum::First(l2_object) => {
                        l2_object.maximum = index;

                        if !l2_object.lx_top.is_set(j as usize) {
                            let mut l3_level = L3Ebene::from_null();
                            insert_l3_level(&mut l3_level,index,k,&elements);

                            l2_object.hash_map.insert(j,l3_level);
                            l2_object.lx_top.set_bit(j as usize);
                        } else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            insert_l3_level(l2_object.hash_map.get_mut(&j).unwrap(),index,k,&elements);
                        }
      
                    },
                    PointerEnum::Second(elem_index) => {
                        let mut l2_object = Level::new();
                        let elem2 = elements[*elem_index];
                        let (_,j2,k2) = Splittable::split_integer_down(&elem2);
                        
                        // Da die Elemente sortiert sind
                        l2_object.minimum = *elem_index;
                        l2_object.maximum = index;

                        l2_object.lx_top.set_bit(j as usize);

                        let mut l3_level = L3Ebene::from_null();

                        if j2 != j {
                            let mut l3_level = L3Ebene::from_null();
                            insert_l3_level(&mut l3_level,*elem_index,k2,&elements);

                            l2_object.hash_map.insert(j2,l3_level);
                            l2_object.lx_top.set_bit(j2 as usize)
                        } else {
                            insert_l3_level(&mut l3_level,*elem_index,k2,&elements);
                        }
 
                        insert_l3_level(&mut l3_level,index,k,&elements);
                        l2_object.hash_map.insert(j,l3_level);

                        root_table[i] = L2Ebene::from_level(Box::new(l2_object));
                    },
                }
            }
        }
        STree {
            root_table: root_table,
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
    pub fn maximum_level<E>(&self, lx: &Level<E, T>) -> T {
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

        let (i, j, k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || element < self.element_list[self.root_table[i].minimum()]
        {
            return self
                .root_top
                .get_prev_set_bit(i as usize)
                .map(|x| self.root_table[x].maximum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none()
                    || element < self.element_list[third_level.unwrap().minimum()]
                {
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
                            return new_k.map(|x| *(*l).try_get(x as LXKey).unwrap());
                        }
                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(*e);
                    }
                }
            }

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

        let (i, j, k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || self.element_list[self.root_table[i].maximum()] < element
        {
            return self
                .root_top
                .get_next_set_bit(i as usize)
                .map(|x| self.root_table[x].minimum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none()
                    || self.element_list[third_level.unwrap().maximum()] < element
                {
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
                            return new_k.map(|x| *(*l).try_get(x as LXKey).unwrap());
                        };
                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(*e);
                    }
                }
            }

            PointerEnum::Second(e) => {
                return Some(*e);
            }
        }
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays.
#[derive(Clone)]
#[repr(align(4))]
pub struct Level<T, E> {
    /// Perfekte Hashmap, die immer (außer zur Inialisierung) gesetzt ist.
    pub hash_map: HashMap<LXKey, T>,

    /// Speichert einen Zeiger auf den Index des Maximum dieses Levels
    pub maximum: usize,

    /// Speichert einen Zeiger auf den Index des Minimums dieses Levels
    pub minimum: usize,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64].
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    pub lx_top: TopArray<E, u16>,
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
    pub fn new() -> Level<T, E> {
        Level {
            hash_map: HashMap::default(),
            minimum: 0,
            maximum: 0,
            lx_top: TopArray::new(),
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
            self.hash_map.get(&key)
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
        self.hash_map.get_mut(&key).unwrap()
    }
}

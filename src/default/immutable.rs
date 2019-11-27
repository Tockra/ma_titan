use uint::{u40, u48};

use crate::internal::{Splittable};
use crate::default::build::insert_l3_level;
/// Die L2-Ebene ist eine Zwischenebene, die mittels eines u8-Integers und einer originale STree Hashtabelle auf eine
/// L3-Ebene zeigt.
pub type L2Ebene<T> = LevelPointer<L3Ebene<T>, T>;

/// Die L3-Ebene ist eine Zwischenebene, die mittels eines u8-Integers und einer originale STree Hashtabelle auf
/// ein Indize der STree.element_list zeigt.
pub type L3Ebene<T> = LevelPointer<*const T, T>;

use crate::internal::{self, PointerEnum};

type HashMap<V> = DynamicLookup<V>;

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein usize-Objekt zeigt (Index aus Elementliste),
/// oder auf ein Levelobjekt
#[derive(Clone)]
pub struct LevelPointer<T, E> {
    pointer: internal::Pointer<Level<T, E>, E>,
}

impl<T, E> LevelPointer<T, E> {

    #[inline]
    pub fn minimum(&self) -> &E {
        match self.pointer.get() {
            PointerEnum::First(l) => unsafe { &*l.minimum },

            PointerEnum::Second(e) => &*e,
        }
    }

    #[inline]
    pub fn maximum(&self) -> &E {
        match self.pointer.get() {
            PointerEnum::First(l) => unsafe { &*l.maximum },

            PointerEnum::Second(e) => &*e,
        }
    }

    #[inline]
    pub fn from_level(level_box: Box<Level<T, E>>) -> Self {
        Self {
            pointer: internal::Pointer::from_first(level_box),
        }
    }

    #[inline]
    pub fn get(&self) -> PointerEnum<Level<T, E>, E> {
        self.pointer.get()
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    #[inline]
    pub fn from_null() -> Self {
        Self {
            pointer: internal::Pointer::null(),
        }
    }

    #[inline]
    pub fn from_usize(usize_box: *const E) -> Self {
        Self {
            pointer: internal::Pointer::from_second(usize_box),
        }
    }

    #[inline]
    pub fn change_to_usize(&mut self, usize_box: *const E) {
        self.pointer = internal::Pointer::from_second(usize_box);
    }
}

/// Statische Predecessor-Datenstruktur. Sie verwendet originale STree Hashtabelle und ein Array auf der Element-Listen-Ebene.
/// Sie kann nur sortierte und einmalige Elemente entgegennehmen.
#[derive(Clone)]
pub struct STree<T> {
    /// Mit Hilfe der ersten 24-Bits des zu speichernden Wortes wird in `root_table` eine L2-Ebene je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^24]`
    pub root_table: Box<[L2Ebene<T>]>,

    /// Das Root-Top-Array speichert für jeden Eintrag `root_table[i][x]`, der belegt ist, ein 1-Bit, sonst einen 0-Bit.
    /// Auch hier werden nicht 2^24 Einträge, sondern lediglich [u64;2^24/64] gespeichert.
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
    data: Box<[*mut u32]>,

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
            length = length >> (i + 1) * 5;
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
            length = length >> (i + 1) * 5;
            let mut tmp = vec![];
            unsafe {
                for i in 0..length {
                    tmp.push(*ptr.add(i));
                }
            }
            top_arrays.push(Box::into_raw(tmp.into_boxed_slice()) as *mut u32);
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
            1 << std::mem::size_of::<T>() * 8 - 16
        } else if std::mem::size_of::<V>() == std::mem::size_of::<LXKey>() {
            1 << 8
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
        while length >= 32 {
            length = length >> 5;
            top_arrays.push(Box::into_raw(vec![0_u32; length].into_boxed_slice()) as *mut u32);
        }

        Self {
            data: top_arrays.into_boxed_slice(),
            lowest_len: length,
            phantom: std::marker::PhantomData,
            phantom_type: std::marker::PhantomData,
        }
    }

    #[inline]
    const fn get_bit_mask(in_index: usize) -> u32 {
        1 << 31 - in_index
    }

    /// Baut das Root-Top-Array mit Hilfe der sich in der Datenstruktur befindenden Werte.
    #[inline]
    pub fn set_bit(&mut self, bit: usize) {
        let mut index = bit / 32;
        let mut in_index = bit % 32;

        // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
        for i in 0..(self.data.len()) {
            // Aktueller in_index wird für Bitmaske verwendet
            let bit_mask = Self::get_bit_mask(in_index);
            let bit_window = unsafe { self.data.get_unchecked(i).add(index) };

            in_index = index % 32;
            index = index / 32;

            unsafe {
                *bit_window = *bit_window | bit_mask;
            }
        }
    }

    #[inline]
    pub fn is_set(&self, bit: usize) -> bool {
        let (index, in_index) = (bit / 32, bit % 32);
        let bit_mask = Self::get_bit_mask(in_index);
        let bit_window = unsafe { self.data.get_unchecked(0).add(index) };

        unsafe { *bit_window & bit_mask != 0 }
    }

    #[inline]
    fn get_next_set_bit_translation(&self, index: usize, last_level: usize) -> usize {
        let mut index = index;
        for i in (0..(last_level)).rev() {
            let zeros_to_bit = unsafe { *self.data.get_unchecked(i).add(index) };
            index = index * 32 + zeros_to_bit.leading_zeros() as usize;
        }
        index
    }

    /// Diese Funktion as nächste Bit zurück, dass hinter `bit` gesetzt ist.
    #[inline]
    pub fn get_next_set_bit(&self, bit: usize) -> Option<usize> {
        let mut index = bit / 32;
        let mut in_index = bit % 32;

        // Steigt alle Ebenen des TopArrays herunter und prüft, ob in den 64-Bit Blöcken bereits das nachfolgende Bit liegt.
        for level in 0..(self.data.len()) {
            let bit_mask: u32 = u32::max_value()
                .checked_shr(in_index as u32 + 1)
                .unwrap_or(0);
            let zeros_to_bit = unsafe { *self.data.get_unchecked(level).add(index) & bit_mask };

            if zeros_to_bit != 0 {
                let zeros = zeros_to_bit.leading_zeros() as usize;
                if zeros != 0 {
                    return Some(self.get_next_set_bit_translation(index * 32 + zeros, level));
                }
            }

            if level < self.data.len() - 1 {
                in_index = index % 32;
                index = index / 32;
            }
        }

        let bit_mask: u32 = u32::max_value()
            .checked_shr(in_index as u32 + 1)
            .unwrap_or(0);
        let mut zeros_to_bit =
            unsafe { (*self.data.get_unchecked(self.data.len() - 1).add(index)) & bit_mask };

        for i in (index)..self.lowest_len {
            if zeros_to_bit != 0 {
                return Some(self.get_next_set_bit_translation(
                    i * 32 + zeros_to_bit.leading_zeros() as usize,
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
            index = index * 32 + 63 - zeros_to_bit.trailing_zeros() as usize;
        }
        index
    }

    /// Diese Funktion as nächste Bit zurück, dass vor `bit` gesetzt ist.
    #[inline]
    pub fn get_prev_set_bit(&self, bit: usize) -> Option<usize> {
        let mut index = bit / 32;
        let mut in_index = bit % 32;

        // Steigt alle Ebenen des TopArrays herunter und prüft, ob in den 64-Bit Blöcken bereits das Vorgänger Bit liegt.
        for level in 0..self.data.len() {
            let bit_mask: u32 = u32::max_value()
                .checked_shl(32 - in_index as u32)
                .unwrap_or(0);

            let zeros_to_bit = unsafe { *self.data.get_unchecked(level).add(index) & bit_mask };
            if zeros_to_bit != 0 {
                let zeros = zeros_to_bit.trailing_zeros();

                if zeros != 0 {
                    return Some(
                        self.get_prev_set_bit_translation(index * 32 + 31 - zeros as usize, level),
                    );
                }
            }

            if level < self.data.len() - 1 {
                in_index = index % 32;
                index = index / 32;
            }
        }

        let bit_mask: u32 = u32::max_value()
            .checked_shl(32 - in_index as u32)
            .unwrap_or(0);
        let mut zeros_to_bit =
            unsafe { (*self.data.get_unchecked(self.data.len() - 1).add(index)) & bit_mask };

        for i in (0..(index + 1)).rev() {
            if zeros_to_bit != 0 {
                return Some(self.get_prev_set_bit_translation(
                    i * 32 + 31 - zeros_to_bit.trailing_zeros() as usize,
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
pub trait Int: Ord + PartialOrd + Into<u64> + Copy + Splittable {
    fn new(k: u64) -> Self;

    fn root_array_size() -> usize {
        1 << (std::mem::size_of::<Self>() * 8 - 16)
    }
}

impl Int for u32 {
    fn new(k: u64) -> Self {
        k as u32
    }
}

impl Int for u40 {
    fn new(k: u64) -> Self {
        Self::from(k)
    }
}

impl Int for u48 {
    fn new(k: u64) -> Self {
        Self::from(k)
    }
}

impl Int for u64 {
    fn new(k: u64) -> Self {
        Self::from(k)
    }
}

pub type LXKey = u8;
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

        for (index,_) in elements.iter().enumerate() {
            let (i,j,k) = Splittable::split_integer_down(&elements[index]);
            root_top.set_bit(i as usize);
 
            if root_table[i].is_null() {
                root_table[i] = LevelPointer::from_usize(&elements[index] as *const T);
            } else {
                match root_table[i].get() {
                    PointerEnum::First(l2_object) => {
                        l2_object.maximum = &elements[index] as *const T;

                        if !l2_object.lx_top.is_set(j as usize) {
                            let mut l3_level = L3Ebene::from_null();
                            insert_l3_level(&mut l3_level,&elements[index],k,&elements);

                            l2_object.hash_map.insert(j,l3_level);
                            l2_object.lx_top.set_bit(j as usize);
                        } else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            insert_l3_level(l2_object.hash_map.get_mut(j).unwrap(),&elements[index],k,&elements);
                        }
      
                    },
                    PointerEnum::Second(elem_index) => {
                        let mut l2_object = Level::new();
                        let elem2 = *elem_index;
                        let (_,j2,k2) = Splittable::split_integer_down(&elem2);
                        
                        // Da die Elemente sortiert sind
                        l2_object.minimum = elem_index as *const T;
                        l2_object.maximum = &elements[index] as *const T;

                        l2_object.lx_top.set_bit(j as usize);

                        let mut l3_level = L3Ebene::from_null();

                        if j2 != j {
                            let mut l3_level = L3Ebene::from_null();
                            insert_l3_level(&mut l3_level,elem_index,k2,&elements);

                            l2_object.hash_map.insert(j2,l3_level);
                            l2_object.lx_top.set_bit(j2 as usize)
                        } else {
                            insert_l3_level(&mut l3_level,elem_index,k2,&elements);
                        }
 
                        insert_l3_level(&mut l3_level,&elements[index],k,&elements);
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

        let (i, j, k) = Splittable::split_integer_down(&element);

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
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none()
                    || element < *third_level.unwrap().minimum()
                {
                    let new_j = second_level.lx_top.get_prev_set_bit(j as usize);
                    return unsafe { new_j
                        .and_then(|x| second_level.try_get(x as LXKey))
                        .map(|x| &*(x.maximum() as *const T))};
                }

                // Paper z.7
                match third_level.unwrap().get() {
                    PointerEnum::First(l) => {
                        if l.lx_top.is_set(k as usize) {
                            return unsafe { Some(&**l.get(k)) };
                        } else {
                            // Paper z.8
                            let new_k = (*l).lx_top.get_prev_set_bit(k as usize);
                            return unsafe { new_k.map(|x| &**l.try_get(x as LXKey).unwrap()) };
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

        let (i, j, k) = Splittable::split_integer_down(&element);

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
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none()
                    || *third_level.unwrap().maximum() < element
                {
                    let new_j = second_level.lx_top.get_next_set_bit(j as usize);
                    return unsafe { new_j
                        .and_then(|x| second_level.try_get(x as LXKey))
                        .map(|x| &*(x.minimum() as *const T)) };
                }

                // Paper z.7
                match third_level.unwrap().get() {
                    PointerEnum::First(l) => {
                        if l.lx_top.is_set(k as usize) {
                            return unsafe { Some(&**l.get(k)) };
                        } else {
                            // Paper z.8
                            let new_k = (*l).lx_top.get_next_set_bit(k as usize);
                            return new_k.map(|x| unsafe {  &**l.try_get(x as LXKey).unwrap()} );
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
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays.
#[derive(Clone)]
#[repr(align(4))]
pub struct Level<T, E> {
    /// Perfekte Hashmap, die immer (außer zur Inialisierung) gesetzt ist.
    pub hash_map: HashMap<T>,

    /// Speichert einen Zeiger auf den Index des Maximum dieses Levels
    pub maximum: *const E,

    /// Speichert einen Zeiger auf den Index des Minimums dieses Levels
    pub minimum: *const E,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^8 (Bits) besitzt. Also [u64;2^8/64].
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    pub lx_top: TopArray<E, u8>,
}

impl<T: Clone, E> Level<T, E> {
    /// Gibt ein Level<T> mit Origin-Key j zurück. Optional kann eine Schlüsselliste übergeben werden, für welche dann
    /// eine originale STree Hashtabelle generiert wird.
    ///
    /// # Arguments
    ///
    /// * `j` - Falls eine andere Ebene auf diese mittels Hashfunktion zeigt, muss der verwendete key gespeichert werden.
    /// * `keys` - Eine Liste mit allen Schlüsseln, die mittels originale STree Hashtabelle auf die nächste Ebene zeigen.
    #[inline]
    pub fn new() -> Level<T, E> {
        Level {
            minimum: std::ptr::null(),
            maximum: std::ptr::null(),
            hash_map: HashMap::new(),
            lx_top: TopArray::new(),
        }
    }

    /// Mit Hilfe dieser Funktion kann die originale STree Hashtabelle verwendet werden.
    /// Es muss beachtet werden, dass sichergestellt werden muss, dass der verwendete Key auch existiert!
    ///
    /// # Arguments
    ///
    /// * `key` - u8-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn try_get(&self, key: LXKey) -> Option<&T> {
        if self.lx_top.is_set(key as usize) {
            self.hash_map.get(key)
        } else {
            None
        }
    }

    /// Der zum `key` gehörende gehashte Wert wird aus der Datenstruktur ermittelt. Hierbei muss sichergestellt sein
    /// das zu `key` ein Schlüssel gehört. Anderenfalls sollte `try_hash` verwendet werden
    ///
    /// # Arguments
    ///
    /// * `key` - u8-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get(&mut self, key: LXKey) -> &mut T {
        self.hash_map.get_mut(key).unwrap()
    }
}

pub struct DynamicLookup<E> {

    // keys.len == objects.len
    keys: *mut [u8;4],

    objects: *mut [Option<E>;4],

    shift_value: u8,

    array_len: u16,

    size: u8,
}

impl<E> Drop for DynamicLookup<E> {
    fn drop(&mut self) {
        unsafe {
            Box::from_raw(std::slice::from_raw_parts_mut(self.keys, (self.array_len/4) as usize));
            Box::from_raw(std::slice::from_raw_parts_mut(self.objects, (self.array_len/4) as usize));   
        }
    }
}

impl<E: Clone> Clone for DynamicLookup<E> {
    fn clone(&self) -> Self {
        unsafe {

            let mut new_keys = vec![];
            for i in 0..self.array_len/4 {
                new_keys.push(*self.keys.add(i as usize));
            }

            let mut new_objects = vec![];
            for i in 0..self.array_len/4 {
                let mut tmp: [Option<E>; 4] = [None, None, None, None];
                for j in 0..4 {
                    tmp[j] = (*self.objects.add(i as usize))[j].clone();
                }
                new_objects.push(tmp);
            }

        
            Self {
                keys: Box::into_raw(new_keys.into_boxed_slice()) as *mut [u8;4],
                objects: Box::into_raw(new_objects.into_boxed_slice()) as *mut [Option<E>;4],
                shift_value: self.shift_value,
                array_len: self.array_len,
                size: self.size,
            }
        }
    }
}

impl<E: Clone> DynamicLookup<E> {
    const HASHMAP: [u8;256] = [127, 254, 59, 44, 146, 151, 118, 112, 137, 47, 164, 4, 1, 86, 14, 37, 100, 45, 189, 194, 169, 89, 144, 188, 12, 236, 84, 34, 219, 65, 72, 131, 78, 222, 29, 19, 225, 130, 2, 42, 179, 193, 197, 54, 10, 35, 232, 175, 145, 174, 227, 135, 87, 167, 150, 125, 3, 214, 204, 119, 171, 5, 241, 66, 11, 109, 26, 160, 41, 191, 96, 196, 234, 183, 198, 80, 170, 157, 163, 57, 148, 83, 21, 233, 147, 195, 9, 50, 153, 156, 158, 190, 32, 143, 120, 103, 82, 230, 46, 52, 13, 200, 18, 218, 165, 149, 95, 106, 94, 242, 20, 60, 51, 6, 250, 104, 152, 63, 0, 129, 223, 88, 154, 173, 75, 177, 73, 110, 226, 244, 255, 33, 134, 8, 166, 211, 159, 252, 36, 98, 39, 56, 247, 77, 215, 43, 25, 181, 162, 115, 48, 64, 209, 101, 216, 220, 202, 107, 212, 132, 184, 138, 31, 199, 186, 176, 245, 117, 133, 58, 185, 205, 85, 187, 207, 231, 201, 246, 237, 23, 93, 105, 49, 61, 240, 203, 81, 210, 67, 238, 99, 217, 180, 141, 192, 71, 228, 239, 16, 126, 124, 224, 22, 172, 182, 235, 111, 38, 249, 243, 128, 251, 55, 28, 53, 24, 161, 139, 102, 76, 114, 123, 17, 30, 178, 136, 90, 206, 248, 229, 168, 121, 122, 79, 40, 116, 221, 213, 91, 70, 108, 7, 69, 113, 97, 142, 68, 155, 15, 140, 62, 208, 27, 92, 253, 74];
    /// Vorbindung: keys sind sortiert. Weiterhin gilt keys.len() == objects.len() und  keys.len() > 0
    /// Nachbedingung : keys[i] -> objects[i]
    pub fn new() -> Self {
        // benötigt die Eigenschaft, dass die keys sortiert sind
        let keys: Vec<[u8;4]> = vec![[0; 4]];
        let objects: Vec<[Option<E>;4]> = vec![[None, None, None, None]];
        
        Self {
            keys: Box::into_raw(keys.into_boxed_slice()) as *mut [u8;4],
            objects: Box::into_raw(objects.into_boxed_slice()) as *mut [Option<E>;4],
            array_len: 4,
            size: 0,
            shift_value: 6,
        }
    }
    #[inline]
    pub fn get(&self, key: u8) -> Option<&E> {
        let mut n = Self::HASHMAP[key as usize] >> self.shift_value;
        let mut m = n >> 2;
        let mut i = n & 3;
        unsafe {
            while !(*self.objects.add(m as usize))[i as usize].is_none() {
                if (*self.keys.add(m as usize))[i as usize] == key {
                    return (*self.objects.add(m as usize))[i as usize].as_ref();
                }
                n = (n+1) & ((self.array_len-1) as u8);
                m = n >> 2;
                i = n & 3;
            }
        }

        return None;
    }
    #[inline]
    pub fn get_mut(&self, key: u8) -> Option<&mut E> {
        let mut n = Self::HASHMAP[key as usize] >> self.shift_value;
        let mut m = n >> 2;
        let mut i = n & 3;
        unsafe {
            while !(*self.objects.add(m as usize))[i as usize].is_none() {
                if (*self.keys.add(m as usize))[i as usize] == key {
                    return (*self.objects.add(m as usize))[i as usize].as_mut();
                }
                n = (n+1) & ((self.array_len-1) as u8);
                m = n >> 2;
                i = n & 3;
            }
        }

        return None;
    }
    #[inline]
    fn double_size(&mut self) {
        unsafe {
            debug_assert!(self.array_len <= 128);
            self.shift_value -= 1;
            self.array_len *= 2;
            let new_keys = vec![[0;4]; (self.array_len/4) as usize];
            let mut new_objects: Vec<[Option<E>;4]> = Vec::with_capacity((self.array_len/4) as usize);
            for _ in 0..self.array_len/4 {
                new_objects.push([None, None, None, None]);
            }
            let old_keys = Box::from_raw(std::slice::from_raw_parts_mut(std::mem::replace(&mut self.keys, Box::into_raw(new_keys.into_boxed_slice()) as *mut [u8;4]),(self.array_len/8) as usize));
            let mut old_objects = Box::from_raw(std::slice::from_raw_parts_mut(std::mem::replace(&mut self.objects, Box::into_raw(new_objects.into_boxed_slice()) as *mut [Option<E>;4]),(self.array_len/8) as usize));
            
            self.size = 0;
            for i in 0..self.array_len/2 {
                let index = (i>>2) as usize;
                let sub_index = (i&3) as usize;
                let tmp_key = old_keys[index][sub_index];
                let tmp_object = std::mem::replace(&mut old_objects[index][sub_index], None);
                if !tmp_object.is_none() {
                    self.insert(tmp_key, tmp_object.unwrap());
                }
            }
        }
    }
    #[inline]
    pub fn insert(&mut self, key: u8, elem: E) {
        unsafe {
            if (self.size as u16) < (self.array_len - ( self.array_len >> 2)) || self.array_len == 256 {
                let mut n = (Self::HASHMAP[key as usize] >> self.shift_value) as usize;
                while !(*self.objects.add(n>>2))[n&3].is_none() {
                    n = (n+1) & (self.array_len-1) as usize;
                }

                (*self.keys.add(n>>2))[n&3] = key;
                (*self.objects.add(n>>2))[n&3] = Some(elem);

                self.size += 1;
            } else {
                self.double_size();
                self.insert(key,elem);
            }
        }
        
    }

    
}

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::default::immutable::{Int, L2Ebene, LXKey, Level, LevelPointer, TopArray};
use crate::internal::Splittable;

type HashMap<K, T> = hashbrown::hash_map::HashMap<K, T>;

/// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen,
/// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
pub const GAMMA: f64 = 2.0;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L2-Ebene hat.AsMut
type L2EbeneBuilder<T> = internal::Pointer<BuilderLevel<L3EbeneBuilder<T>, T>, usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L3-Ebene hat.AsMut
type L3EbeneBuilder<T> = internal::Pointer<BuilderLevel<usize, T>, usize>;

/// Hilfsdatenstruktur zum Bauen eines STrees (nötig wegen der perfekten Hashfunktionen, die zum Erzeugungszeitpunkt alle Schlüssel kennen müssen).
pub struct STreeBuilder<T: 'static> {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2EbeneBuilder je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    root_table: Box<[L2EbeneBuilder<T>]>,

    /// Root-Top-Array
    root_top: Option<TopArray<T, usize>>,

    /// Eine Liste, die alle belegten Indizes von `root_table` speichert.
    root_indexs: Vec<usize>,
}

impl<T: Int> STreeBuilder<T> {
    /// Gibt einen STreeBuilder mit den in `elements` enthaltenen Werten zurück. Dabei werden normale Hashfunktionen verwendet.
    ///
    /// # Arguments
    ///
    /// * `elements` - Eine Liste mit sortierten u40-Werten, die in die statische Datenstruktur eingefügt werden sollten. Kein Wert darf doppelt vorkommen!
    pub fn new(elements: Box<[T]>) -> Self {
        let mut root_indexs = vec![];
        let mut root_top: TopArray<T, usize> = TopArray::new();

        // Hier wird ein root_array der Länge T::root_array_size() angelegt, was 2^i entspricht. Dabei entspricht bei einem u40 Integer i=40 .
        let mut root_table: Box<[L2EbeneBuilder<T>]> =
            vec![internal::Pointer::null(); T::root_array_size()].into_boxed_slice();

        for (index, element) in elements.iter().enumerate() {
            let (i, j, k) = Splittable::split_integer_down(element);

            if !root_top.is_set(i as usize) {
                root_top.set_bit(i as usize);
                root_indexs.push(i);
            }

            if root_table[i].is_null() {
                root_table[i] = internal::Pointer::from_second(Box::new(index));
            } else {
                match root_table[i].get() {
                    PointerEnum::First(l) => {
                        let second_level = l;
                        second_level.maximum = index;

                        if !second_level.lx_top.as_ref().unwrap().is_set(j as usize) {
                            second_level.keys.push(j);

                            let mut l3_level = internal::Pointer::null();
                            Self::insert_l3_level(&mut l3_level, index, k, &elements);

                            second_level.hash_map.insert(j, l3_level);
                            second_level.lx_top.as_mut().unwrap().set_bit(j as usize);
                        } else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            Self::insert_l3_level(
                                second_level.hash_map.get_mut(j).unwrap(),
                                index,
                                k,
                                &elements,
                            );
                        }
                    }

                    PointerEnum::Second(e) => {
                        let (_, j2, k2) = Splittable::split_integer_down(&elements[*e]);
                        let mut second_level = BuilderLevel::new();

                        second_level.lx_top.as_mut().unwrap().set_bit(j as usize);
                        // Minima- und Maximasetzung auf der ersten Ebene
                        second_level.minimum = *e;
                        second_level.maximum = index;

                        let mut l3_level = internal::Pointer::null();

                        if j2 != j {
                            let mut l3_level = internal::Pointer::null();
                            Self::insert_l3_level(&mut l3_level, *e, k2, &elements);

                            second_level.keys.push(j2);
                            second_level.hash_map.insert(j2, l3_level);
                            second_level.lx_top.as_mut().unwrap().set_bit(j2 as usize)
                        } else {
                            Self::insert_l3_level(&mut l3_level, *e, k2, &elements);
                        }

                        // Reihenfolge der keys ist relevant!
                        second_level.keys.push(j);
                        Self::insert_l3_level(&mut l3_level, index, k, &elements);
                        second_level.hash_map.insert(j, l3_level);

                        root_table[i] = internal::Pointer::from_first(Box::new(second_level));
                    }
                }
            }
        }
        Self {
            root_table: root_table,
            root_top: Some(root_top),
            root_indexs: root_indexs,
        }
    }
    #[inline]
    fn insert_l3_level(l3_level: &mut L3EbeneBuilder<T>, index: usize, k: LXKey, elements: &[T]) {
        if l3_level.is_null() {
            *l3_level = internal::Pointer::from_second(Box::new(index));
        } else {
            match l3_level.get() {
                PointerEnum::First(l) => {
                    let l3_level = l;

                    debug_assert!(!l3_level.keys.contains(&k));

                    l3_level.lx_top.as_mut().unwrap().set_bit(k as usize);
                    l3_level.keys.push(k);

                    //Maximasetzung auf der zweiten Ebene
                    l3_level.maximum = index;

                    l3_level.hash_map.insert(k, index);
                }

                PointerEnum::Second(e) => {
                    let (_, _, k2) = Splittable::split_integer_down(&elements[*e]);
                    let mut l3_level_n = BuilderLevel::new();
                    l3_level_n.keys.push(k2);
                    l3_level_n.keys.push(k);

                    debug_assert!(k2 != k);

                    // Minima- und Maximasetzung auf der zweiten Ebene
                    l3_level_n.minimum = *e;
                    l3_level_n.maximum = index;

                    l3_level_n.hash_map.insert(k2, *e);
                    l3_level_n.hash_map.insert(k, index);

                    l3_level_n.lx_top.as_mut().unwrap().set_bit(k as usize);
                    l3_level_n.lx_top.as_mut().unwrap().set_bit(k2 as usize);

                    *l3_level = internal::Pointer::from_first(Box::new(l3_level_n));
                }
            }
        }
    }

    /// Baut ein Array `root_table` für den STree-Struct. Dabei werden zuerst die `Level`-Structs korrekt mittels neuer perfekter Hashfunktionen
    /// angelegt und miteinander verbunden. Nachdem die Struktur mit normalen Hashfunktionen gebaut wurde können nun perfekte Hashfunktionen berechnet
    /// werden!
    pub fn build(&mut self) -> Box<[L2Ebene<T>]> {
        let mut tmp: Vec<L2Ebene<T>> = Vec::with_capacity(T::root_array_size());
        // Die L2Level-Elemente werden angelegt. Hierbei wird direkt in der new()-Funktion die perfekte Hashfunktion berechnet
        for i in 0..tmp.capacity() {
            if self.root_table[i].is_null() {
                tmp.push(LevelPointer::from_null());
            } else {
                match self.root_table[i].get() {
                    PointerEnum::First(l) => {
                        let second_level = l;
                        let objects: Vec<LevelPointer<usize, T>> =
                            vec![LevelPointer::from_null(); second_level.keys.len()];
                        let val = Box::new(Level::new(
                            second_level.lx_top.take().unwrap(),
                            objects.into_boxed_slice(),
                            second_level.keys.clone().into_boxed_slice(),
                            second_level.minimum,
                            second_level.maximum,
                        ));
                        tmp.push(LevelPointer::from_level(val));
                    }

                    PointerEnum::Second(e) => {
                        tmp.push(LevelPointer::from_usize(Box::new(*e)));
                    }
                }
            }
        }
        let result: Box<[L2Ebene<T>]> = tmp.into_boxed_slice();

        for &i in &self.root_indexs {
            // L3-Level werden nur angelegt, falls mehr als 1 Wert in der DS existiert.
            if !result[i].is_null() {
                match &mut result[i].get() {
                    PointerEnum::First(l) => {
                        // Hier muss l2_level aufgrund der symmetrischen Befüllung auch == Ptr::Level sein.LevelPointerBuilder
                        match std::mem::replace(&mut self.root_table[i], L2EbeneBuilder::null())
                            .get()
                        {
                            PointerEnum::First(l2) => {
                                let l2_level = l2;

                                for &j in &l2_level.keys {
                                    // Die L2-Top-Tabellen werden gefüllt und die
                                    let l3_level = l2_level.hash_map.get_mut(j).unwrap();
                                    // TODO
                                    if (*l).get(j).is_null() {
                                        let pointered_data = (*l).get(j);

                                        *pointered_data = match l3_level.get() {
                                            PointerEnum::First(l2) => {
                                                let l3_level = l2;
                                                let mut level = Level::new(
                                                    l3_level.lx_top.take().unwrap(),
                                                    vec![0; l3_level.keys.len()].into_boxed_slice(),
                                                    l3_level.keys.clone().into_boxed_slice(),
                                                    l3_level.minimum,
                                                    l3_level.maximum,
                                                );
                                                for k in &l3_level.keys {
                                                    let result = level.get(*k);
                                                    *result = *l3_level.hash_map.get(*k).unwrap();
                                                }

                                                LevelPointer::from_level(Box::new(level))
                                            }
                                            PointerEnum::Second(e) => {
                                                LevelPointer::from_usize(Box::new(*e))
                                            }
                                        };
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    _ => {}
                }
            }
        }
        result
    }

    pub fn get_root_top(&mut self) -> TopArray<T, usize> {
        self.root_top.take().unwrap()
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays.
#[derive(Clone)]
pub struct BuilderLevel<T: 'static, E: 'static> {
    /// Klassische HashMap zum aufbauen der perfekten Hashmap
    pub hash_map: DynamicLookup<T>,

    /// Eine Liste aller bisher gesammelter Schlüssel, die später auf die nächste Ebene zeigen.
    /// Diese werden zur Erzeugung der perfekten Hashfunktion benötigt.
    pub keys: Vec<LXKey>,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64].
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    /// Dieses Array wird später an den `Level`-Struct weitergegeben
    lx_top: Option<TopArray<E, LXKey>>,

    /// Speichert das Maximum des Levels zwischen
    pub maximum: usize,

    /// Speichert das Minimum des Levels zwischen
    pub minimum: usize,
}

impl<T: Clone, E> BuilderLevel<T, E> {
    /// Gibt ein BuilderLevel<T> zurück.
    ///
    /// # Arguments
    ///
    /// * `lx_top_size` - Gibt die Länge des Arrays `lx_top_size` an.
    #[inline]
    pub fn new() -> BuilderLevel<T, E> {
        BuilderLevel {
            hash_map: DynamicLookup::new(),
            keys: vec![],
            lx_top: Some(TopArray::new()),
            maximum: 0,
            minimum: 1,
        }
    }
}

// ------------------------- Pointer Magie, zum Verhindern der Nutzung von HashMaps für kleine Datenmengen ----------------------------------

use crate::internal::{self, PointerEnum};
pub struct BuildHM<K, T> {
    pointer: internal::Pointer<HashMap<K, T>, (Box<Vec<K>>, Box<Vec<T>>)>,
}

impl<K: 'static + Clone, T: 'static + Clone> Clone for BuildHM<K, T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
        }
    }
}

impl<K: 'static + Eq + Copy + Ord + std::hash::Hash, T: 'static> BuildHM<K, T> {
    fn new() -> Self {
        Self {
            pointer: internal::Pointer::from_second(Box::new((Box::new(vec![]), Box::new(vec![])))),
        }
    }

    /// Die eigentliche Updatemechanik der HashMaps, wird hier ignoriert, da keine Werte geupdatet werden müssen!
    pub fn insert(&mut self, key: K, val: T) {
        match self.pointer.get() {
            PointerEnum::Second((keys, values)) => {
                if true {
                    keys.push(key);
                    values.push(val);
                } else {
                    let mut hm = HashMap::<K, T>::with_capacity(1025);
                    let values = std::mem::replace(values, Box::new(vec![]));
                    for (i, val) in values.into_iter().enumerate() {
                        hm.insert(keys[i], val);
                    }
                    hm.insert(key, val);
                    self.pointer = internal::Pointer::from_first(Box::new(hm));
                }
            }
            PointerEnum::First(x) => {
                x.insert(key, val);
            }
        }
    }

    fn get_mut(&mut self, k: &K) -> Option<&mut T> {
        match self.pointer.get() {
            PointerEnum::Second((keys, values)) => match keys.binary_search(k) {
                Ok(x) => values.get_mut(x),
                Err(_) => None,
            },
            PointerEnum::First(x) => x.get_mut(k),
        }
    }

    fn get(&mut self, k: &K) -> Option<&T> {
        match self.pointer.get() {
            PointerEnum::Second((keys, values)) => match keys.binary_search(k) {
                Ok(x) => values.get(x),
                Err(_) => None,
            },
            PointerEnum::First(x) => x.get(k),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------------------------------------------------------
/// Diese Datenstruktur dient als naive Hashmap. Sie speichert eine Lookuptable und die Daten


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
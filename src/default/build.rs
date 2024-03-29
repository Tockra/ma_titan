use crate::default::immutable::{Int, L2Ebene, LXKey, Level, LevelPointer, TopArray};
use crate::internal::Splittable;
use crate::internal::{self, PointerEnum};

type HashMap<K, T> = hashbrown::hash_map::HashMap<K, T>;

/// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen,
/// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
pub const GAMMA: f64 = 2.0;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L2-Ebene hat.AsMut
type L2EbeneBuilder<T> = internal::Pointer<BuilderLevel<L3EbeneBuilder<T>, T>, usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L3-Ebene hat.AsMut
type L3EbeneBuilder<T> = internal::Pointer<BuilderLevel<usize, T>, usize>;

/// Hilfsdatenstruktur zum Bauen eines STrees (nötig wegen der perfekten Hashfunktionen, die zum Erzeugungszeitpunkt alle Schlüssel kennen müssen).
pub struct STreeBuilder<T> {
    /// Mit Hilfe der ersten 24-Bits des zu speichernden Wortes wird in `root_table` eine L2EbeneBuilder je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^24]`
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
                                second_level.hash_map.get_mut(&j).unwrap(),
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
                            std::mem::replace(&mut second_level.keys, vec![]).into_boxed_slice(),
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
                                let hm = std::mem::replace(&mut l2_level.hash_map, HashMap::new());
                                for (j, l3_level) in hm.into_iter() {
                                    if (*l).get(j).is_null() {
                                        let pointered_data = (*l).get(j);

                                        *pointered_data = match l3_level.get() {
                                            PointerEnum::First(l2) => {
                                                let l3_level = l2;
                                                let mut level = Level::new(
                                                    l3_level.lx_top.take().unwrap(),
                                                    vec![0; l3_level.keys.len()].into_boxed_slice(),
                                                    std::mem::replace(&mut l3_level.keys, vec![]).into_boxed_slice(),
                                                    l3_level.minimum,
                                                    l3_level.maximum,
                                                );
                                                let hm = std::mem::replace(&mut l3_level.hash_map, HashMap::new());
                                                for (k,val) in hm.into_iter() {
                                                    let result = level.get(k);
                                                    *result = val;
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
pub struct BuilderLevel<T, E> {
    /// Klassische HashMap zum aufbauen der perfekten Hashmap
    pub hash_map: HashMap<LXKey, T>,

    /// Eine Liste aller bisher gesammelter Schlüssel, die später auf die nächste Ebene zeigen.
    /// Diese werden zur Erzeugung der perfekten Hashfunktion benötigt.
    pub keys: Vec<LXKey>,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^8 (Bits) besitzt. Also [u64;2^8/64].
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    /// Dieses Array wird später an den `Level`-Struct weitergegeben
    lx_top: Option<TopArray<E, u8>>,

    /// Speichert das Maximum des Levels zwischen
    pub maximum: usize,

    /// Speichert das Minimum des Levels zwischen
    pub minimum: usize,
}

impl<T, E> BuilderLevel<T, E> {
    /// Gibt ein BuilderLevel<T> zurück.
    ///
    /// # Arguments
    ///
    /// * `lx_top_size` - Gibt die Länge des Arrays `lx_top_size` an.
    #[inline]
    pub fn new() -> BuilderLevel<T, E> {
        BuilderLevel {
            hash_map: HashMap::new(),
            keys: vec![],
            lx_top: Some(TopArray::new()),
            maximum: 0,
            minimum: 1,
        }
    }
}
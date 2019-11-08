use crate::default::immutable::{Int, L1Ebene, LXKey, Level, LevelPointer, TopArray, L2Ebene, L3Ebene, LXEbene, LYEbene};
use crate::internal::Splittable;
use crate::internal::{self, PointerEnum};

type HashMap<K, T> = hashbrown::hash_map::HashMap<K, T>;

/// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen,
/// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
pub const GAMMA: f64 = 2.0;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L1-Ebene hat.AsMut
type L1EbeneBuilder<T> = internal::Pointer<BuilderLevel<L2EbeneBuilder<T>, T>, usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L2-Ebene hat.AsMut
type L2EbeneBuilder<T> = internal::Pointer<BuilderLevel<LXEbeneBuilder<T>, T>, usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur LX-Ebene hat.AsMut
type LXEbeneBuilder<T> = internal::Pointer<BuilderLevel<LYEbeneBuilder<T>, T>, usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur LY-Ebene hat.AsMut
type LYEbeneBuilder<T> = internal::Pointer<BuilderLevel<L3EbeneBuilder<T>, T>, usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L3-Ebene hat.AsMut
type L3EbeneBuilder<T> = internal::Pointer<BuilderLevel<usize, T>, usize>;

/// Hilfsdatenstruktur zum Bauen eines STrees (nötig wegen der perfekten Hashfunktionen, die zum Erzeugungszeitpunkt alle Schlüssel kennen müssen).
pub struct STreeBuilder<T: 'static> {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2EbeneBuilder je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    root_table: Box<[L1EbeneBuilder<T>]>,

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
        let mut root_table: Box<[L1EbeneBuilder<T>]> =
            vec![internal::Pointer::null(); T::root_array_size()].into_boxed_slice();

        for (index, element) in elements.iter().enumerate() {
            let (i, l, j, x, y , k) = Splittable::split_integer_down(element);

            if !root_top.is_set(i as usize) {
                root_top.set_bit(i as usize);
                root_indexs.push(i);
            }

            if root_table[i].is_null() {
                root_table[i] = internal::Pointer::from_second(Box::new(index));
            } else {
                match root_table[i].get() {
                    PointerEnum::First(l1_object) => {
                        l1_object.maximum = index;

                        if !l1_object.lx_top.as_ref().unwrap().is_set(l as usize) {
                            l1_object.keys.push(l);

                            let mut l2_object = internal::Pointer::null();
                            Self::insert_l2_level(&mut l2_object, index, &elements, j, x, y, k);
                            l1_object.hash_map.insert(l, l2_object);
                            l1_object.lx_top.as_mut().unwrap().set_bit(l as usize);
                        } else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            Self::insert_l2_level(
                                l1_object.hash_map.get_mut(&l).unwrap(),
                                index,
                                &elements,
                                j, x, y, k
                            );
                        }
                    }

                    PointerEnum::Second(e) => {
                        let (_, l2, j2, x2, y2, k2) = Splittable::split_integer_down(&elements[*e]);
                        let mut l1_object = BuilderLevel::new();

                        l1_object.lx_top.as_mut().unwrap().set_bit(l as usize);
                        // Minima- und Maximasetzung auf der ersten Ebene
                        l1_object.minimum = *e;
                        l1_object.maximum = index;

                        let mut l2_object = internal::Pointer::null();

                        if l2 != l {
                            let mut l2_object = internal::Pointer::null();
                            Self::insert_l2_level(&mut l2_object, *e, &elements, j2, x2, y2, k2);

                            l1_object.keys.push(l2);
                            l1_object.hash_map.insert(l2, l2_object);
                            l1_object.lx_top.as_mut().unwrap().set_bit(l2 as usize)
                        } else {
                            Self::insert_l2_level(&mut l2_object, *e, &elements, j2, x2, y2, k2);
                        }

                        // Reihenfolge der keys ist relevant!
                        l1_object.keys.push(l);
                        Self::insert_l2_level(&mut l2_object, index, &elements, j, x, y, k);

                        l1_object.hash_map.insert(l, l2_object);

                        root_table[i] = internal::Pointer::from_first(Box::new(l1_object));
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
    fn insert_l3_level(l3_level: &mut L3EbeneBuilder<T>, index: usize, elements: &[T], k: LXKey,) {
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
                    let (_, _, _, _, _, k2) = Splittable::split_integer_down(&elements[*e]);
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

    pub fn insert_ly_level(ly_level: &mut LYEbeneBuilder<T>,index: usize, elements: &[T], y: u8, k: u8) {
        if ly_level.is_null() {
            *ly_level = LYEbeneBuilder::from_second(Box::new(index));
        } else {
            match ly_level.get() {
                PointerEnum::First(ly_object) => {
                    ly_object.maximum = index;

                    if !ly_object.lx_top.as_mut().unwrap().is_set(y as usize) {
                        let mut l3_level = L3EbeneBuilder::null();
                        Self::insert_l3_level(&mut l3_level,index,&elements, k);

                        ly_object.hash_map.insert(y,l3_level);
                        ly_object.lx_top.as_mut().unwrap().set_bit(y as usize);
                        ly_object.keys.push(y);
                    } else {
                        // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                        Self::insert_l3_level(ly_object.hash_map.get_mut(&y).unwrap(),index,&elements, k);
                    }
                },

                PointerEnum::Second(elem_index) => {
                    let mut ly_object = BuilderLevel::new();
                    let elem2 = elements[*elem_index];
                    let (_,_,_, _, y2, k2) = Splittable::split_integer_down(&elem2);
                    
                    // Da die Elemente sortiert sind
                    ly_object.minimum = *elem_index;
                    ly_object.maximum = index;

                    let mut l3_level = L3EbeneBuilder::null();

                    if y2 != y {
                        let mut l3_level = L3EbeneBuilder::null();
                        Self::insert_l3_level(&mut l3_level,*elem_index,&elements, k2);

                        ly_object.hash_map.insert(y2,l3_level);
                        ly_object.lx_top.as_mut().unwrap().set_bit(y2 as usize);
                        ly_object.keys.push(y2);
                    } else {
                        Self::insert_l3_level(&mut l3_level,*elem_index,&elements, k2);
                    }

                    ly_object.lx_top.as_mut().unwrap().set_bit(y as usize);
                    ly_object.keys.push(y);
                    Self::insert_l3_level(&mut l3_level,index,&elements, k);
                    ly_object.hash_map.insert(y,l3_level);

                    *ly_level = LYEbeneBuilder::from_first(Box::new(ly_object));
                }
            }
        }
    }

    pub fn insert_lx_level(lx_level: &mut LXEbeneBuilder<T>,index: usize, elements: &[T], x: u8, y: u8, k: u8) {
        if lx_level.is_null() {
            *lx_level = LXEbeneBuilder::from_second(Box::new(index));
        } else {
            match lx_level.get() {
                PointerEnum::First(lx_object) => {
                    lx_object.maximum = index;

                    if !lx_object.lx_top.as_mut().unwrap().is_set(x as usize) {
                        let mut ly_level = LYEbeneBuilder::null();
                        Self::insert_ly_level(&mut ly_level,index,&elements, y, k);

                        lx_object.hash_map.insert(x,ly_level);
                        lx_object.lx_top.as_mut().unwrap().set_bit(x as usize);
                        lx_object.keys.push(x);
                    } else {
                        // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                        Self::insert_ly_level(lx_object.hash_map.get_mut(&x).unwrap(),index,&elements, y, k);
                    }
                },

                PointerEnum::Second(elem_index) => {
                    let mut lx_object = BuilderLevel::new();
                    let elem2 = elements[*elem_index];
                    let (_,_,_, x2, y2, k2) = Splittable::split_integer_down(&elem2);
                    
                    // Da die Elemente sortiert sind
                    lx_object.minimum = *elem_index;
                    lx_object.maximum = index;

                    
                    let mut ly_level = LYEbeneBuilder::null();

                    if x2 != x {
                        let mut ly_level = LYEbeneBuilder::null();
                        Self::insert_ly_level(&mut ly_level,*elem_index,&elements, y2, k2);

                        lx_object.hash_map.insert(x2,ly_level);
                        lx_object.lx_top.as_mut().unwrap().set_bit(x2 as usize);
                        lx_object.keys.push(x2);
                    } else {
                        Self::insert_ly_level(&mut ly_level,*elem_index,&elements, y2, k2);
                    }

                    lx_object.lx_top.as_mut().unwrap().set_bit(x as usize);
                    lx_object.keys.push(x);
                    Self::insert_ly_level(&mut ly_level,index,&elements, y, k);
                    lx_object.hash_map.insert(x,ly_level);

                    *lx_level = LXEbeneBuilder::from_first(Box::new(lx_object));
                }
            }
        }
    }

    pub fn insert_l2_level(l2_level: &mut L2EbeneBuilder<T>,index: usize, elements: &[T], j: u8, x: u8, y: u8, k: u8 ) {
        if l2_level.is_null() {
            *l2_level = L2EbeneBuilder::from_second(Box::new(index));
        } else {
            match l2_level.get() {
                PointerEnum::First(l2_object) => {
                    l2_object.maximum = index;

                    if !l2_object.lx_top.as_mut().unwrap().is_set(j as usize) {
                        let mut lx_level = LXEbeneBuilder::null();
                        Self::insert_lx_level(&mut lx_level,index,&elements, x, y, k);

                        l2_object.hash_map.insert(j,lx_level);
                        l2_object.lx_top.as_mut().unwrap().set_bit(j as usize);
                        l2_object.keys.push(j);
                    } else {
                        // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                        Self::insert_lx_level(l2_object.hash_map.get_mut(&j).unwrap(),index,&elements, x, y, k);
                    }
                },

                PointerEnum::Second(elem_index) => {
                    let mut l2_object = BuilderLevel::new();
                    let elem2 = elements[*elem_index];
                    let (_,_,j2,x2,y2,k2) = Splittable::split_integer_down(&elem2);
                    
                    // Da die Elemente sortiert sind
                    l2_object.minimum = *elem_index;
                    l2_object.maximum = index;

                    let mut lx_level = LXEbeneBuilder::null();

                    if j2 != j {
                        let mut lx_level = LXEbeneBuilder::null();
                        Self::insert_lx_level(&mut lx_level,*elem_index,&elements, x2, y2, k2);

                        l2_object.hash_map.insert(j2,lx_level);
                        l2_object.lx_top.as_mut().unwrap().set_bit(j2 as usize);
                        l2_object.keys.push(j2);
                    } else {
                        Self::insert_lx_level(&mut lx_level,*elem_index,&elements, x2, y2, k2);
                    }

                    l2_object.lx_top.as_mut().unwrap().set_bit(j as usize);
                    l2_object.keys.push(j);
                    Self::insert_lx_level(&mut lx_level,index,&elements, x, y, k);
                    l2_object.hash_map.insert(j,lx_level);

                    *l2_level = L2EbeneBuilder::from_first(Box::new(l2_object));
                }
            }
        }
    }

    /// Baut ein Array `root_table` für den STree-Struct. Dabei werden zuerst die `Level`-Structs korrekt mittels neuer perfekter Hashfunktionen
    /// angelegt und miteinander verbunden. Nachdem die Struktur mit normalen Hashfunktionen gebaut wurde können nun perfekte Hashfunktionen berechnet
    /// werden!
    pub fn build(&mut self) -> Box<[L1Ebene<T>]> {
        let mut tmp: Vec<L1Ebene<T>> = Vec::with_capacity(T::root_array_size());
        // Die L2Level-Elemente werden angelegt. Hierbei wird direkt in der new()-Funktion die perfekte Hashfunktion berechnet
        for i in 0..tmp.capacity() {
            if self.root_table[i].is_null() {
                tmp.push(LevelPointer::from_null());
            } else {
                match self.root_table[i].get() {
                    PointerEnum::First(l1_object) => {
                        let objects: Vec<L2Ebene<T>> =
                            vec![LevelPointer::from_null(); l1_object.keys.len()];
                        let val = Box::new(Level::new(
                            l1_object.lx_top.take().unwrap(),
                            objects.into_boxed_slice(),
                            std::mem::replace(l1_object.keys.as_mut(), vec![]).into_boxed_slice(),
                            l1_object.minimum,
                            l1_object.maximum,
                        ));

                        tmp.push(LevelPointer::from_level(val));
                    }

                    PointerEnum::Second(e) => {
                        tmp.push(LevelPointer::from_usize(Box::new(*e)));
                    }
                }
            }
        }
        let result: Box<[L1Ebene<T>]> = tmp.into_boxed_slice();

        for &i in &self.root_indexs {
                match &mut result[i].get() {
                    PointerEnum::First(l1_object) => {
                        // Hier muss l2_level aufgrund der symmetrischen Befüllung auch == Ptr::Level sein.LevelPointerBuilder
                        match std::mem::replace(&mut self.root_table[i], L1EbeneBuilder::null())
                            .get()
                        {
                            PointerEnum::First(l1_object_builder) => {
                                let l1_object_hm = std::mem::replace(&mut l1_object_builder.hash_map, HashMap::new());
                                for (l, l2_builder) in l1_object_hm.into_iter() {
                                    let pointered_data = l1_object.get(l);
                                    Self::insert_l2_level_final(pointered_data, l2_builder);
                                }
                            }
                            _ => {

                            }
                        }
                    }
                    _ => {

                    }
                }
        }
        result
    }

    #[inline]
    fn insert_l2_level_final(content: &mut L2Ebene<T>, builder: L2EbeneBuilder<T>) {
        *content = match builder.get() {
            PointerEnum::First(builder) => {
                let mut l2_object = Level::new(
                    builder.lx_top.take().unwrap(),
                    vec![LXEbene::from_null(); builder.keys.len()].into_boxed_slice(),
                    std::mem::replace(builder.keys.as_mut(), vec![]).into_boxed_slice(),
                    builder.minimum,
                    builder.maximum,
                );
                let builder_hm = std::mem::replace(&mut builder.hash_map, HashMap::new());
                for (j,lx_builder) in builder_hm.into_iter() {
                    let pointered_data = l2_object.get(j);
                    Self::insert_lx_level_final(pointered_data, lx_builder);
                }
                L2Ebene::from_level(Box::new(l2_object))
            }
            PointerEnum::Second(e) => {
                LevelPointer::from_usize(Box::new(*e))
            }
        };
    }

    #[inline]
    fn insert_lx_level_final(content: &mut LXEbene<T>, builder: LXEbeneBuilder<T>) {
        *content = match builder.get() {
            PointerEnum::First(builder) => {
                let mut lx_object = Level::new(
                    builder.lx_top.take().unwrap(),
                    vec![LYEbene::from_null(); builder.keys.len()].into_boxed_slice(),
                    std::mem::replace(builder.keys.as_mut(), vec![]).into_boxed_slice(),
                    builder.minimum,
                    builder.maximum,
                );
                let builder_hm = std::mem::replace(&mut builder.hash_map, HashMap::new());
                for (x,ly_builder) in builder_hm.into_iter() {
                    let pointered_data = lx_object.get(x);
                    Self::insert_ly_level_final(pointered_data, ly_builder);
                }

                LXEbene::from_level(Box::new(lx_object))
            }
            PointerEnum::Second(e) => {
                LevelPointer::from_usize(Box::new(*e))
            }
        };
    }

    #[inline]
    fn insert_ly_level_final(content: &mut LYEbene<T>, builder: LYEbeneBuilder<T>) {
        *content = match builder.get() {
            PointerEnum::First(builder) => {
                let mut ly_object = Level::new(
                    builder.lx_top.take().unwrap(),
                    vec![L3Ebene::from_null(); builder.keys.len()].into_boxed_slice(),
                    std::mem::replace(builder.keys.as_mut(), vec![]).into_boxed_slice(),
                    builder.minimum,
                    builder.maximum,
                );
                let builder_hm = std::mem::replace(&mut builder.hash_map, HashMap::new());
                for (y,ly_builder) in builder_hm.into_iter() {
                    let pointered_data = ly_object.get(y);
                    Self::insert_l3_level_final(pointered_data, ly_builder);
                }

                LYEbene::from_level(Box::new(ly_object))
            }
            PointerEnum::Second(e) => {
                LevelPointer::from_usize(Box::new(*e))
            }
        };
    }

    #[inline]
    fn insert_l3_level_final(content: &mut L3Ebene<T>, builder: L3EbeneBuilder<T>) {
        *content = match builder.get() {
            PointerEnum::First(builder) => {
                let mut l3_object = Level::new(
                    builder.lx_top.take().unwrap(),
                    vec![0_usize; builder.keys.len()].into_boxed_slice(),
                    std::mem::replace(builder.keys.as_mut(), vec![]).into_boxed_slice(),
                    builder.minimum,
                    builder.maximum,
                );
                let builder_hm = std::mem::replace(&mut builder.hash_map, HashMap::new());
                for (y,val) in builder_hm.into_iter() {
                    let pointered_data = l3_object.get(y);
                    *pointered_data = val;
                }

                L3Ebene::from_level(Box::new(l3_object))
            }
            PointerEnum::Second(e) => {
                LevelPointer::from_usize(Box::new(*e))
            }
        };
    }

    pub fn get_root_top(&mut self) -> TopArray<T, usize> {
        self.root_top.take().unwrap()
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays.
#[derive(Clone)]
pub struct BuilderLevel<T: 'static, E: 'static> {
    /// Klassische HashMap zum aufbauen der perfekten Hashmap
    pub hash_map: HashMap<LXKey, T>,

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
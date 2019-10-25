use crate::internal::{Splittable};
use crate::default::immutable::{Level, L2Ebene, Int, LevelPointer};

type HashMap<K,T> = hashbrown::hash_map::HashMap<K,T>;

/// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen, 
/// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
pub const GAMMA: f64 = 2.0;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L2-Ebene hat.AsMut
type L2EbeneBuilder = internal::Pointer<BuilderLevel<L3EbeneBuilder>,usize>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L3-Ebene hat.AsMut
type L3EbeneBuilder = internal::Pointer<BuilderLevel<usize>,usize>;

/// Hilfsdatenstruktur zum Bauen eines STrees (nötig wegen der perfekten Hashfunktionen, die zum Erzeugungszeitpunkt alle Schlüssel kennen müssen).
pub struct STreeBuilder {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2EbeneBuilder je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    root_table: Box<[L2EbeneBuilder]>,

    /// Root-Top-Array
    root_top: Box<[Box<[u64]>]>,

    /// Eine Liste, die alle belegten Indizes von `root_table` speichert. 
    root_indexs: Vec<usize>,
}

impl STreeBuilder {
    /// Gibt einen STreeBuilder mit den in `elements` enthaltenen Werten zurück. Dabei werden normale Hashfunktionen verwendet.
    ///
    /// # Arguments
    ///
    /// * `elements` - Eine Liste mit sortierten u40-Werten, die in die statische Datenstruktur eingefügt werden sollten. Kein Wert darf doppelt vorkommen! 
    pub fn new<T: Int>(elements: Box<[T]>) ->  Self{
        let mut root_indexs = vec![];
                
        // root_top_deep verwenden um die richtige Tiefe von root_top zu bestimmen
        let mut root_top_deep = 0;
        while T::root_array_size()/(1<<root_top_deep*6) > 256 {
            root_top_deep +=1;
        }
   
        let mut root_top: Vec<Box<Vec<u64>>> = Vec::with_capacity(root_top_deep);

        for i in 0..root_top.capacity() {
            root_top.push(Box::new(vec![0;T::root_array_size()/(1<<i*6)]));
        }

        let mut root_top: Box<[Box<[u64]>]> = root_top.into_iter().map(|x| x.into_boxed_slice()).collect::<Vec<_>>().into_boxed_slice();

        // Hier wird ein root_array der Länge T::root_array_size() angelegt, was 2^i entspricht. Dabei entspricht bei einem u40 Integer i=40 .
        let mut root_table: Box<[L2EbeneBuilder]> = vec![internal::Pointer::null(); T::root_array_size()].into_boxed_slice();
    
        for (index,element) in elements.iter().enumerate() {
            let (i,j,k) = Splittable::split_integer_down(element);

            if !root_indexs_contains(&root_top, i) {
                Self::build_root_top(&mut root_top, &i);
                root_indexs.push(i);
            }
            
            if root_table[i].is_null() {
                root_table[i] = internal::Pointer::from_second(Box::new(index));
            } else {
                match root_table[i].get() {
                    PointerEnum::First(l) => {
                        let second_level = l;
                        second_level.maximum = index;

                        if !second_level.contains(j) {
                            second_level.keys.push(j);

                            let mut l3_level = internal::Pointer::null();
                            Self::insert_l3_level(&mut l3_level,index,k,&elements);

                            second_level.hash_map.insert(j,l3_level);
                            Self::build_lx_top(&mut second_level.lx_top, j);
                        }
                        else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            Self::insert_l3_level(second_level.hash_map.get_mut(&j).unwrap(),index,k,&elements);
                        }
                    },

                    PointerEnum::Second(e) => {
                        let (_,j2,k2) = Splittable::split_integer_down(&elements[*e]);
                        let lx_array_size  = 1_usize<<(((std::mem::size_of::<T>()*8)/2)/2); 
                        let mut second_level = BuilderLevel::new(lx_array_size/64);

                        Self::build_lx_top(&mut second_level.lx_top, j);
                        // Minima- und Maximasetzung auf der ersten Ebene
                        second_level.minimum = *e;
                        second_level.maximum = index;

                        let mut l3_level = internal::Pointer::null();

                        if j2 != j {
                            let mut l3_level = internal::Pointer::null();
                            Self::insert_l3_level(&mut l3_level,*e,k2,&elements);

                            second_level.keys.push(j2);
                            second_level.hash_map.insert(j2,l3_level);
                            Self::build_lx_top(&mut second_level.lx_top, j2);
                        } else {
                            Self::insert_l3_level(&mut l3_level,*e,k2,&elements);
                        }

                        // Reihenfolge der keys ist relevant!
                        second_level.keys.push(j);
                        Self::insert_l3_level(&mut l3_level,index,k,&elements);
                        second_level.hash_map.insert(j,l3_level);

                        root_table[i] = internal::Pointer::from_first(Box::new(second_level));

                    }
                }
            }
        }
        Self {root_table: root_table, root_top: root_top, root_indexs: root_indexs}
    }
    #[inline]
    fn insert_l3_level<T: Int + Into<u64>>(l3_level: &mut L3EbeneBuilder,index: usize, k: u16, elements: &[T]) {
        let lx_array_size  = 1_usize<<(((std::mem::size_of::<T>()*8)/2)/2); 

        if l3_level.is_null() {
            *l3_level = internal::Pointer::from_second(Box::new(index));
        } else {
            match l3_level.get() {
                PointerEnum::First(l) => {
                    let l3_level = l;

                    debug_assert!(!l3_level.contains(k));
                    Self::build_lx_top(&mut l3_level.lx_top, k);
                    l3_level.keys.push(k);
                
                    //Maximasetzung auf der zweiten Ebene
                    l3_level.maximum = index;

                    l3_level.hash_map.insert(k, index);
                },

                PointerEnum::Second(e) => {
                    let (_,_,k2) = Splittable::split_integer_down(&elements[*e]);
                    let mut l3_level_n = BuilderLevel::new(lx_array_size/64);
                    l3_level_n.keys.push(k2);
                    l3_level_n.keys.push(k);

                    debug_assert!(k2!=k);

                     // Minima- und Maximasetzung auf der zweiten Ebene
                    l3_level_n.minimum = *e;
                    l3_level_n.maximum = index;

                    l3_level_n.hash_map.insert(k2, *e);
                    l3_level_n.hash_map.insert(k, index);
                    Self::build_lx_top(&mut l3_level_n.lx_top, k);
                    Self::build_lx_top(&mut l3_level_n.lx_top, k2);
                    
                    *l3_level = internal::Pointer::from_first(Box::new(l3_level_n));
                }
            }
        }
    }

    /// Baut ein Array `root_table` für den STree-Struct. Dabei werden zuerst die `Level`-Structs korrekt mittels neuer perfekter Hashfunktionen
    /// angelegt und miteinander verbunden. Nachdem die Struktur mit normalen Hashfunktionen gebaut wurde können nun perfekte Hashfunktionen berechnet 
    /// werden!
    pub fn build<T: Int>(&mut self) -> Box<[L2Ebene]> {
        let mut tmp: Vec<L2Ebene> = Vec::with_capacity(T::root_array_size());
        // Die L2Level-Elemente werden angelegt. Hierbei wird direkt in der new()-Funktion die perfekte Hashfunktion berechnet
        for i in 0..tmp.capacity() {
            if self.root_table[i].is_null() {
                tmp.push(LevelPointer::from_null());
            } else {
                match self.root_table[i].get() {
                    PointerEnum::First(l) => {
                        let second_level = l;

                        let val = Box::new(Level::new(std::mem::replace(&mut second_level.lx_top, Box::new([])), vec![LevelPointer::from_null(); second_level.keys.len()].into_boxed_slice(), second_level.clone().keys.into_boxed_slice(),second_level.minimum, second_level.maximum));
                        tmp.push(LevelPointer::from_level(val));
                    },

                    PointerEnum::Second(e) => {
                        tmp.push(LevelPointer::from_usize(Box::new(*e)));
                    }
                }
            }
        }
        let result: Box<[L2Ebene]> = tmp.into_boxed_slice();

        for &i in &self.root_indexs {
            // L3-Level werden nur angelegt, falls mehr als 1 Wert in der DS existiert.
            if !result[i].is_null() {
                match &mut result[i].get() {
                    PointerEnum::First(l) => {
                        // Hier muss l2_level aufgrund der symmetrischen Befüllung auch == Ptr::Level sein.LevelPointerBuilder
                        match std::mem::replace(&mut self.root_table[i],L2EbeneBuilder::null()).get() {
                            PointerEnum::First(l2) => {
                                let l2_level = l2;

                                for &j in &l2_level.keys {
                                    // Die L2-Top-Tabellen werden gefüllt und die 
                                    let l3_level = l2_level.hash_map.get_mut(&j).unwrap();
                                    // TODO 
                                    if (*l).get(j).is_null() {
                                        let pointered_data = (*l).get(j);

                                        *pointered_data =  match l3_level.get() {
                                            PointerEnum::First(l2) => {
                                                let l3_level = l2;
                                                let mut level = Level::new(std::mem::replace(&mut l3_level.lx_top, Box::new([])), vec![0; l3_level.keys.len()].into_boxed_slice(), l3_level.keys.clone().into_boxed_slice(),l3_level.minimum,l3_level.maximum);
                                                for k in &l3_level.keys {
                                                    let result = level.get(*k);
                                                    *result = *l3_level.hash_map.get(k).unwrap();
                                                }
                                                
                                                LevelPointer::from_level(Box::new(level))
                                            },
                                            PointerEnum::Second(e) => {
                                                LevelPointer::from_usize(Box::new(*e))
                                            }
                                        };
                                    }
                                }

                            }
                            _ => {

                            }
                        }
            
                    },

                    _ => {
                        
                    }
                }
            }
 
        }
        result
    }

    /// Hilfsfunktion zum erzeugen der LX-Top-Arrays. 
    /// Annahme: Größe des lx_top-Arrays 2^10 Elemente
    /// 
    /// # Arguments
    ///
    /// * `lx_top` - Mutable Referenz auf ein Array, das nach diesem Funktionsaufruf das Bit für `key` gesetzt hat. 
    /// * `key` - Ein Schlüssel, dessen Index als Bit im LX-Top-Array gesetzt wird. 
    #[inline]
    fn build_lx_top(lx_top: &mut Box<[u64]>, key: u16) {
        let key = u16::from(key);

        let index = (key/64) as usize;
        let in_index_mask = 1<<(63-(key % 64));
        lx_top[index] = lx_top[index] | in_index_mask;
    }

    /// Baut das Root-Top-Array mit Hilfe der sich in der Datenstruktur befindenden Werte.
    #[inline]
    fn build_root_top(root_top: &mut Box<[Box<[u64]>]>, bit: &usize) {
        // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
        for i in 0..root_top.len() {
            let curr_bit_repr = bit/(1<<(i*6));
            let index = curr_bit_repr/64;
            let bit_mask: u64  = 1<<(63-(curr_bit_repr%64));
            root_top[i][index] = root_top[i][index] | bit_mask;
        }
    }

    pub fn get_root_tops(&mut self) -> (Box<[Box<[u64]>]>) {
        (std::mem::replace(&mut self.root_top,Box::new([])))
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays. 
#[derive(Clone)]
pub struct BuilderLevel<T: 'static> {
    /// Klassische HashMap zum aufbauen der perfekten Hashmap
    pub hash_map: BuildHM<u16,T>,

    /// Eine Liste aller bisher gesammelter Schlüssel, die später auf die nächste Ebene zeigen.
    /// Diese werden zur Erzeugung der perfekten Hashfunktion benötigt.
    pub keys: Vec<u16>,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64]. 
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    /// Dieses Array wird später an den `Level`-Struct weitergegeben
    pub lx_top: Box<[u64]>,

    /// Speichert das Maximum des Levels zwischen
    pub maximum: usize,

    /// Speichert das Minimum des Levels zwischen
    pub minimum: usize,
}

impl<T> BuilderLevel<T> {
    /// Gibt ein BuilderLevel<T> zurück. 
    ///
    /// # Arguments
    ///
    /// * `lx_top_size` - Gibt die Länge des Arrays `lx_top_size` an. 
    #[inline]
    pub fn new(lx_top_size: usize) -> BuilderLevel<T> {
        BuilderLevel {
            hash_map: BuildHM::new(),
            keys: vec![],
            lx_top: vec![0;lx_top_size].into_boxed_slice(),
            maximum: 0,
            minimum: 1
        }
    }

    pub fn contains(&self, key: u16) -> bool {
        let index = (key/64) as usize;
        let in_index_mask = 1<<(63-(key % 64));

        (self.lx_top[index] & in_index_mask) != 0
    }
}

// ------------------------- Pointer Magie, zum Verhindern der Nutzung von HashMaps für kleine Datenmengen ----------------------------------

use crate::internal::{self, PointerEnum};
pub struct BuildHM<K,T> {
    pointer: internal::Pointer<HashMap<K,T>,(Box<Vec<K>>,Box<Vec<T>>)>,
}

impl<K:'static + Clone,T:'static + Clone> Clone for BuildHM<K,T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone()
        }
    }
}

impl<K:'static + Eq + Copy + Ord + std::hash::Hash,T: 'static> BuildHM<K,T> {
    fn new() -> Self{
        Self {
            pointer: internal::Pointer::from_second(Box::new((Box::new(vec![]),Box::new(vec![]))))
        }
    }

    /// Die eigentliche Updatemechanik der HashMaps, wird hier ignoriert, da keine Werte geupdatet werden müssen!
    fn insert(&mut self, key: K, val: T) {
        match self.pointer.get() {
            PointerEnum::Second((keys,values)) => {
                if true {
                    keys.push(key);
                    values.push(val);
                } else {
                    let mut hm = HashMap::<K,T>::with_capacity(513);
                  //  let values = std::mem::replace(values, Box::new(vec![]));
                 //   for (i,val) in values.into_iter().enumerate() {
                  //      hm.insert(keys[i], val);
                  //  }
                    hm.insert(key, val);
                    self.pointer = internal::Pointer::from_first(Box::new(hm));
                }
            },
            PointerEnum::First(x) => {
                x.insert(key, val);
            },
        }
    }

    fn get_mut(&mut self, k: &K) -> Option<&mut T> {
        match self.pointer.get() {
            PointerEnum::Second((keys,values)) => {
                match keys.binary_search(k) {
                    Ok(x) => values.get_mut(x),
                    Err(_) => None,
                }
            },
            PointerEnum::First(x) => {
                x.get_mut(k)
            },
        }
    }

    fn get(&mut self, k: &K) -> Option<&T> {
        match self.pointer.get() {
            PointerEnum::Second((keys,values)) => {
                match keys.binary_search(k) {
                    Ok(x) => values.get(x),
                    Err(_) => None,
                }
            },
            PointerEnum::First(x) => {
                x.get(k)
            },
        }
    }
}

fn root_indexs_contains(root_top: &Box<[Box<[u64]>]>, bit: usize) -> bool {
    let curr_bit_repr = bit/(1<<(0*6));
    let index = curr_bit_repr/64;
    let bit_mask: u64  = 1<<(63-(curr_bit_repr%64));
    if root_top[0][index] & bit_mask == 0 {
        return false;
    }

    true
}

//-------------------------------------------------------------------------------------------------------------------------------------------------------------------
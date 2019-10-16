use crate::internal::{Splittable};
use crate::default::immutable::{Level, L2Ebene, Int, LevelPointer};

type HashMap<K,T> = std::collections::HashMap<K,T>;

/// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen, 
/// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
pub const GAMMA: f64 = 2.0;

/// Die Länge der L2- und L3-Top-Arrays, des STrees (basierend auf 40-Bit /2/2.).
const LX_ARRAY_SIZE: usize = 1 << 10;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L2-Ebene hat.AsMut
type L2EbeneBuilder = LevelPointerBuilder<L3EbeneBuilder>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L3-Ebene hat.AsMut
type L3EbeneBuilder = LevelPointerBuilder<usize>;

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
        let mut root_table: Box<[L2EbeneBuilder]> = vec![LevelPointerBuilder::from_null(); T::root_array_size()].into_boxed_slice();
    
        for (index,element) in elements.iter().enumerate() {
            let (i,j,k) = Splittable::split_integer_down(element);

            if !root_indexs.contains(&i) {
                Self::build_root_top(&mut root_top, &i);
                root_indexs.push(i);
            }
            
            if root_table[i].is_null() {
                root_table[i] = LevelPointerBuilder::from_usize(Box::new(index));
            } else {
                match root_table[i].get() {
                    PointerBuilder::Level(l) => {
                        let second_level = l;
                        second_level.maximum = index;

                        if !second_level.keys.contains(&j) {
                            second_level.keys.push(j);

                            let mut l3_level = LevelPointerBuilder::from_null();
                            Self::insert_l3_level(&mut l3_level,index,k,&elements);

                            second_level.hash_map.insert(j,l3_level);
                        }
                        else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            Self::insert_l3_level(second_level.hash_map.get_mut(&j).unwrap(),index,k,&elements);
                        }
                    },

                    PointerBuilder::Element(e) => {
                        let (_,j2,k2) = Splittable::split_integer_down(&elements[*e]);
                        let mut second_level = BuilderLevel::new(LX_ARRAY_SIZE/64);
                        second_level.keys.push(j);

                        // Minima- und Maximasetzung auf der ersten Ebene
                        second_level.minimum = *e;
                        second_level.maximum = index;

                        let mut l3_level = LevelPointerBuilder::from_null();

                        if j2 != j {
                            let mut l3_level = LevelPointerBuilder::from_null();
                            Self::insert_l3_level(&mut l3_level,*e,k2,&elements);

                            second_level.keys.push(j2);
                            second_level.hash_map.insert(j2,l3_level);
                        } else {
                            Self::insert_l3_level(&mut l3_level,*e,k2,&elements);
                        }
                        Self::insert_l3_level(&mut l3_level,index,k,&elements);
                        second_level.hash_map.insert(j,l3_level);

                        root_table[i] = LevelPointerBuilder::from_level(Box::new(second_level));

                    }
                }
            }
        }
        Self {root_table: root_table, root_top: root_top, root_indexs: root_indexs}
    }
    #[inline]
    fn insert_l3_level<T: Int + Into<u64>>(l3_level: &mut L3EbeneBuilder,index: usize, k: u16, elements: &[T]) {
        if l3_level.is_null() {
            *l3_level = LevelPointerBuilder::from_usize(Box::new(index));
        } else {
            match l3_level.get() {
                PointerBuilder::Level(l) => {
                    let l3_level = l;

                    assert!(!l3_level.keys.contains(&k));

                    l3_level.keys.push(k);
                
                    //Maximasetzung auf der zweiten Ebene
                    l3_level.maximum = index;

                    l3_level.hash_map.insert(k, index);
                },

                PointerBuilder::Element(e) => {
                    let (_,_,k2) = Splittable::split_integer_down(&elements[*e]);
                    let mut l3_level_n = BuilderLevel::new(LX_ARRAY_SIZE/64);
                    l3_level_n.keys.push(k);
                    l3_level_n.keys.push(k2);

                    assert!(k2!=k);

                     // Minima- und Maximasetzung auf der zweiten Ebene
                    l3_level_n.minimum = *e;
                    l3_level_n.maximum = index;

                    l3_level_n.hash_map.insert(k, index);
                    l3_level_n.hash_map.insert(k2, *e);
                    *l3_level = LevelPointerBuilder::from_level(Box::new(l3_level_n));
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
                    PointerBuilder::Level(l) => {
                        let second_level = l;
                        let val = Box::new(Level::new(LX_ARRAY_SIZE/64,Some(vec![LevelPointer::from_null(); second_level.keys.len()].into_boxed_slice()), Some(&second_level.keys),second_level.minimum, second_level.maximum));
                        tmp.push(LevelPointer::from_level(val));
                    },

                    PointerBuilder::Element(e) => {
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
                        match std::mem::replace(&mut self.root_table[i],L2EbeneBuilder::from_null()).get() {
                            PointerBuilder::Level(l2) => {
                                let l2_level = l2;

                                for &j in &l2_level.keys {
                                    Self::build_lx_top(&mut (*l).lx_top, j);
                                    
                                    // Die L2-Top-Tabellen werden gefüllt und die 
                                    let l3_level = l2_level.hash_map.get_mut(&j).unwrap();
                                    // TODO 
                                    if (*l).get(j).is_null() {
                                        let hash = (*l).hash_function.as_ref().unwrap().try_hash(&j).unwrap() as usize;
                                        (*l).objects[hash] =  match l3_level.get() {
                                            PointerBuilder::Level(l2) => {
                                                let l3_level = l2;
                                                let mut level = Level::new(LX_ARRAY_SIZE/64, Some(vec![0; l3_level.keys.len()].into_boxed_slice()), Some(&l3_level.keys),l3_level.minimum,l3_level.maximum);
                                                for k in &l3_level.keys {
                                                    Self::build_lx_top(&mut level.lx_top, *k);
                                                    let result = level.get(*k);
                                                    *result = *l3_level.hash_map.get(k).unwrap();
                                                }
                                                
                                                LevelPointer::from_level(Box::new(level))
                                            },
                                            PointerBuilder::Element(e) => {
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
}

pub enum PointerBuilder<T: 'static> {
    Level(&'static mut BuilderLevel<T>),
    Element(&'static mut usize)
}

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein usize-Objekt zeigt (Index aus Elementliste),
/// oder auf ein Levelobjekt
#[derive(Clone)]
pub struct LevelPointerBuilder<T: 'static> {
    pointer: *mut BuilderLevel<T>
}

impl<T> Drop for LevelPointerBuilder<T> {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }

        if (self.pointer as usize % 4) == 0 {
            unsafe { Box::from_raw(self.pointer) };
        } else {
            assert!((self.pointer as usize % 4) == 1);

            unsafe { Box::from_raw((self.pointer as usize -1) as *mut usize) };
        }
    }
}

impl<T: 'static> LevelPointerBuilder<T> {
    pub fn get(&self) -> PointerBuilder<T> {
        if self.pointer.is_null() {
            panic!("LevelPointer<T> is null!");
        }

        if (self.pointer as usize % 4) == 0 {
            unsafe {PointerBuilder::Level(&mut (*self.pointer))}
        } else {
            assert!((self.pointer as usize % 4) == 1);

            unsafe {PointerBuilder::Element(&mut *((self.pointer as usize -1) as *mut usize))}
        }
    }

    pub fn from_level(level_box: Box<BuilderLevel<T>>) -> Self {
        Self {
            pointer: Box::into_raw(level_box)
        }
    }

    pub fn from_null() -> Self {
        Self {
            pointer: std::ptr::null_mut()
        }
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    pub fn from_usize(usize_box: Box<usize>) -> Self {
        let pointer = Box::into_raw(usize_box);
        assert!((pointer as usize % 4) == 0);

        let pointer = (pointer as usize + 1) as *mut BuilderLevel<T>;
        Self {
            pointer: pointer
        }
    }
}



// ------------------------- Pointer Magie, zum Verhindern der Nutzung von HashMaps für kleine Datenmengen ----------------------------------

use crate::internal::{self, PointerEnum};
pub struct BuildHM<K,T> {
    pointer: internal::Pointer<HashMap<K,T>,Vec<(K,T)>>,
}

impl<K:'static + Clone,T:'static + Clone> Clone for BuildHM<K,T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone()
        }
    }
}

impl<K:'static + Eq + Ord + std::hash::Hash,T: 'static> BuildHM<K,T> {
    fn new() -> Self{
        Self {
            pointer: internal::Pointer::from_second(Box::new(Vec::<(K,T)>::new()))
        }
    }

    /// Die eigentliche Updatemechanik der HashMaps, wird hier ignoriert, da keine Werte geupdatet werden müssen!
    fn insert(&mut self, key: K, val: T) {
        match self.pointer.get() {
            PointerEnum::Second(x) => {
                if x.len() <= 1023 {
                    x.push((key,val));
                } else {
                    let mut hm = HashMap::<K,T>::with_capacity(1025);
                    let x = std::mem::replace(x, vec![]);
                    for val in x.into_iter() {
                        hm.insert(val.0, val.1);
                    }
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
            PointerEnum::Second(x) => {
                let mut l = 0;
                let mut r = x.len()-1;

                while l != r && x[l].0 != *k && x[r].0 != *k{
                    let m = (l+r)/2;
                    if *k == x[m].0 {
                        return Some(&mut x[m].1);
                    } else if *k > x[m].0 {
                        l = m+1;
                    } else {
                        r = m-1;
                    }
                }

                if x[l].0 == *k  {
                    Some(&mut x[l].1)
                } else {
                    Some(&mut x[r].1)
                }
            },
            PointerEnum::First(x) => {
                x.get_mut(k)
            },
        }
    }

    fn get(&mut self, k: &K) -> Option<&T> {
        match self.pointer.get() {
            PointerEnum::Second(x) => {
                let mut l = 0;
                let mut r = x.len()-1;

                while l != r && x[l].0 != *k && x[r].0 != *k{
                    let m = (l+r)/2;
                    if *k == x[m].0 {
                        return Some(&x[m].1);
                    } else if *k > x[m].0 {
                        l = m+1;
                    } else {
                        r = m-1;
                    }
                }

                if x[l].0 == *k  {
                    Some(&x[l].1)
                } else {
                    Some(&x[r].1)
                }
            },
            PointerEnum::First(x) => {
                x.get(k)
            },
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------------------------------------------------------
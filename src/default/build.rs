use boomphf::Mphf;

use crate::internal::{Splittable};
use crate::default::immutable::{Level, L2Ebene, L3Ebene, Int, LevelPointer, Pointer};

type HashMap<T,K> = std::collections::HashMap<T,K>;
#[derive(Clone)]
pub enum HashMapEnum<T,K> {
    None,
    One((T,K)),
    Some(HashMap<T,K>)
}

/// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen, 
/// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
pub const GAMMA: f64 = 2.0;

/// Die Länge der L2- und L3-Top-Arrays, des STrees (basierend auf 40-Bit /2/2.).
const LX_ARRAY_SIZE: usize = 1 << 10;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L2-Ebene hat.AsMut
type L2EbeneBuilder = BuilderLevel<L3EbeneBuilder>;

/// Hilfsebene, die eine sehr starke Ähnlichkeit zur L3-Ebene hat.AsMut
type L3EbeneBuilder = BuilderLevel<usize>;

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
    pub fn new<T: Int>(elements: Vec<T>) ->  Self{
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
        let mut root_table: Box<[L2EbeneBuilder]> = vec![L2EbeneBuilder::new(LX_ARRAY_SIZE/64);T::root_array_size()].into_boxed_slice();
    
        for (index,element) in elements.iter().enumerate() {
            let (i,j,k) = Splittable::split_integer_down(element);

            if !root_indexs.contains(&i) {
                Self::build_root_top(&mut root_top, &i);
                root_indexs.push(i);
            }

            let l2_level = &mut root_table[i];
            // Minima- und Maximasetzung auf der ersten Ebene
            if l2_level.minimum > l2_level.maximum {
                l2_level.minimum = index;
            }
            l2_level.maximum = index;
            
            match &mut root_table[i].hash_map  {
                // Wenn noch kein Element existiert, wird ein One, anstelle einer hashmap angelegt
                HashMapEnum::None => {
                    root_table[i].keys.push(j);
                    let mut l3_level = L3EbeneBuilder::new(LX_ARRAY_SIZE/64);
                    Self::insert_l3_level(&mut l3_level,index,k);
                    root_table[i].hash_map = HashMapEnum::One((j,l3_level));
                },
                // Falls bereits genau ein Element ext. wird eine Hashmap angelegt
                HashMapEnum::One((key,x)) => {
                    if *key != j {
                        root_table[i].keys.push(j);

                        let mut l3_level = L3EbeneBuilder::new(LX_ARRAY_SIZE/64);
                        let mut hash_map = HashMap::default();
                        Self::insert_l3_level(&mut l3_level,index,k);

                        hash_map.insert(j,l3_level);
                        let key = std::mem::replace(key, 0);
                        let x = std::mem::replace(x, L3EbeneBuilder::new(0));
                        
                        hash_map.insert(key,x);
                        root_table[i].hash_map = HashMapEnum::Some(hash_map);
                    } else {
                        // Fall nur bei Duplikaten
                        Self::insert_l3_level(x,index,k);
                    }
                },
                HashMapEnum::Some(x) => {
                    if !root_table[i].keys.contains(&j) {
                        root_table[i].keys.push(j);

                        let mut l3_level = L3EbeneBuilder::new(LX_ARRAY_SIZE/64);
                        Self::insert_l3_level(&mut l3_level,index,k);

                        x.insert(j,l3_level);
                    }
                    else {
                        // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen
                        Self::insert_l3_level(x.get_mut(&j).unwrap(),index,k);
                    }
                }
            }
        }
        Self {root_table: root_table, root_top: root_top, root_indexs: root_indexs}
    }
    #[inline]
    fn insert_l3_level(l3_level: &mut L3EbeneBuilder,index: usize, k: u16) {
        // Minima- und Maximasetzung auf der zweiten Ebene
        if l3_level.minimum > l3_level.maximum {
            l3_level.minimum = index;
        }
        l3_level.maximum = index;

        // Hier ist keine Prüfung notwendig, da die Elemente einmalig sind.
        // Prüfung wird trotzdem gemacht. Um auch bei falscher Eingabe noch lauffähig zu sein.
        if !l3_level.keys.contains(&k) {
            l3_level.keys.push(k);
            match &mut l3_level.hash_map {
                HashMapEnum::None => {
                    l3_level.hash_map = HashMapEnum::One((k,index));
                },
                HashMapEnum::One((key,x)) => {
                    let mut hash_map = HashMap::default();
                    let key = std::mem::replace(key, 0);
                    let x = std::mem::replace(x, 0);
                    hash_map.insert(key,x);
                    hash_map.insert(k,index);

                    l3_level.hash_map = HashMapEnum::Some(hash_map);
                },
                HashMapEnum::Some(x) => {
                    x.insert(k,index);
                }
            }
        }
    }

    /// Baut ein Array `root_table` für den STree-Struct. Dabei werden zuerst die `Level`-Structs korrekt mittels neuer perfekter Hashfunktionen
    /// angelegt und miteinander verbunden. Nachdem die Struktur mit normalen Hashfunktionen gebaut wurde können nun perfekte Hashfunktionen berechnet 
    /// werden!
    pub fn build<T: Int>(&mut self) -> Box<[Option<L2Ebene>]> {
        let mut tmp: Vec<Option<L2Ebene>> = Vec::with_capacity(T::root_array_size());
        // Die L2Level-Elemente werden angelegt. Hierbei wird direkt in der new()-Funktion die perfekte Hashfunktion berechnet
        for i in 0..tmp.capacity() {
            if self.root_table[i].minimum == self.root_table[i].maximum {
                tmp.push(Some(LevelPointer::from_usize(Box::new(self.root_table[i].minimum))));
            } else if self.root_table[i].maximum < self.root_table[i].minimum {
                tmp.push(None);
            } else {
                let val = Box::new(Level::new(LX_ARRAY_SIZE/64, Some(&self.root_table[i].keys),self.root_table[i].minimum, self.root_table[i].maximum));
                tmp.push(Some(LevelPointer::from_level(val)));
            }
        }
        let result: Box<[Option<L2Ebene>]> = tmp.into_boxed_slice();

        for &i in &self.root_indexs {
            // L3-Level werden nur angelegt, falls mehr als 1 Wert in der DS existiert.
            if !result[i].is_none() {
                match result[i].as_ref().unwrap().get() {
                    Pointer::Level(l) => {
                        let l2_level = &mut self.root_table[i];

                        // Die leeren L3Level-Elemente auf die L2 später zeigt werden angelegt
                        (*l).objects = Vec::with_capacity(l2_level.keys.len());
                        for _ in &l2_level.keys {
                            let val = Box::new(Level::new(LX_ARRAY_SIZE/64, None, 1, 0));
                            (*l).objects.push(LevelPointer::from_level(val));
                        }

                        for &j in &l2_level.keys {
                            // Die L2-Top-Tabellen werden gefüllt und die 
                            let l3_level = match &mut l2_level.hash_map {
                                HashMapEnum::One((_,x)) => {
                                    x
                                },
                                HashMapEnum::Some(x) => {
                                    x.get_mut(&j).unwrap()
                                },
                                _ => {
                                    panic!("Die HashMap des root_arrays im Builder Struct fehlt!");
                                }
                            };

                            Self::build_lx_top(&mut (*l).lx_top, j);
                            let keys = l3_level.keys.as_ref();
                            
                            
                            // Die L3-Elemente bekommen die Symantik aus dem L3BuilderStruct und die perfekte Hashfunktion wird berechnet
                            match (*l).get(j).get() {
                                Pointer::Level(l2) => {
                                    (*l2).minimum = l3_level.minimum;
                                    (*l2).maximum = l3_level.maximum
                                },

                                _ => {
                                   
                                }
                            }


                            // Verhindert das Anlegen einer Hashfunktion, wenn nur ein Element existiert
                            if l3_level.minimum == l3_level.maximum {
                                (*l).get(j).change_to_usize(Box::new(l3_level.minimum));
                            } else {
                                match (*l).get(j).get() {
                                    Pointer::Level(l2) => {
                                        let third_level = l2;
                                        third_level.hash_function = Some(Mphf::new_parallel(GAMMA,keys, None));

                                        // Die leeren usizes, die auf die Element-Liste zeigen werden angelegt
                                        third_level.objects = Vec::with_capacity(l3_level.keys.len());

                                        
                                        for _ in &l3_level.keys {
                                            third_level.objects.push(0);
                                        }
                                        // Die usizes werden sinnvoll belegt + die L3-Top-Tabellen werden gefüllt
                                        for &k in &l3_level.keys {
                                            Self::build_lx_top(&mut third_level.lx_top,k);
                                            let result = third_level.get(k);
                                            *result = match &l3_level.hash_map {
                                                HashMapEnum::One((_,x)) => {
                                                    *x
                                                },
                                                HashMapEnum::Some(x) => {
                                                    *x.get(&k).unwrap()
                                                },
                                                _ => {
                                                    panic!("Die HashMap des root_arrays im Builder Struct fehlt!");
                                                }
                                            };
                                        }   

                                    },
                                    _ => {
                                        panic!("Dead Code");
                                    }
                                }
   
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
    fn build_lx_top(lx_top: &mut Vec<u64>, key: u16) {
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
pub struct BuilderLevel<T> {
    /// Klassische HashMap zum aufbauen der perfekten Hashmap
    pub hash_map: HashMapEnum<u16,T>,

    /// Eine Liste aller bisher gesammelter Schlüssel, die später auf die nächste Ebene zeigen.
    /// Diese werden zur Erzeugung der perfekten Hashfunktion benötigt.
    pub keys: Vec<u16>,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64]. 
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    /// Dieses Array wird später an den `Level`-Struct weitergegeben
    pub lx_top: Vec<u64>,

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
            hash_map: (HashMapEnum::None),
            keys: vec![],
            lx_top: vec![0;lx_top_size],
            maximum: 0,
            minimum: 1
        }
    }
}


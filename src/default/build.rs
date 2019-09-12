use std::collections::HashMap;

use boomphf::Mphf;

use crate::internal::{Splittable};
use crate::default::immutable::{L2Ebene, L3Ebene, Int};


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
    root_top: Box<[u64]>,

    /// Root-Sub-Top-Array zur Leistungsoptimierung (siehe Paper)
    root_top_sub: Box<[u64]>, 

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
                
        let mut root_top: Vec<u64> = vec![0; T::root_array_size()/64];
        let mut root_top_sub: Vec<u64> = vec![0; T::root_array_size()/64/64];

        // Hier wird ein root_array der Länge T::root_array_size() angelegt, was 2^i entspricht. Dabei entspricht bei einem u40 Integer i=40 .
        let mut root_table: Box<[L2EbeneBuilder]> = vec![L2EbeneBuilder::new(LX_ARRAY_SIZE/64);T::root_array_size()].into_boxed_slice();
    
        for element in elements {
            let (i,j,k) = Splittable::split_integer_down(&element);

            if !root_indexs.contains(&i) {
                Self::build_root_top(&mut root_top, &mut root_top_sub, &i);
                root_indexs.push(i);
            }
            
            if !root_table[i].hash_map.contains_key(&j) {
                root_table[i].keys.push(j);
                root_table[i].hash_map.insert(j,L3EbeneBuilder::new(LX_ARRAY_SIZE/64));
            }
            
            // Hier ist keine Prüfung notwendig, da die Elemente einmalig sind.
            root_table[i].hash_map.get_mut(&j).unwrap().keys.push(k);
        }
        Self {root_table: root_table, root_top: root_top.into_boxed_slice(), root_top_sub: root_top_sub.into_boxed_slice(), root_indexs: root_indexs}
    }

    /// Baut ein Array `root_table` für den STree-Struct. Dabei werden zuerst die `Level`-Structs korrekt mittels neuer perfekter Hashfunktionen
    /// angelegt und miteinander verbunden. Nachdem die Struktur mit normalen Hashfunktionen gebaut wurde können nun perfekte Hashfunktionen berechnet 
    /// werden!
    pub fn build<T: Int>(&self) -> Box<[L2Ebene]> {
        let mut tmp: Vec<L2Ebene> = Vec::with_capacity(T::root_array_size());
        for i in 0..tmp.capacity() {
            tmp.push(L2Ebene::new(LX_ARRAY_SIZE/64, Some(self.root_table[i].keys.clone())));
        }
        let mut result: Box<[L2Ebene]> = tmp.into_boxed_slice();

        for &i in &self.root_indexs {
            for _ in &self.root_table[i].keys {
                result[i].objects.push(L3Ebene::new(LX_ARRAY_SIZE, None));
            }

            for &key in &self.root_table[i].keys {
                let len = self.root_table[i].hash_map.get(&key).unwrap().keys.len();
                Self::build_lx_top(&mut result[i].lx_top, key);
                let keys = self.root_table[i].hash_map.get(&key).unwrap().keys.as_ref();

                result[i].get(key).hash_function = Some(Mphf::new_parallel(GAMMA,&keys, None));
                    
                for _ in 0..len {
                    for &sub_key in &self.root_table[i].hash_map.get(&key).unwrap().keys {
                        Self::build_lx_top(&mut result[i].get(key).lx_top,sub_key);
                    }
                    result[i].get(key).objects.push(None);
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
    fn build_root_top(root_top: &mut Vec<u64>, root_top_sub: &mut Vec<u64>, bit: &usize) {
        // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
        let index = bit/64;
        let bit_mask: u64  = 1<<(63-(bit%64));
        root_top[index] = root_top[index] | bit_mask;

        // Berechnung des Indexs (sub_bit) im root_top_sub array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
        let index_sub = index/64;
        let bit_mask_sub: u64 = 1<<(63-(index%64));
        root_top_sub[index_sub] = root_top_sub[index_sub] | bit_mask_sub;
    }

    pub fn get_root_tops(&mut self) -> (Box<[u64]>,Box<[u64]>) {
        (std::mem::replace(&mut self.root_top,Box::new([])), std::mem::replace(&mut self.root_top_sub,Box::new([])))
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays. 
#[derive(Clone)]
pub struct BuilderLevel<T> {
    /// Klassische HashMap zum aufbauen der perfekten Hashmap
    pub hash_map: std::collections::HashMap<u16,T>,

    /// Eine Liste aller bisher gesammelter Schlüssel, die später auf die nächste Ebene zeigen.
    /// Diese werden zur Erzeugung der perfekten Hashfunktion benötigt.
    pub keys: Vec<u16>,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64]. 
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    /// Dieses Array wird später an den `Level`-Struct weitergegeben
    pub lx_top: Vec<u64>,
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
            hash_map: (HashMap::<u16,T>::default()),
            keys: vec![],
            lx_top: vec![0;lx_top_size],
        }
    }
}

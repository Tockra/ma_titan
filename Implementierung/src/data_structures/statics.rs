#![allow(dead_code)]  
use ux::{u40,u10};
use boomphf::Mphf;
use crate::help::internal::{Splittable};
use crate::help::builder::STreeBuilder;

/// In dieser Implementierung werden u40 Integer gespeichert.
pub type Int = u40;

/// Die L2-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf eine
/// L3-Ebene zeigt.
pub type L2Ebene = Level<L3Ebene>;

/// Die L3-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf 
/// ein Indize der STree.element_list zeigt.
pub type L3Ebene = Level<Option<usize>>;

/// Statische Predecessor-Datenstruktur. Sie verwendet perfektes Hashing und ein Array auf der Element-Listen-Ebene.
/// Sie kann nur sortierte und einmalige Elemente entgegennehmen.
pub struct STree {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2-Ebene je Eintrag abgelegt.AsMut
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    root_table: Box<[L2Ebene]>,
    
    /// Das Root-Top-Array speichert für jeden Eintrag `root_table[x]`, der belegt ist, ein 1-Bit, sonst einen 0-Bit.AsMut
    /// Auch hier werden nicht 2^20 Einträge, sondern lediglich [u64;2^20/64] gespeichert. 
    root_top: Box<[u64; (1<<20)/64]>,

    /// Das Root-Top-Sub-Array ist ein Hilfsarray. Dabei werden immer 64-Bit des Root-Top-Arrays (also ein Index) verodert und zu einem 
    /// Bit in `root_top_sub`. Somit können die nächsten gesetzten Bits in `root_top` gefunden werden, ohne alle Einträge zu überprüfen.
    /// Die Länge des Arrays beträgt somit [u64;2^20 / 64 / 64].
    root_top_sub: Box<[u64; (1<<20)/64/64]>, 

    /// Die Elementliste beinhaltet einen Vektor konstanter Länge mit jeweils allen gespeicherten Elementen in sortierter Reihenfolge.
    element_list: Box<[Int]>,
}

impl STree {
    /// Gibt einen STree mit den in items enthaltenen Werten zurück.
    ///
    /// # Arguments
    ///
    /// * `items` - Eine Liste mit sortierten u40-Werten, die in die statische Datenstruktur eingefügt werden sollten. Kein Wert darf doppelt vorkommen! 
    pub fn new(items: Vec<Int>) -> STree {
        let builder = STreeBuilder::new(items.clone());

        let (root_top,root_top_sub) = builder.build_root_top();
        let mut result = STree{
            root_table: builder.build(),
            root_top: root_top,
            root_top_sub: root_top_sub, 
            element_list: items.into_boxed_slice(),
        };
        for (index,element) in result.element_list.iter().enumerate() {
            // Dadurch das die Reihenfolge sortiert ist, wird das letzte hinzugefügte Element das größte und das erste das kleinste sein.

            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(element);

            let root = &mut result.root_table[i];
            root.minimum.get_or_insert(index);
            root.maximum = Some(index);

            let first_key = root.hash_function.as_ref().unwrap().hash(&j) as usize;
            let first = &mut root.objects[first_key];

            // Minima- und Maximasetzung auf der ersten Ebene
            first.minimum.get_or_insert(index);
            first.maximum = Some(index);

            let second_key = first.hash_function.as_ref().unwrap().hash(&k) as usize;
            // Werte korrekt auf die Array-Indizes zeigen lassen:Level
            first.objects[second_key] = Some(index);
        }
        result
    }


    /// Gibt die Anzahl der in self enthaltenen Elemente zurück.
    #[inline]
    pub fn len(&self) -> usize {
        self.element_list.len()
    }

    /// Gibt das in der Datenstruktur gespeicherte Minimum zurück. Falls die Datenstruktur leer ist, wird None zurückgegeben.
    #[inline]
    pub fn minimum(&self) -> Option<Int> {
        if self.len() == 0 {
            return None;
        }
        Some(self.element_list[0])
    }

    /// Gibt das in der Datenstruktur gespeicherte Minimum zurück. Falls die Datenstruktur leer ist, wird None zurückgegeben.
    #[inline]
    pub fn maximum(&self) -> Option<Int> {
        if self.len() == 0 {
            return None;
        }
        Some(self.element_list[self.len() - 1])
    }


    /// Diese Methode gibt den Index INDEX des kleinsten Elements zurück für das gilt element<=element_list[INDEX].
    /// Somit kann mit dieser Methode geprüft werden, ob ein Element in der Datenstruktur enthalten ist. Dann wird der Index dieses Elements zurückgegeben.
    /// Ist das Element nicht enthalten, wird der "Nachfolger" dieses Elements zurückgegeben.
    /// 
    /// # Arguments
    ///
    /// * `element` - Evtl. in der Datenstruktur enthaltener Wert, dessen Index zurückgegeben wird. Anderenfalls wird der Index des Nachfolgers von element zurückgegeben.
    #[inline]
    pub fn locate_or_succ(&self, element: Int) -> Option<usize> {
        let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&element);

        // Paper z.1 
        if self.len() < 1 || element > self.maximum().unwrap() {
            return None;
        } 

        // Paper z. 3 
        if self.root_table[i].maximum.is_none() || self.element_list[self.root_table[i].maximum.unwrap()] < element {
            return self.compute_next_set_bit(u40::new(i as u64))
                .map(|x| self.root_table[u64::from(x) as usize].minimum.unwrap());
        }
       
        // Paper z. 4
        if self.root_table[i].maximum == self.root_table[i].minimum {
            return Some(self.root_table[i].minimum.unwrap());
        }

        // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
        if self.root_table[i].get(&j).is_none() || self.root_table[i].get(&j).and_then(|x| if x.origin_key == Some(j) {Some(x)} else {None}).is_none() || self.element_list[self.root_table[i].get(&j).unwrap().maximum.unwrap()] < element {
            let new_j = self.root_table[i].compute_next_set_bit(&(j+u10::new(1)));
            return new_j
                .and_then(|x| self.root_table[i].get(&(x)))
                .map(|x| x.minimum.unwrap());
        }
    

        // Paper z.7
        if self.root_table[i].get(&j).unwrap().maximum == self.root_table[i].get(&j).unwrap().minimum {
            return Some(self.root_table[i].get(&j).unwrap().minimum.unwrap());
        }

        // Paper z.8
        let new_k = self.root_table[i].get(&j).unwrap().compute_next_set_bit(&k);
        return new_k
            .map(|x| self.root_table[i].get(&j).unwrap().get(&x).unwrap().unwrap());

    }

    /// Hilfsfunktion, die in der Root-Top-Sub-Tabelle das nächste Bit, dass nach Index `bit` gesetzt ist, zurückgibt. 
    /// 
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top_sub, dessen "Nachfolger" gesucht werden soll.
    fn compute_next_set_bit_deep(&self, bit: Int, level:u8) -> Option<Int> {
        let bit = u64::from(bit) + 1;
        let index = bit as usize/64;
        let in_index = bit%64;
        let bit_mask: u64 = u64::max_value() >> in_index;

        if level != 0 {
            let nulls = (self.root_top_sub[index] & bit_mask).leading_zeros();
            if nulls != 64 {
                return Some(u40::new(index as u64 *64 + nulls as u64));
            } else {
                for i in index+1..self.root_top_sub.len() {
                    if self.root_top_sub[i] != 0 {
                        let nulls = self.root_top_sub[i].leading_zeros();
                        return Some(u40::new(i as u64 * 64 + nulls as u64));
                    }
                } 
            }

            None
        }
        else {
            self.compute_next_set_bit(u40::new(bit))
        }
    }

    /// Hilfsfunktion, die in der Root-Top-Tabelle das nächste Bit, dass nach Index `bit` gesetzt ist, zurückgibt. 
    /// 
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top, dessen "Nachfolger" gesucht werden soll.
    fn compute_next_set_bit(&self, bit: Int) -> Option<Int> {
        let bit = u64::from(bit) + 1;
        let index = bit as usize/64;
        let in_index = bit%64;
        // Da der Index von links nach rechts gezählt wird, aber 2^i mit i=index von rechts nach Links gilt, muss 64-in_index gerechnet werden.
        // Diese Bit_Maske dient dem Nullen der Zahlen hinter in_index
        let bit_mask: u64 = u64::max_value() >> in_index; // genau falschherum
        
        // Leading Zeros von root_top[index] bestimmen und mit in_index vergleichen. Die erste führende 1 muss rechts von in_index liegen oder an Position in_index.
        let nulls = (self.root_top[index] & bit_mask).leading_zeros();
        if nulls != 64 {
            return Some(u40::new(index as u64 *64+nulls as u64));
        }
        
        // Wenn Leading Zeros=64, dann locate_top_level(element,level+1)
        let new_index = self.compute_next_set_bit_deep(u40::new(bit as u64/64) ,1);
        new_index.and_then(|x|
            match self.root_top[u64::from(x) as usize].leading_zeros() {
                64 => None,
                val => Some(u40::new(u64::from(x)*64 + val as u64))
            }
        )
        
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays. 
pub struct Level<T> {
    /// Perfekte Hashfunktion, die immer (außer zur Inialisierung) gesetzt ist. 
    pub hash_function: Option<Mphf<u10>>,

    /// Array, das mit Hilfe der perfekten Hashfunktion `hash_function` auf Objekte zeigt. 
    /// In objects sind alle Objekte gespeichert, auf die die Hashfunktion zeigen kann. Diese Objekte sind vom Typ T.
    pub objects: Vec<T>,

    /// Falls mittels Hashfunktion auf ein Level gezeigt wird, muss geprüft werden, ob der verwendete Key überhaupt "Hashbar" sein sollte
    pub origin_key: Option<u10>,

    /// Speichert einen Zeiger auf den Index des Maximum dieses Levels
    pub maximum: Option<usize>,

    /// Speichert einen Zeiger auf den Index des Minimums dieses Levels
    pub minimum: Option<usize>,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64]. 
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    pub lx_top: Vec<u64>,
}

impl<T> Level<T> {
    /// Gibt ein Level<T> mit Origin-Key j zurück. Optional kann eine Schlüsselliste übergeben werden, für welche dann
    /// eine perfekte Hashfunktion generiert wird.
    ///
    /// # Arguments
    ///
    /// * `j` - Falls eine andere Ebene auf diese mittels Hashfunktion zeigt, muss der verwendete key gespeichert werden. 
    /// * `keys` - Eine Liste mit allen Schlüsseln, die mittels perfekter Hashfunktion auf die nächste Ebene zeigen.
    #[inline]
    pub fn new(level: usize, origin_key: Option<u10>, keys: Option<Vec<u10>>) -> Level<T> {
        /*
            Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen, 
            Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
        */
        match keys {
            Some(x) => Level {
                hash_function: Some(Mphf::new_parallel(2.0,&x,None)),
                objects: vec![],
                origin_key: origin_key,
                maximum: None,
                minimum: None,
                lx_top: vec![0;level],
            },
            None => Level {
                hash_function: None,
                objects: vec![],
                origin_key: origin_key,
                maximum: None,
                minimum: None,
                lx_top: vec![0;level],
            }
        }
    }

    /// Mit Hilfe dieser Funktion kann die perfekte Hashfunktion verwendet werden. 
    /// Es muss beachtet werden, dass sichergestellt werden muss, dass der verwendete Key auch existiert!
    /// 
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get(&self, key: &u10) -> Option<&T> {
        let hash = self.hash_function.as_ref().unwrap().try_hash(&key)? as usize;
        self.objects.get(hash)
    }

    /// Hilfsfunktion, die in der Lx-Top-Tabelle das nächste Bit, dass nach dem `bit` gesetzt ist, zurückgibt. 
    /// 
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top, dessen "Nachfolger" gesucht werden soll.
    #[inline]
    pub fn compute_next_set_bit(&self, bit: &u10) -> Option<u10> {
        let bit = u16::from(*bit);
        let index = bit as usize/64;

        if self.lx_top[index] != 0 {
            let in_index = bit%64;
            let bit_mask: u64 = u64::max_value() >> in_index;
            let num_zeroes = (self.lx_top[index] & bit_mask).leading_zeros();

            if num_zeroes != 64 {
                return Some(u10::new(index as u16 *64 + num_zeroes as u16));
            }
        }
        for i in index+1..self.lx_top.len() {
            let val = self.lx_top[i];
            if val != 0 {
                let num_zeroes = val.leading_zeros();
                return Some(u10::new(i as u16 *64 + num_zeroes as u16));
            }
        }
        None
    }

}








#[cfg(test)]
mod tests {
    use ux::{u40,u10};
    use super::STree;
    use crate::help::internal::{Splittable};

    /// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
    #[test]
    fn test_new_hashfunctions() {

        // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
        // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
        let mut data: Vec<u40> = vec![u40::new(0);1<<10];
        
        for i in 0..data.len() {
            data[i] = u40::new(i as u64);
        }
 
        let check = data.clone();
        let data_structure: STree = STree::new(data);

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u40::new(0));
        assert_eq!(data_structure.maximum().unwrap(),u40::new(check.len() as u64 - 1));
        for val in check {
            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&val);
            let second_level = &data_structure.root_table[i].objects[data_structure.root_table[i].hash_function.as_ref().unwrap().hash(&j) as usize];
            let saved_val = second_level.objects[second_level.hash_function.as_ref().unwrap().hash(&k) as usize].unwrap();
            assert_eq!(data_structure.element_list[saved_val],val);
        }
    }
    
    /// Die Top-Arrays werden geprüft. Dabei wird nur grob überprüft, ob sinnvolle Werte gesetzt wurden.
    /// Dieser Test ist ein Kandidat zum Entfernen oder Erweitern.
    #[test]
    fn test_top_arrays() {
        let data: Vec<u40> = vec![u40::new(0b00000000000000000000_1010010010_0101010101),u40::new(0b00000000000000000000_1010010010_0101010111),u40::new(0b11111111111111111111_1010010010_0101010101)];
        let check = data.clone();
        let data_structure: STree = STree::new(data);

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u40::new(0b00000000000000000000_1010010010_0101010101));
        assert_eq!(data_structure.maximum().unwrap(),u40::new(0b11111111111111111111_1010010010_0101010101));

        for val in check {
            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&val);
            let second_level = &data_structure.root_table[i].objects[data_structure.root_table[i].hash_function.as_ref().unwrap().hash(&j) as usize];
            let saved_val = second_level.objects[second_level.hash_function.as_ref().unwrap().hash(&k) as usize].unwrap();
            assert_eq!(data_structure.element_list[saved_val],val);
        }
        // Root_TOP
        // 61 Nullen
        assert_eq!(data_structure.root_top[0],0b1000000000000000000000000000000000000000000000000000000000000000);
        for i in 1..16383 {
            assert_eq!(data_structure.root_top[i],0);
        }
        assert_eq!(data_structure.root_top[16383],1);

        // ROOT_TOP_SUB
        assert_eq!(data_structure.root_top_sub[0], 0b1000000000000000000000000000000000000000000000000000000000000000);
        for i in 1..255 {
            assert_eq!(data_structure.root_top_sub[i],0);
        }
        assert_eq!(data_structure.root_top_sub[255], 1);
        
    }

    /// Die locate_or_succ-Funktion wird getestet. Dabei werden beliebige Werte in ein STree gegeben und anschließend wird
    /// `locate_or_succ(x) mit allen x zwischen STree.min() und STree.max() getestet.
    #[test]
    fn test_locate_or_succ_bruteforce() {
        let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
        let mut data: Vec<u40> = vec![];
        for val in data_v1.iter() {
            data.push(u40::new(*val));
        }
        
        let data_structure: STree = STree::new(data);
        for (index,_) in data_v1.iter().enumerate() {
            if index < data_v1.len()-1 {
                for i in data_v1[index]+1..data_v1[index+1]+1 {
                    let locate = data_structure.locate_or_succ(u40::new(i)).unwrap();
                    assert_eq!(data_structure.element_list[locate], u40::new(data_v1[index+1]));
                }
            }
        }
    }

    /// # Äquivalenzklassentest mit Bruteforce
    /// `locate_or_succ` wird getestet. Dabei werden in jeder Ebene die gesuchten Elemente einmal im Minimum, im Maximum und irgendwo dazwischen liegen.
    #[test]
    fn test_locate_or_succ_eqc_bruteforce_test() {
        let data_raw: Vec<u64> = vec![
            0b00000000000000000000_0000000000_0000000001,
            0b00000000000000000000_0000000000_0000111000,
            0b00000000000000000000_0000000000_1111111111,

            0b00000000000000000000_0001110000_0000000000,
            0b00000000000000000000_0001110000_0000111000,
            0b00000000000000000000_0001110000_1111111111,

            0b00000000000000000000_1111111111_0000000000,
            0b00000000000000000000_1111111111_0000111000,
            0b00000000000000000000_1111111111_1111111111,

            0b00000000001111000000_0000000000_0000000000,
            0b00000000001111000000_0000000000_0000111000,
            0b00000000001111000000_0000000000_1111111111,

            0b00000000001111000000_0001110000_0000000000,
            0b00000000001111000000_0001110000_0000111000,
            0b00000000001111000000_0001110000_1111111111,

            0b00000000001111000000_1111111111_0000000000,
            0b00000000001111000000_1111111111_0000111000,
            0b00000000001111000000_1111111111_1111111111,

            0b11111111111111111111_0000000000_0000000000,
            0b11111111111111111111_0000000000_0000111000,
            0b11111111111111111111_0000000000_1111111111,

            0b11111111111111111111_0001110000_0000000000,
            0b11111111111111111111_0001110000_0000111000,
            0b11111111111111111111_0001110000_1111111111,

            0b11111111111111111111_1111111111_0000000000,
            0b11111111111111111111_1111111111_0000111000,
            0b11111111111111111111_1111111111_1111111110,
            
        ];

        let mut data: Vec<u40> = vec![];
        for val in data_raw.iter() {
            data.push(u40::new(*val));
        }
        let data_structure: STree = STree::new(data.clone());
        assert_eq!(data_structure.locate_or_succ(u40::new(0b11111111111111111111_1111111111_1111111111)), None);
        assert_eq!(data_structure.locate_or_succ(u40::new(0)), Some(0));

        for (i,&elem) in data.iter().enumerate() {
            if i > 0 {
                for j in 0..16877216 {
                    if u64::from(elem)>=j as u64 {
                        let index = elem - u40::new(j);
                        if index > data_structure.element_list[i-1] {
                            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(index).unwrap() as usize], elem);
                        }
                    }
                }
            } else {
                assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem).unwrap() as usize], elem);
                assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem-u40::new(1)).unwrap() as usize], elem);
            }
        }
    }
}
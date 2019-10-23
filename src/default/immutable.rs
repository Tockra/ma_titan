use uint::{u40, u48};

use crate::default::build::{insert_l3_level, build_lx_top,build_root_top, create_root_top};
use crate::internal::{Splittable, MphfHashMapThres};

/// Die L2-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf eine
/// L3-Ebene zeigt.
pub type L2Ebene = LevelPointer<L3Ebene>;

/// Die L3-Ebene ist eine Zwischenebene, die mittels eines u10-Integers und einer perfekten Hashfunktion auf 
/// ein Indize der STree.element_list zeigt.
pub type L3Ebene = LevelPointer<usize>;

pub enum Pointer<T: 'static> {
    Level(&'static mut Level<T>),
    Element(&'static mut usize)
}

use crate::internal::{self, PointerEnum};

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein usize-Objekt zeigt (Index aus Elementliste),
/// oder auf ein Levelobjekt
#[derive(Clone)]
pub struct LevelPointer<T: 'static> {
    pointer: internal::Pointer<Level<T>,usize>
}

impl<T: 'static> LevelPointer<T> {

    fn minimum(&self) -> usize {
        if self.pointer.is_null() {
            panic!("Minimum-Aufruf auf Null!");
        }

        match self.pointer.get() {
            PointerEnum::First(l) => {
                (*l).minimum
            },

            PointerEnum::Second(e) => {
                *e
            }
        }
    }

    fn maximum(&self) -> usize {
        if self.pointer.is_null() {
            panic!("Maximum-Aufruf auf Null!");
        }
        
        match self.pointer.get() {
            PointerEnum::First(l) => {
                (*l).maximum
            },

            PointerEnum::Second(e) => {
                *e
            }
        }
    }

    pub fn from_level(level_box: Box<Level<T>>) -> Self {
        Self {
            pointer: internal::Pointer::from_first(level_box)
        }
    }

    pub fn get(&self) -> PointerEnum<Level<T>, usize> {
        self.pointer.get()
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }

    pub fn from_null() -> Self {
        Self {
            pointer: internal::Pointer::null()
        }
    }

    pub fn from_usize(usize_box: Box<usize>) -> Self {
        Self {
            pointer: internal::Pointer::from_second(usize_box)
        }
    }
}

/// Statische Predecessor-Datenstruktur. Sie verwendet perfektes Hashing und ein Array auf der Element-Listen-Ebene.
/// Sie kann nur sortierte und einmalige Elemente entgegennehmen.
#[derive(Clone)]
pub struct STree<T> {
    /// Mit Hilfe der ersten 20-Bits des zu speichernden Wortes wird in `root_table` eine L2-Ebene je Eintrag abgelegt.
    /// Dabei gilt `root_table: [L2Ebene;2^20]`
    pub root_table: Box<[L2Ebene]>,
    
    /// Das Root-Top-Array speichert für jeden Eintrag `root_table[i][x]`, der belegt ist, ein 1-Bit, sonst einen 0-Bit.
    /// Auch hier werden nicht 2^20 Einträge, sondern lediglich [u64;2^20/64] gespeichert.
    /// i steht dabei für die Ebene der root_tabelle. Ebene i+1 beinhaltet an Index [x] immer 64 Veroderungen aus Ebene i. 
    /// Somit gilt |root_table[i+1]| = |root_table[i]|/64  
    pub root_top: Box<[Box<[u64]>]>,

    /// Die Elementliste beinhaltet einen Vektor konstanter Länge mit jeweils allen gespeicherten Elementen in sortierter Reihenfolge.
    pub element_list: Box<[T]>,
}

/// Dieser Trait dient als Platzhalter für u40, u48 und u64. 
/// Er stellt sicher das der generische Parameter gewisse Traits implementiert und die New-Methode besitzt.
/// Zusätzlich wird die Größe des Root-Arrays in Form einer Funktion rückgebar gemacht.
pub trait Int: Ord + PartialOrd + From<u64> + Into<u64> + Copy + Splittable { 
    fn new(k: u64) -> Self {
        Self::from(k)
    }
    fn root_array_size() -> usize {
        1 << (std::mem::size_of::<Self>()*8/2)
    }
}

impl Int for u40 {

}

impl Int for u48 {

}

impl Int for u64 {

}

impl<T: Int> STree<T> {
    /// Gibt einen STree mit den in `elements` enthaltenen Werten zurück.
    ///
    /// # Arguments
    ///
    /// * `elements` - Eine Liste mit sortierten u40-Werten, die in die statische Datenstruktur eingefügt werden sollten. Kein Wert darf doppelt vorkommen! 
    pub fn new(elements: Box<[T]>) -> Self {
        if elements.len() == 0 {
            panic!("Ein statischer STree muss mindestens 1 Element beinhalten.");
        }
        let mut root_table: Box<[L2Ebene]> = vec![LevelPointer::from_null(); T::root_array_size()].into_boxed_slice();
        let mut root_top = create_root_top::<T>();

        for (index,elem) in elements.iter().enumerate() {
            let (i,j,k) = Splittable::split_integer_down(elem);
            build_root_top(&mut root_top, &i);
 
            if root_table[i].is_null() {
                root_table[i] = LevelPointer::from_usize(Box::new(index));
            } else {
                match root_table[i].get() {
                    PointerEnum::First(l2_object) => {
                        l2_object.maximum = index;

                        if !l2_object.contains(j) {
                            let mut l3_level = L3Ebene::from_null();
                            insert_l3_level(&mut l3_level,index,k,&elements);

                            l2_object.hash_map.insert(j,l3_level);
                            build_lx_top(&mut l2_object.lx_top, j);
                        } else {
                            // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                            insert_l3_level(l2_object.hash_map.get(&j),index,k,&elements);
                        }
      
                    },
                    PointerEnum::Second(elem_index) => {
                        let lx_array_size  = 1_usize<<(((std::mem::size_of::<T>()*8)/2)/2); 
                        let mut l2_object = Level::new(lx_array_size);
                        let elem2 = elements[*elem_index];
                        let (_,j2,k2) = Splittable::split_integer_down(&elem2);
                        
                        // Da die Elemente sortiert sind
                        l2_object.minimum = *elem_index;
                        l2_object.maximum = index;

                        build_lx_top(&mut l2_object.lx_top, j);

                        let mut l3_level = L3Ebene::from_null();

                        if j2 != j {
                            let mut l3_level = L3Ebene::from_null();
                            insert_l3_level(&mut l3_level,*elem_index,k2,&elements);

                            l2_object.hash_map.insert(j2,l3_level);
                            build_lx_top(&mut l2_object.lx_top, j2);
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
    pub fn maximum_level<E>(&self, lx: &Level<E>) -> T {
        self.element_list[lx.maximum]
    }

    /// Gibt das Minimum der übergebenen Ebene zurück.
    /// 
    /// # Arguments
    ///
    /// * `lx` - Referenz auf die Ebene, dessen Minimum zurückgegeben werden soll.
    #[inline]
    pub fn minimum_level<E>(&self, lx: &Level<E>) -> T {
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

        let (i,j,k) = Splittable::split_integer_down(&element);

        // Paper z.3
        if self.root_table[i].is_null() || element < self.element_list[self.root_table[i].minimum()] {
            return self.compute_last_set_bit_deep(T::new(i as u64),0)
                .map(|x| self.root_table[x].maximum());
        }

        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none() || element < self.element_list[third_level.unwrap().minimum()] {
                    let new_j = second_level.compute_last_set_bit(&(j-1u16));
                    return new_j
                        .and_then(|x| second_level.try_get(x))
                        .map(|x| x.maximum());
                }

                // Paper z.7
                match third_level.unwrap().get() {
                    PointerEnum::First(l) => {
                        // Paper z.8
                        let new_k = (*l).compute_last_set_bit(&k);
                        return new_k
                            .map(|x| *(*l).try_get(x).unwrap());
                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(*e);
                    }
                }
        
                
                
            },

            PointerEnum::Second(e) => {
                return Some(*e);
            }
        }


}

    /// Hilfsfunktion, die in der Root-Top-Tabelle das letzte Bit, dass vor Index `bit` gesetzt ist, zurückgibt. 
    /// Achtung diese Funktion funktioniert etwas anders als Level::compute_last_set_bit_deep !
    /// 
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top_sub, dessen "Vorgänger" gesucht werden soll.
    fn compute_last_set_bit_deep(&self, bit: T, level:usize) -> Option<usize> {
        let bit: u64 = bit.into() - 1u64;
        let index = bit as usize/64;
        let in_index = bit%64;
        // Da der Index von links nach rechts gezählt wird, aber 2^i mit i=index von rechts nach Links gilt, muss 64-in_index gerechnet werden.
        // Diese Bit_Maske dient dem Nullen der Zahlen hinter in_index
        let bit_mask: u64 = u64::max_value() << (63-in_index); // genau andersrum (in 111..11 werden 0en reingeschoben)

        if level != self.root_top.len()-1 {
            // Leading Zeros von root_top[index] bestimmen und mit in_index vergleichen. Die erste führende 1 muss rechts von in_index liegen oder an Position in_index.
            let nulls = (self.root_top[0][index] & bit_mask).trailing_zeros();
            if nulls != 64 {
                return Some(((index + 1) as u64 *64-(nulls+1) as u64) as usize);
            }
            
            // Wenn Leading Zeros=64, dann locate_top_level(element,level+1)
            let new_index = self.compute_last_set_bit_deep(T::new(bit as u64/64) ,1);
            new_index.and_then(|x|
                match self.root_top[0][x].trailing_zeros() {
                    64 => None,
                    val => Some(((x+1) as u64 *64 - (val+1) as u64) as usize)
                }
            )
        }
        else {
            let nulls = (self.root_top[1][index] & bit_mask).trailing_zeros();
            if nulls != 64 {
                return Some(((index+1) as u64 *64 - (nulls+1) as u64) as usize);
            } else {
                for i in (0..index).rev() {
                    if self.root_top[1][i] != 0 {
                        let nulls = self.root_top[1][i].trailing_zeros();
                        return Some(((i+1) as u64 * 64 - (nulls+1) as u64) as usize);
                    }
                } 
            }
            
            None
        }
        /*else {
            //self.compute_last_set_bit_deep(T::new(bit+1))
        }*/
       
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
    

        let (i,j,k) = Splittable::split_integer_down(&element);
        
        // Paper z.3
        
        if self.root_table[i].is_null() || self.element_list[self.root_table[i].maximum()] < element {
            return self.compute_next_set_bit_deep(T::new(i as u64),0)
                .map(|x| self.root_table[x].minimum());
        }

       
        // Paper z. 4 (durch die Match-Arme)
        match self.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l;
                let third_level = second_level.try_get(j);
                
                // Paper z. 6 mit kleiner Anpassung wegen "Perfekten-Hashings"
                if third_level.is_none() || self.element_list[third_level.unwrap().maximum()] < element {
                    let new_j = second_level.compute_next_set_bit(&(j+1u16));
                    return new_j
                        .and_then(|x| second_level.try_get(x))
                        .map(|x| x.minimum());
                }
               
                // Paper z.7
                match third_level.unwrap().get() {
                    PointerEnum::First(l) => {
                        // Paper z.8
                        let new_k = (*l).compute_next_set_bit(&k);
                        return new_k
                            .map(|x| *(*l).try_get(x).unwrap());
                    }
                    // Paper z.7
                    PointerEnum::Second(e) => {
                        return Some(*e);
                    }
                }
        
                
                
            },

            PointerEnum::Second(e) => {
                return Some(*e);
            }
        }
        
    }

    /// Hilfsfunktion, die in der Root-Top-Tabelle das nächste Bit, dass nach Index `bit` gesetzt ist, zurückgibt. 
    /// Achtung diese Funktion funktioniert etwas anders als Level::compute_next_set_bit_deep !
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top_sub, dessen "Nachfolger" gesucht werden soll.
    fn compute_next_set_bit_deep(&self, bit: T, level:usize) -> Option<usize> {
        let bit: u64 = bit.into() + 1u64;
        let index = bit as usize/64;
        let in_index = bit%64;
        // Da der Index von links nach rechts gezählt wird, aber 2^i mit i=index von rechts nach Links gilt, muss 64-in_index gerechnet werden.
        // Diese Bit_Maske dient dem Nullen der Zahlen hinter in_index
        let bit_mask: u64 = u64::max_value() >> in_index; // genau andersrum (in 111..11 werden 0en reingeschoben)

        if level != self.root_top.len()-1 {
            // Leading Zeros von root_top[index] bestimmen und mit in_index vergleichen. Die erste führende 1 muss rechts von in_index liegen oder an Position in_index.
            let nulls = (self.root_top[0][index] & bit_mask).leading_zeros();
            if nulls != 64 {
                return Some((index as u64 *64+nulls as u64) as usize);
            }
            
            // Wenn Leading Zeros=64, dann locate_top_level(element,level+1)
            let new_index = self.compute_next_set_bit_deep(T::new(bit as u64/64) ,1);
            new_index.and_then(|x|
                match self.root_top[0][x].leading_zeros() {
                    64 => None,
                    val => Some(((x as u64)*64 + val as u64) as usize)
                }
            ) 
        } else {
            let nulls = (self.root_top[1][index] & bit_mask).leading_zeros();
            if nulls != 64 {
                return Some((index as u64 *64 + nulls as u64) as usize);
            } else {
                for i in index+1..self.root_top[1].len() {
                    if self.root_top[1][i] != 0 {
                        let nulls = self.root_top[1][i].leading_zeros();
                        return Some((i as u64 * 64 + nulls as u64) as usize);
                    }
                } 
            }
            
            None
        }
        /*else {
            self.compute_next_set_bit(T::new(bit-1))
        }*/
    }
}

/// Zwischenschicht zwischen dem Root-Array und des Element-Arrays. 
#[derive(Clone)]
#[repr(align(4))]
pub struct Level<T: 'static> {
    /// Perfekte Hashmap, die immer (außer zur Inialisierung) gesetzt ist. 
    pub hash_map: MphfHashMapThres<u16,T>,

    /// Speichert einen Zeiger auf den Index des Maximum dieses Levels
    pub maximum: usize,

    /// Speichert einen Zeiger auf den Index des Minimums dieses Levels
    pub minimum: usize,

    /// Speichert die L2-, bzw. L3-Top-Tabelle, welche 2^10 (Bits) besitzt. Also [u64;2^10/64]. 
    /// Dabei ist ein Bit lx_top[x]=1 gesetzt, wenn x ein Schlüssel für die perfekte Hashfunktion ist und in objects[hash_function.hash(x)] mindestens ein Wert gespeichert ist.
    pub lx_top: Box<[u64]>,
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
    pub fn new(lx_size: usize) -> Level<T> {
        Level {
            hash_map: MphfHashMapThres::new(),
            minimum: 0,
            maximum: 0,
            lx_top: vec![0;lx_size].into_boxed_slice(),
        }
    }

    /// Mit Hilfe dieser Funktion kann die perfekte Hashfunktion verwendet werden. 
    /// Es muss beachtet werden, dass sichergestellt werden muss, dass der verwendete Key auch existiert!
    /// 
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn try_get(&self, key: u16) -> Option<&T> {
        self.hash_map.try_get(key,&self.lx_top)
    }

    pub fn contains(&self, key: u16) -> bool {
        let index = (key/64) as usize;
        let in_index_mask = 1<<(63-(key % 64));

        (self.lx_top[index] & in_index_mask) != 0
    }

    /// Der zum `key` gehörende gehashte Wert wird aus der Datenstruktur ermittelt. Hierbei muss sichergestellt sein
    /// das zu `key` ein Schlüssel gehört. Anderenfalls sollte `try_hash` verwendet werden
    /// 
    /// # Arguments
    ///
    /// * `key` - u10-Wert mit dessen Hilfe das zu `key` gehörende Objekt aus dem Array `objects` bestimmt werden kann.
    #[inline]
    pub fn get(&mut self, key: u16) -> &mut T {
        self.hash_map.get(&key)
    }

    

    

    /// Hilfsfunktion, die in der Lx-Top-Tabelle das nächste Bit, dass nach `bit` gesetzt ist, zurückgibt. Ist `bit=1` dann wird
    /// `bit` selbst zurückgegeben.
    /// 
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top, dessen "Nachfolger" gesucht werden soll.
    #[inline]
    pub fn compute_next_set_bit(&self, bit: &u16) -> Option<u16> {
        let bit = u16::from(*bit);
        let index = bit as usize/64;

        if self.lx_top[index] != 0 {
            let in_index = bit%64;
            let bit_mask: u64 = u64::max_value() >> in_index;
            let num_zeroes = (self.lx_top[index] & bit_mask).leading_zeros();

            if num_zeroes != 64 {
                return Some(index as u16 *64 + num_zeroes as u16);
            }
        }
        for i in index+1..self.lx_top.len() {
            let val = self.lx_top[i];
            if val != 0 {
                let num_zeroes = val.leading_zeros();
                return Some(i as u16 *64 + num_zeroes as u16);
            }
        }

        None
    }

    /// Hilfsfunktion, die in der Lx-Top-Tabelle das letzte Bit, dass vor `bit` gesetzt ist, zurückgibt. Ist `bit=1` dann wird
    /// `bit` selbst zurückgegeben. 
    /// 
    /// # Arguments
    ///
    /// * `bit` - Bitgenauer Index in self.root_top, dessen "Vorgänger" gesucht werden soll.
    #[inline]
    pub fn compute_last_set_bit(&self, bit: &u16) -> Option<u16> {
        let bit = u16::from(*bit);
        let index = bit as usize/64;

        if self.lx_top[index] != 0 {
            let in_index = bit%64;
            let bit_mask: u64 = u64::max_value() << (63-in_index);
            let num_zeroes = (self.lx_top[index] & bit_mask).trailing_zeros();

            if num_zeroes != 64 {
                return Some((index + 1) as u16 *64 - (num_zeroes+1) as u16);
            }
        }
        for i in (0..index).rev() {
            let val = self.lx_top[i];
            if val != 0 {
                let num_zeroes = val.trailing_zeros();
                return Some((i + 1) as u16 *64 - (num_zeroes+1) as u16);
            }
        }
        None
    }

}

#[cfg(test)]
mod tests {
    use uint::u40;
    use super::STree;
    use crate::internal::Splittable;

    use crate::internal::{PointerEnum};
    
    /// Größe der LX-Top-Arrays
    const LX_ARRAY_SIZE: usize = 1 << 10;

    /// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
    #[test]
    fn test_new_hashfunctions() {

        // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
        // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
        let mut data: Vec<u40> = vec![u40::new(0);LX_ARRAY_SIZE];
        
        for i in 0..data.len() {
            data[i] = u40::new(i as u64);
        }
 
        let check = data.clone();
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u40::new(0));
        assert_eq!(data_structure.maximum().unwrap(),u40::new(check.len() as u64 - 1));
        for val in check {
            let (i,j,k) = Splittable::split_integer_down(&val);
            match data_structure.root_table[i].get() {
                PointerEnum::First(l) => {
                    let second_level = l.get(j);
                    let saved_val = match second_level.get() {
                        PointerEnum::First(l) => {
                            *(*l).get(k)
                        },
                        PointerEnum::Second(e) => {
                            *e
                        }
                    };
                    assert_eq!(data_structure.element_list[saved_val],val);
                },

                PointerEnum::Second(e) => {
                    assert_eq!(data_structure.element_list[*e],val);
                }
            };

        }
    }
    
    /// Die Top-Arrays werden geprüft. Dabei wird nur grob überprüft, ob sinnvolle Werte gesetzt wurden.
    /// Dieser Test ist ein Kandidat zum Entfernen oder Erweitern.
    #[test]
    fn test_top_arrays() {
        let data: Vec<u40> = vec![u40::new(0b00000000000000000000_1010010010_0101010101),u40::new(0b00000000000000000000_1010010010_0101010111),u40::new(0b11111111111111111111_1010010010_0101010101_u64)];
        let check = data.clone();
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),u40::new(0b00000000000000000000_1010010010_0101010101));
        assert_eq!(data_structure.maximum().unwrap(),u40::new(0b11111111111111111111_1010010010_0101010101_u64));

        for val in check {
            let (i,j,k) = Splittable::split_integer_down(&val);
            if data_structure.root_table[i].minimum() != data_structure.root_table[i].maximum() {
                let second_level = match data_structure.root_table[i].get() {
                        PointerEnum::First(l) => {
                            l.get(j)
                        },
                        _ => {
                            panic!("Das sollte nicht geschehen");
                        }
                };
                if second_level.minimum() != second_level.maximum() {
                    let saved_val = match second_level.get() {
                        PointerEnum::First(l) => {
                            l.get(k)
                        },
                        _ => {
                            panic!("Das sollte nicht geschehen");
                        }
                    };
                    assert_eq!(data_structure.element_list[*saved_val],val);
                } else {
                    assert_eq!(data_structure.element_list[second_level.minimum()],val);
                }

            } else {
                assert_eq!(data_structure.element_list[data_structure.root_table[i].minimum()],val);
            }

        }
        // Root_TOP
        // 61 Nullen
        assert_eq!(data_structure.root_top[0][0],0b1000000000000000000000000000000000000000000000000000000000000000);
        for i in 1..16383 {
            assert_eq!(data_structure.root_top[0][i],0);
        }
        assert_eq!(data_structure.root_top[0][16383],1);

        // ROOT_TOP_SUB
        assert_eq!(data_structure.root_top[1][0], 0b1000000000000000000000000000000000000000000000000000000000000000);
        for i in 1..255 {
            assert_eq!(data_structure.root_top[1][i],0);
        }
        assert_eq!(data_structure.root_top[1][255], 1);
        
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
        
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());
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
        let data_structure: STree<u40> = STree::new(data.clone().into_boxed_slice());
        assert_eq!(data_structure.locate_or_succ(u40::new(0b11111111111111111111_1111111111_1111111111_u64)), None);
        
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

    /// Die locate_or_pred-Funktion wird getestet. Dabei werden beliebige (fest gewählte) Werte in ein STree gegeben und anschließend wird
    /// `locate_or_pred(x) mit allen x zwischen STree.min() und STree.max() getestet.
    #[test]
    fn test_locate_or_pred_bruteforce() {
        let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
        let mut data: Vec<u40> = vec![];
        for val in data_v1.iter() {
            data.push(u40::new(*val));
        }
        
        let data_structure: STree<u40> = STree::new(data.into_boxed_slice());
        assert_eq!(u40::new(1065983), data_structure.element_list[data_structure.locate_or_pred(u40::new(1065983)).unwrap()]);
        for (index,_) in data_v1.iter().enumerate().rev() {
            if index > 0 {
                for i in (data_v1[index-1]..data_v1[index]).rev() {
                    let locate = data_structure.locate_or_pred(u40::new(i)).unwrap();
                    assert_eq!(u40::new(data_v1[index-1]), data_structure.element_list[locate]);
                }
            }
        }
    }

    use num::Bounded;
     /// # Äquivalenzklassentest mit Bruteforce
    /// `locate_or_pred` wird getestet. Dabei werden in jeder Ebene die gesuchten Elemente einmal im Minimum, im Maximum und irgendwo dazwischen liegen.
    #[test]
    fn test_locate_or_pred_eqc_bruteforce_test() {
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
        let data_structure: STree<u40> = STree::new(data.clone().into_boxed_slice());
        assert_eq!(data_structure.locate_or_pred(u40::new(0)), None);

        for (i,&elem) in data.iter().enumerate().rev() {
            if i < data.len()-1 {
                for j in 0..16877216 {
                    if u40::max_value() > elem && u40::new(j) < u40::max_value() - elem {
                        let index = elem + u40::new(j);
                        if index < data_structure.element_list[i+1] {
                            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(index).unwrap() as usize], elem);
                        }
                    }
                }
            } else {
                assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem).unwrap() as usize], elem);
                assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem+u40::new(1)).unwrap() as usize], elem);
            }
        }
    }
}
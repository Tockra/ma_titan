#![allow(dead_code)]

mod internal;
use internal::{List,Element};

use fnv::FnvHashMap;
use std::ptr;
use std::mem::{self, MaybeUninit};

#[cfg(test)]
mod tests;

/*
    Diese Datenstruktur agiert als Menge, die Zahlenwerte speichert. Explizit wird hier eine Implementierung für i32, i40, i48 und i64 geschaffen, 
    wobei die Datentypen i40 und i48 eigene Datentypen sind. 
    Diese Datenstruktur bietet predecessor(x) und sucessor(x) Methoden, welche den Vorgänger bzw. Nachfolger einer beliebigen Zahl x (egal ob enthalten oder nicht) ausgibt.

    Folgender Trait definiert die Methoden, die eine PredecessorSet beinhalten soll.
*/
pub trait PredecessorSet<T> {
    fn insert(&mut self,element: T);
    fn delete(&mut self,element: T);
    fn predecessor(&self,number: T) -> Option<T>;
    fn sucessor(&self,number: T) -> Option<T>; // Optional
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>; 
    fn contains(&self) -> bool;
}

const fn root_size<T>() -> usize {
    1 << 8*mem::size_of::<T>() / 2
}

pub type Int = i32;
type SecondLevel = Level<*mut Element<Int>,Int>;
type FirstLevel = Level<SecondLevel,Int>;

pub struct STree {
    root_table: [FirstLevel; root_size::<Int>()],
    // Da die Größe in in Bytes von size_of zurückgegeben wird, mal 8. Durch 64, da 64 Bits in einen u64 passen.
    root_top: [u64; root_size::<Int>()/64],
    root_top_sub: [u64; root_size::<Int>()/64/64], //Hier nur ein Element, da 2^16/64/64 nur noch 16 Bit sind, die alle in ein u64 passen!
    element_list: internal::List<Int>,
}

// Implementiert die zwei Level unter der Root-Tabelle. Diese besitzen ein Maximum- und ein Minimumpointer und ggf. eine hash_map, wenn *minimum!= *maximum
// Hier ext. 2 Ansätze die getestet werden müssen (abhängig vom Speicherverbrauch):
// 1. max/min sind RAW-Pointer auf Elemente
// 2. min/max sind u8 Werte und hashMap ist entweder ein RAW-Pointer auf eine FnvHashMap oder ein RAW Pointer auf ein Element (falls min==max) evtl. mit Union
struct Level<T,V> {
    pub hash_map: FnvHashMap<u8,T>,
    pub maximum: *mut Element<V>,
    pub minimum: *mut Element<V>,
    pub lx_top: Vec<u64>,
}

impl<T,V> Level<T,V> {
    #[inline]
    fn new(level: usize) -> Level<T,Int> {
        Level {
            hash_map: (FnvHashMap::<u8,T>::default()),
            maximum: ptr::null_mut(),
            minimum: ptr::null_mut(),
            lx_top: vec![0;level],
        }
    }

    // Die Hashtabelle beinhaltet viele Werte, die abhängig der nächsten 8 Bits der Binärdarstellung der zu lokalisierenden Zahl sind
    // Der lx_top-Vektor hält die Information, ob im Wert 0 bis 2^8 ein Wert steht. Da 64 Bit in einen u64 passen, hat der Vektor nur 4 Einträge mit jeweils 64 Bit (u64)
    #[inline]
    fn locate_top_level(&mut self, bit: u8) -> Option<u8> {
        let index = bit as usize/64;

        if self.lx_top[index] != 0 {
            let in_index = bit%64;
            let bit_mask: u64 = (1 << (64-in_index))-1;
            let num_zeroes = (self.lx_top[index] & bit_mask).leading_zeros();

            return Some(index as u8 *64 + num_zeroes as u8);
        }
        for i in index+1..self.lx_top.len() {
            let val = self.lx_top[i];
            if val != 0 {
                let num_zeroes = val.leading_zeros();
                return Some(i as u8 *64 + num_zeroes as u8);
            }
        }
        None
    }
}

impl STree {
    #[inline]
    pub fn new() -> STree {
        let data = {
            let mut data: [MaybeUninit<FirstLevel>; root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut data[..] {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), FirstLevel::new((1<<8)/64)); 
                }
            }

            unsafe { 
                mem::transmute::<_, [FirstLevel; root_size::<Int>()]>(data) 
            }
        };
        STree {
            element_list: List::new(),
            root_top: [0; root_size::<Int>()/64],
            root_top_sub: [0; root_size::<Int>()/64/64],
            root_table: data,
        }
    }

    /** 
     * Gibt den kleinstne Wert j mit element <= j zurück. 
     * Kann verwendet werden, um zu prüfen ob element in der Datenstruktur enthalten ist. 
     * Gibt anderenfalls den Nachfolger zurück, falls dieser existiert.
     */
    #[inline]
    pub fn locate(&mut self, element: Int) -> Option<*mut Element<Int>> {
        let i: usize = (element >> 16) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = element & 0xFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u8 = (low >> 8) as u8;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u8 = (element & 255) as u8;

        // Paper z.1 
        if self.len() < 1 || element > self.maximum().unwrap(){
            return None;
        } 

        // Paper z. 3 
        unsafe {
            if self.root_table[i].maximum.is_null() || (*self.root_table[i].maximum).elem < element {
                return self.locate_top_level(i as Int,0)
                    .map(|x| self.root_table[x as usize].minimum);
            }
        }

        // Paper z. 4
        if self.root_table[i].maximum == self.root_table[i].minimum {
            return Some(self.root_table[i].minimum);
        }

        // Paper z. 6
        unsafe {
            if self.root_table[i].hash_map.get_mut(&j).is_none() || (*self.root_table[i].hash_map.get_mut(&j).unwrap().maximum).elem < element {
                let new_j = self.root_table[i].locate_top_level(j);
                return new_j
                    .and_then(|x| self.root_table[i].hash_map.get_mut(&(x)))
                    .map(|x| x.minimum);
            }
        }

        // Paper z.7
        if self.root_table[i].hash_map.get_mut(&j).unwrap().maximum == self.root_table[i].hash_map.get_mut(&j).unwrap().minimum {
            return Some(self.root_table[i].hash_map.get_mut(&j).unwrap().minimum);
        }

        // Paper z.8
        let new_k = self.root_table[i].hash_map.get_mut(&j).unwrap().locate_top_level(k);
        return new_k
            .map(|x| *self.root_table[i].hash_map.get_mut(&j).unwrap().hash_map.get_mut(&x).unwrap());
       
    }

    /**
     * Gibt das kleinste j zurück, so dass element <= j und k_level[j]=1
     * Hierbei beachten, dass j zwar Bitweise adressiert wird, die Level-Arrays allerdings ganze 64-Bit-Blöcke besitzen. Somit ist z.B: root_top[5] nicht das 6. 
     * Bit sondern, der 6. 64-Bit-Block. Die Methode gibt aber die Bit-Position zurück!
     */ 
    fn locate_top_level(&mut self, bit: Int, level: u8) -> Option<Int> {
        let index = bit as usize/64;
        let in_index = bit%64;
        // Da der Index von links nach rechts gezählt wird, aber 2^i mit i=index von rechts nach Links gilt, muss 64-in_index gerechnet werden.
        // Diese Bit_Maske dient dem Nullen der Zahlen hinter in_index
        let bit_mask: u64 = (1 << (64-in_index))-1; // genau falschherum
        // Siehe Paper, irgendwo muss noch Fill Zeros implementiert werden
        
        if level != 0 {
            for i in index..self.root_top_sub.len() {
                if self.root_top_sub[i] != 0 {
                    let nulls = self.root_top_sub[i].leading_zeros();
                    return Some(i as i32*64 + nulls as i32);
                }
            }
            return None;
        }
        
        let nulls = (self.root_top[index] & bit_mask).leading_zeros();
        
        // Leading Zeros von root_top[index] bestimmen und mit in_index vergleichen. Die erste führende 1 muss rechts von in_index liegen oder an Position in_index.
        if nulls != 64 {
            return Some(index as i32 *64+nulls as i32);
        }
        
        // Wenn Leading Zeros=64, dann locate_top_level(element,level+1)
        let new_index = self.locate_top_level(index as i32 ,level+1);

        new_index.and_then(|x|
            match self.root_top[x as usize].leading_zeros() {
                64 => None,
                val => Some(x as i32*64  + val as i32)
            }
        )
        
    }
    

    // Diese Methode setzt die benötigten Bits in der Root-Top-Tabelle und in L1-Top und L2-Top
    #[inline]
    fn insert_into_top_table(&mut self, _element: Int) {
        unimplemented!();
    }

    #[inline]
    fn change_bounds(&mut self, _element: Int, _minimum: *mut Element<Int>, _maximum: *mut Element<Int>) {
        unimplemented!();
    }

    // Diese Funktion dient dem Einfügen eines Elementes in die Liste. Hierbei wird das Element definitiv eingefügt.Element
    // TODO: Predecessor implementieren.
    fn insert_into_hashtables(&mut self, _element: Element<Int> ) {
        unimplemented!();
    }

    #[inline]
    fn len(&self) -> usize {
        self.element_list.len()
    }
}

impl PredecessorSet<Int> for STree {
    // Diese Methode fügt ein Element vom Typ Int=i32 in die Datenstruktur ein.
    #[inline]
    fn insert(&mut self,element: Int) {
        let mut new_list_element = Box::new(Element::new(element));
        let pointer_to_new_element: *mut _ = &mut *new_list_element;

        if self.element_list.len() == 0 {
            self.element_list.last = pointer_to_new_element;
            self.element_list.first = Some(new_list_element);
        } else {
            let minimum = self.minimum().unwrap();
            let maximum = self.maximum().unwrap();

            if element < minimum {
                let mut old_first = mem::replace(&mut self.element_list.first, Some(new_list_element)).unwrap();
                let second = mem::replace(&mut old_first.next, None);
                match second {
                    Some(x) => {
                        x.insert_before(pointer_to_new_element);
                    }
                    _ => {}
                }
                self.insert(minimum);
            } else if element > maximum {
                unsafe {(*(*self.element_list.last).prev).insert_after(new_list_element);}
                self.element_list.last = pointer_to_new_element;
                self.insert(maximum);
            } //else {
            // Die hochwertigsten 16 Bits als Root-Array-Index
            let i: usize = (element >> 16) as usize;
            // Die niedrigwertigsten 16 Bits
            let low = element & 0xFFFF;
            // Bits 16 bis 23
            let _j = low >> 8;
            // Die niedrigwertigsten 8 Bits
            let _k = element & 255;

            let _first_level = &mut self.root_table[i];

            // Falls Element kleiner oder größer als das bestehende Minimum/Maximum ist
           // if element < 

            unimplemented!();
            /* Hier kann parallelisiert werden! */
            //self.insert_into_top_table(element);
            //}
        }

        self.element_list.increase_len();
    }

    // Diese Method entfernt ein Element vom Typ Int=i32 aus der Datenstruktur.
    #[inline]
    fn delete(&mut self, _element: Int) {
        unimplemented!();
    }

    // Diese Methode gibt den größten Wert, der echt kleiner als number ist und in der Datenstruktur enthalten ist, aus.
    #[inline]
    fn predecessor(&self, _number: Int) -> Option<Int> {
        unimplemented!();
    }

    // Gibt den kleinsten Wert, der echt größer als number ist und in der Datenstruktur enthalten ist, aus.
    #[inline]
    fn sucessor(&self, _number: Int) -> Option<Int> {
        unimplemented!();
    }

    // Gibt den kleinsten in der Datenstruktur enthaltenen Wert zurück. Dies entspricht dem ersten Wert in der Liste.
    #[inline]
    fn minimum(&self) -> Option<Int> {
        self.element_list.first.as_ref().map(|x| {
            x.elem
        })
    }

    // Gibt den größten in der Datenstruktur enthaltenen Wert zurück. Dies entspricht dem letzten Wert in der Liste.
    #[inline]
    fn maximum(&self) -> Option<Int> {
        if self.element_list.last.is_null() {
            None
        } else {
            unsafe {Some((*(self.element_list).last).elem)}
        }
    }

    // Prüft ob ein Wert in der Datenstruktur enthalten ist.
    #[inline]
    fn contains(&self) -> bool {
        unimplemented!();
    }
}

/* TODO :
- prüfen ob maximum und minimum Methoden ohne Variablen und mit element_list.first, element_list.last nicht schneller sind
- prüfen wie die Datenstruktur im Vergleich zur Datenstruktur von Johannes und der vEB-B-Baum-Kombi abschneidet.
- prüfen ob die 2Mbyte Initialisierungsspeicher durch Verzicht auf Pointer, einen Leistungsschub bringen 

- Statische Implementierung für i40
*/

/* Anpassungen an der Vorlage:
- die Minimum und Maximumwerte, die gespeichert werden, liegen immer als RAW-Pointer vor. In der Root-Ebene kann auf diese mittels element_list.{first,last} zugegriffen werden
- Die HashMap in Level<T,V> ist kein Pointer, der im Falle |Level<V,T>| = 1 auf Element<T> und sonst auf eine HashMap zeigt. 
  Sondern es ext. 2 RAW-Pointer im Level (Min- und Max-Pointer, die Laut Spezifikation sowieso da sein sollten) und eine HashMap.  
- Root_Top befindet sich in der Hauptdatenstruktur, L2_Top und L3_Top befinden sich als lx_top im Struct Level*/

  /* Achtung: Datenstruktur funktioniert "nur" auf Little-Endian-Systemen so wie sie soll. Evtl. ist diese performanter/inperformanter auf Big-Endian-Systemen*/

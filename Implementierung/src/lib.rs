use std::mem;

mod internal;
use internal::{List,Element};
use fnv::FnvHashMap;

pub trait PredecessorList<T> {
    fn insert(&mut self,element: T);
    fn delete(&mut self,element: T);
    fn predecessor(&self,number: T) -> Option<T>;
    fn sucessor(&self,number: T) -> Option<T>;
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>; 
    fn contains(&self) -> bool;
}

pub type Store = i32;
pub struct STree<Store> {
    pub root_table: [Level<Level<Store>>;1 << (8 * mem::size_of::<i32>()/2)],
    // Da die Größe in in Bytes von size_of zurückgegeben wird, mal 8. Durch 32 wegen der Fenstergröße
    pub root_top: [u32; 1 << (8 * mem::size_of::<i32>()/2)/32],
    pub l1_top: [u32; (1 << (8 * mem::size_of::<i32>()/2))/32/32],
    pub l2_top: [u32; ((1 << (8 * mem::size_of::<i32>()/2))/32)/32/32],
    pub element_list: internal::List<Store>,
}

struct Level<T,Store> {
    pub hash_map: FnvHashMap<u8,T>,
    pub maximum: Option<Store>,
    pub minimum: Option<Store>,
}

impl Level<T,Store> {
    fn new() -> Level<T,Store> {
        Level {
            hash_map: FnvHashMap<u8,T>::default(),
            maximum: None,
            minimum: None,
        }
    }
}

impl STree<Store> {
    pub fn new() -> STree<Store> {
        STree {
            element_list: List::new(),
            root_top: [0; 1 << (8 * mem::size_of::<i32>()/2)/32],
            l1_top: [0; (1 << (8 * mem::size_of::<i32>()/2))/32/32],
            l2_top: [0; ((1 << (8 * mem::size_of::<i32>()/2))/32)/32/32],
            root_table: [Level<Level<i32>>::new();(8 * mem::size_of::<i32>()/2)]
        }
    }

    fn locate(&mut self, element: Store) -> Element<Store> {
        unimplemented!();
    }
}

impl PredecessorList<Store> for STree<Store> {
    // Diese Methode fügt ein Element vom Typ Store=i32 in die Datenstruktur ein.
    pub fn insert(&mut self,element: Store) {
        unimplemented!();
    }

    // Diese Method entfernt ein Element vom Typ Store=i32 aus der Datenstruktur.
    pub fn delete(&mut self,element: Store) {
        unimplemented!();
    }

    // Diese Methode gibt den größten Wert, der echt kleiner als number ist und in der Datenstruktur enthalten ist, aus.
    pub fn predecessor(&self,number: Store) -> Option<Store> {
        unimplemented!();
    }

    // Gibt den kleinsten Wert, der echt größer als number ist und in der Datenstruktur enthalten ist, aus.
    pub fn sucessor(&self,number: Store) -> Option<Store> {
        unimplemented!();
    }

    // Gibt den kleinsten in der Datenstruktur enthaltenen Wert zurück. Dies entspricht dem ersten Wert in der Liste.
    pub fn minimum(&self) -> Option<Store> {
        unimplemented!();
    }

    // Gibt den größten in der Datenstruktur enthaltenen Wert zurück. Dies entspricht dem letzten Wert in der Liste.
    pub fn maximum(&self) -> Option<Store> {
        if element_list.last.is_null() {
            None
        } else {
            Some((*element_list.last).elem.copy())
        }
    }

    // Prüft ob ein Wert in der Datenstruktur enthalten ist.
    pub fn contains(&self) -> bool {
        unimplemented!();
    }
}

/* TODO :
- adde 
- prüfen ob maximum und minimum Methoden ohne Variablen und mit element_list.first, element_list.last nicht schneller sind
- prüfen wie die Datenstruktur im Vergleich zur Datenstruktur in C++ abschneidet
- prüfen ob die 2Mbyte Initialisierungsspeicher durch Verzicht auf Pointer, einen Leistungsschub bringen 
- herausfinden ob X86 CPUs 32-Bit oder 64-Bit Instruktionen zum Ermitteln von bestimmten Bits haben -> leading Zeros ext.
*/
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

impl PredecessorList<Store> for STree<Store> {
    pub fn new() -> STree<Store> {
        STree {
            element_list: List::new(),
            root_top: [0; 1 << (8 * mem::size_of::<i32>()/2)/32],
            l1_top: [0; (1 << (8 * mem::size_of::<i32>()/2))/32/32],
            l2_top: [0; ((1 << (8 * mem::size_of::<i32>()/2))/32)/32/32],
            root_table: [Level<Level<i32>>::new();(8 * mem::size_of::<i32>()/2)]
        }
    }
    pub fn insert(&mut self,element: T);
    pub fn delete(&mut self,element: T);
    pub fn predecessor(&self,number: T) -> Option<T>;
    pub fn sucessor(&self,number: T) -> Option<T>;
    pub fn minimum(&self) -> Option<T>;
    pub fn maximum(&self) -> Option<T>; 
    pub fn contains(&self) -> bool;
}

/* TODO :
- adde 
- prüfen ob maximum und minimum Methoden ohne Variablen und mit element_list.first, element_list.last nicht schneller sind
- prüfen wie die Datenstruktur im Vergleich zur Datenstruktur in C++ abschneidet
- prüfen ob die 2Mbyte Initialisierungsspeicher durch Verzicht auf Pointer, einen Leistungsschub bringen 
- herausfinden ob X86 CPUs 32-Bit oder 64-Bit Instruktionen zum Ermitteln von bestimmten Bits haben -> leading Zeros ext.
- u32 Bit je Zelle in root_top (weniger Top Arrays evtl. in der Praxis notwendig!)
*/
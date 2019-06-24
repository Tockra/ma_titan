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
    pub minimum: Store,
    pub maximum: Store,
    pub count: u32,
    pub root_table: [FnvHashMap<u8,FnvHashMap<u8,u32>>;1 << (8 * mem::size_of::<i32>()/2)],
    // Uff was und warum passiert hier das?
    pub root_top: [u32; 1 << (8 * mem::size_of::<i32>()/2)/32],
    pub l1_top: [u32; (1 << (8 * mem::size_of::<i32>()/2))/32/8],
    pub l2_top: [u32; ((1 << (8 * mem::size_of::<i32>()/2))/32)/32/8],
    pub element_list: internal::List<Store>,
}

/* TODO :
- adde 
- prüfen ob maximum und minimum Methoden ohne Variablen und mit element_list.first, element_list.last nicht schneller sind
- prüfen wie die Datenstruktur im Vergleich zur Datenstruktur in C++ abschneidet
- prüfen ob die 2Mbyte Initialisierungsspeicher durch Verzicht auf Pointer, einen Leistungsschub bringen 
- herausfinden ob X86 CPUs 32-Bit oder 64-Bit Instruktionen zum Ermitteln von bestimmten Bits haben -> leading Zeros ext.
- u32 Bit je Zelle in root_top (weniger Top Arrays evtl. in der Praxis notwendig!)
*/
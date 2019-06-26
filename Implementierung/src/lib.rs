#![allow(dead_code)]

mod internal;
use internal::{List,Element};

use fnv::FnvHashMap;
use std::ptr;
use std::mem::{self, MaybeUninit};


pub trait PredecessorList<T> {
    fn insert(&mut self,element: T);
    fn delete(&mut self,element: T);
    fn predecessor(&self,number: T) -> Option<T>;
    fn sucessor(&self,number: T) -> Option<T>;
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>; 
    fn contains(&self) -> bool;
}

pub type Int = i32;
type SecondLevel = Level<Int,Int>;
type FirstLevel = Level<SecondLevel,Int>;
pub struct STree {
    root_table: [MaybeUninit<FirstLevel>;1 << (8 * mem::size_of::<Int>()/2)],
    // Da die Größe in in Bytes von size_of zurückgegeben wird, mal 8. Durch 32 wegen der Fenstergröße
    root_top: [u32; 1 << (8 * mem::size_of::<Int>()/2)/32],
    l1_top: [u32; (1 << (8 * mem::size_of::<Int>()/2))/32/32],
    l2_top: [u32; ((1 << (8 * mem::size_of::<Int>()/2))/32)/32/32],
    element_list: internal::List<Int>,
}

// Implementiert die zwei Level unter der Root-Tabelle. Diese besitzen ein Maximum- und ein Minimumpointer und ggf. eine hash_map, wenn *minimum!= *maximum
struct Level<T,V> {
    pub hash_map: FnvHashMap<u8,T>,
    pub maximum: *mut Element<V>,
    pub minimum: *mut Element<V>,
}

impl<T,V> Level<T,V> {
    #[inline]
    fn new() -> Level<T,Int> {
        Level {
            hash_map: (FnvHashMap::<u8,T>::default()),
            maximum: ptr::null_mut(),
            minimum: ptr::null_mut(),
        }
    }
}

impl STree {
    #[inline]
    pub fn new() -> STree {
        let mut data: [MaybeUninit<FirstLevel>; 1 << (8 * mem::size_of::<i32>()/2)] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        for elem in &mut data[..] {
            unsafe { 
                ptr::write(elem.as_mut_ptr(), FirstLevel::new()); 
            }
        }
        STree {
            element_list: List::new(),
            root_top: [0; 1 << (8 * mem::size_of::<i32>()/2)/32],
            l1_top: [0; (1 << (8 * mem::size_of::<i32>()/2))/32/32],
            l2_top: [0; ((1 << (8 * mem::size_of::<i32>()/2))/32)/32/32],
            root_table: data,
        }
    }

    #[inline]
    pub fn locate(&mut self, _element: Int) -> Element<Int> {
        unimplemented!();
    }
}

impl PredecessorList<Int> for STree {
    // Diese Methode fügt ein Element vom Typ Int=i32 in die Datenstruktur ein.
    #[inline]
    pub fn insert(&mut self,element: Int) {
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
            } else {
                /* Der wirklich interessante Insert Stuff */
                insert_into_top_table(element);
            }
        }

        self.element_list.increase_len();
    }

    // Diese Methode setzt die benötigten Bits in der Root-Top-Tabelle und in L1-Top und L2-Top
    #[inline]
    fn insert_into_top_table(&mut self, _element: Int) {
        unimplemented!();
    }

    // Diese Method entfernt ein Element vom Typ Int=i32 aus der Datenstruktur.
    #[inline]
    fn delete(&mut self,_element: Int) {
        unimplemented!();
    }

    // Diese Methode gibt den größten Wert, der echt kleiner als number ist und in der Datenstruktur enthalten ist, aus.
    #[inline]
    fn predecessor(&self,_number: Int) -> Option<Int> {
        unimplemented!();
    }

    // Gibt den kleinsten Wert, der echt größer als number ist und in der Datenstruktur enthalten ist, aus.
    #[inline]
    fn sucessor(&self,_number: Int) -> Option<Int> {
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
- adde 
- prüfen ob maximum und minimum Methoden ohne Variablen und mit element_list.first, element_list.last nicht schneller sind
- prüfen wie die Datenstruktur im Vergleich zur Datenstruktur in C++ abschneidet
- prüfen ob die 2Mbyte Initialisierungsspeicher durch Verzicht auf Pointer, einen Leistungsschub bringen 
- herausfinden ob X86 CPUs 32-Bit oder 64-Bit Instruktionen zum Ermitteln von bestimmten Bits haben -> leading Zeros ext.
*/

/* Anpassungen an der Vorlage:
- die Minimum und Maximumwerte, die gespeichert werden, liegen immer als RAW-Pointer vor. In der Root-Ebene kann auf diese mittels element_list.{first,last} zugegriffen werden
- Die HashMap in Level<T,V> ist kein Pointer, der im Falle |Level<V,T>| = 1 auf Element<T> und sonst auf eine HashMap zeigt. 
  Sondern es ext. 2 RAW-Pointer im Level (Min- und Max-Pointer, die Laut Spezifikation sowieso da sein sollten) und eine HashMap.  */
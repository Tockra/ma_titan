#![allow(dead_code)]  
use std::mem::{self, MaybeUninit};
use std::ptr;
use std::collections::HashMap;
use ux::{i40,u10,u40};
use boomphf::Mphf;

pub struct List<T> {
    pub first: Option<Box<Element<T>>>,
    pub last: *mut Element<T>,
    pub len: usize,
}

pub const fn root_size<T>() -> usize {
    1 << 8*mem::size_of::<T>() / 2
}

pub trait PredecessorSet<T> {
    fn insert(&mut self,element: T);
    fn delete(&mut self,element: T);
    fn predecessor(&self,number: T) -> Option<T>;
    fn sucessor(&self,number: T) -> Option<T>; // Optional
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>; 
    fn contains(&self, number: T) -> bool;
}

pub struct Element<T> {
    pub next: Option<Box<Element<T>>>,
    pub prev: *mut Element<T>,
    pub elem: T,
}

impl<T> List<T> {
    #[inline]
    pub fn new() -> Self {
        List {
            first: None,
            last: ptr::null_mut(),
            len: 0,
        }
    }

    // Fügt am Ende der Liste ein Element mit Wert 'elem' ein.
    #[inline]
    pub fn insert_at_end(&mut self, elem: T) {
        if self.len() == 0 {
            let mut node = Box::new(Element {
                next: None,
                prev: ptr::null_mut(),
                elem
            });
            self.last = &mut *node;
            self.first = Some(node);
        }
        else {
            let mut node = Box::new(Element {
                next: None,
                prev: self.last,
                elem
            });

            let tmp: *mut _ = &mut *node;
            unsafe {
                (*(self.last)).next = Some(node);
            }
            self.last = tmp;
        }
        self.increase_len();
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        self.first.take().map(|head| {
            let mut head = *head;
            self.first = head.next;

            if self.first.is_none() {
                self.last = ptr::null_mut();
            }
            head.prev =  ptr::null_mut();
            self.decrease_len();
            head.elem
        })
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        if self.last.is_null() {
            return None;
        }

        unsafe {
            let node = Box::from_raw(self.last);
            
            if node.prev.is_null() {
                self.first = None;
            } else {
                (*node.prev).next = None;
            };
            
            self.last = node.prev;
            self.decrease_len();
            Some(node.elem)
        }
    }

    #[inline]
    pub fn increase_len(&mut self) {
        self.len += 1;
    }

    #[inline]
    pub fn decrease_len(&mut self) {
        if self.len == 0 {
            panic!("Die Länge einer internal::List kann nicht 0 unterschreiten!");
        }
        self.len -= 1;
    }
}

impl<T> Element<T> {
    #[inline]
    pub fn new(elem: T) -> Self {
        Element {
            next: None,
            prev: ptr::null_mut(),
            elem,
        }
    }

    /* Hinter dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
        muss beachtet werden, dass extern die Size angepasst werden muss und der Last-Zeiger evtl. angepasst werden muss. */
    #[inline]
    pub fn insert_after(&mut self, mut elem: Box<Element<T>>) {
        elem.prev = &mut *self;
        self.next = Some(elem);
    }

    /* Vor dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
        muss beachtet werden, dass extern die Size angepasst werden muss und ggf. der First-Zeiger angepasst werden muss.  */
    #[inline]
    pub fn insert_before(mut self, elem: *mut Element<T>) {
        unsafe {
            self.prev = &mut *elem;
            (*elem).next = Some(Box::new(self));
        }
    }
}

type SecondLevelBuild = PerfectHashBuilderLevel<usize>;
type FirstLevelBuild = PerfectHashBuilderLevel<SecondLevelBuild>;
type Int = u40;
pub struct PerfectHashBuilder {
    root_table: [FirstLevelBuild; root_size::<Int>()],
    root_indexs: Vec<usize>,
}

pub struct PerfectHashBuilderLevel<T> {
    pub hash_map: std::collections::HashMap<u10,T>,
    pub objects: Vec<u10>,
    pub maximum: usize,
    pub minimum: usize,
    pub lx_top: Vec<u64>,
}
impl<T> PerfectHashBuilderLevel<T> {
    #[inline]
    pub fn new(level: usize) -> PerfectHashBuilderLevel<T> {
        PerfectHashBuilderLevel {
            hash_map: (HashMap::<u10,T>::default()),
            objects: vec![],
            maximum: 0,
            minimum: 0,
            lx_top: vec![0;level],
        }
    }
}
impl PerfectHashBuilder {
    pub fn new(objects: Vec<Int>) ->  PerfectHashBuilder{
        let mut root_indexs = vec![];
        let mut root_table = {
            let mut data: [MaybeUninit<FirstLevelBuild>; root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut data[..] {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), FirstLevelBuild::new((1<<10)/64)); 
                }
            }

            unsafe { 
                mem::transmute::<_, [FirstLevelBuild; root_size::<Int>()]>(data) 
            }
        };
        for element in objects {
            let (i,j,k) = split_integer_down(element);

            root_indexs.push(i);
            root_table[i].objects.push(j);
            
            if !root_table[i].hash_map.contains_key(&j) {
                root_table[i].hash_map.insert(j,SecondLevelBuild::new((1<<10)/64));
            }
            root_table[i].hash_map.get_mut(&j).unwrap().objects.push(k);
        }
        PerfectHashBuilder {root_table: root_table, root_indexs: root_indexs}
    }

    pub fn build(self) -> [super::statics::FirstLevel; root_size::<Int>()] {
        let mut result: [super::statics::FirstLevel; root_size::<Int>()] = {
            let mut data: [MaybeUninit<super::statics::FirstLevel>; super::internal::root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for (i, elem) in data.iter_mut().enumerate() {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), super::statics::FirstLevel::new((1<<10)/64, Some(self.root_table[i].objects.clone()))); 
                }
            }

            unsafe { 
                mem::transmute::<_, [super::statics::FirstLevel; super::internal::root_size::<Int>()]>(data) 
            }
        };
        for i in self.root_indexs {
            for _ in self.root_table[i].objects.clone() {
                result[i].objects.push(super::statics::SecondLevel::new(1<<10, None));
            }

            for key in self.root_table[i].objects.clone() {
                result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].hasher = 
                    Some(Mphf::new_parallel(2.0,&self.root_table[i].hash_map.get(&key).unwrap().objects.clone(), None));
            }

        }
        result
    }
}

#[inline]
pub fn split_integer_down(element: u40) -> (usize,u10,u10) {
    // TODO: Achtung funktioniert nicht korrekt mit negativen Zahlen
    let i: usize = (u64::from(element) >> 20) as usize;
    // Die niedrigwertigsten 16 Bits element[16..31]
    let low = u64::from(element) & 0xFFFFF;
    // Bits 16 bis 23 element[8..15]
    let j: u10 = u10::new((low >> 10) as u16) ;
    // Die niedrigwertigsten 8 Bits element[0..7]
    let k: u10 = u10::new((u64::from(element) & 0x3FF) as u16);
    (i, j, k) 
}



#[cfg(test)]
pub mod tests {
    #[test]
    pub fn test_insert_and_pop() {
        let mut l = super::List::new();
        fill_list(&mut l);
        assert_eq!(l.len(), 6);

        let len = l.len();
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);

        fill_list(&mut l);
        
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), 0);

        fill_list(&mut l);

        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 30);
    }

    #[inline]
    fn fill_list(l: &mut super::List<i32>) {
        l.insert_at_end(-20);
        l.insert_at_end(10);
        l.insert_at_end(20);
        l.insert_at_end(30);
        l.insert_at_end(40);
        l.insert_at_end(50);
    }

    #[test]
    pub fn test_insert_before_first() {
        let mut l = super::List::new();
        fill_list(&mut l);
        let mut elem = Box::new(super::Element::new(23));
        l.first.unwrap().insert_before(&mut *elem);
        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
        l.first = Some(elem);

        assert_eq!(l.pop_front().unwrap(), 23);
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);

        fill_list(&mut l);
        let mut elem = Box::new(super::Element::new(23));
        l.first.unwrap().insert_before(&mut *elem);
        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
        l.first = Some(elem);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20);
        assert_eq!(l.pop_back().unwrap(), 23);

        fill_list(&mut l);
        let mut elem = Box::new(super::Element::new(23));
        l.first.unwrap().insert_before(&mut *elem);
        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
        l.first = Some(elem);
        assert_eq!(l.pop_front().unwrap(), 23);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20);
        
        
    }

    #[test]
    pub fn test_insert_after_last() {
        let mut l = super::List::new();
        fill_list(&mut l);
        let mut elem = super::Element::new(23);
        let tmp = l.last;
        l.last = &mut elem;
        unsafe {
            (*tmp).insert_after(Box::new(elem));
        }

        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!

        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);
        assert_eq!(l.pop_front().unwrap(), 23);

        fill_list(&mut l);
        let mut elem = super::Element::new(23);
        let tmp = l.last;
        l.last = &mut elem;
        unsafe {
            (*tmp).insert_after(Box::new(elem));
        }
        assert_eq!(l.pop_back().unwrap(), 23);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20);
        

        fill_list(&mut l);
        let mut elem = super::Element::new(23);
        let tmp = l.last;
        l.last = &mut elem;
        unsafe {
            (*tmp).insert_after(Box::new(elem));
        }
        assert_eq!(l.pop_back().unwrap(), 23);
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);  
    }

}

#![allow(dead_code)]  
use std::mem::{self};
use std::ptr;

use ux::{u10,u40};

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

    pub fn insert_after(&mut self, _element: &mut Element<T>, _to_insert: Box<Element<T>>) {
        unimplemented!();
    }

    pub fn insert_before(&mut self, _element: Box<Element<T>>, _to_insert: Box<Element<T>>) {
        unimplemented!();
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
        unsafe {
            let node = self.last.as_mut()
                .and_then(|last| last.prev.as_mut())
                .and_then(|second_last| second_last.next.take())
                .or_else(|| {
                    self.last = ptr::null_mut();
                    self.first.take()
                })?;

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
        elem.prev = self;
        match self.next.as_mut() {
            Some(x) => {x.prev = &mut *elem},
            None => {},
        };
        elem.next = mem::replace(&mut self.next, None);
        self.next = Some(elem);
    }

    /* Vor dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
        muss beachtet werden, dass extern die Size angepasst werden muss und ggf. der First-Zeiger angepasst werden muss.  */
    #[inline]
    pub fn insert_before(mut self: Box<Self>, mut elem: Box<Element<T>>) -> Result<(), Box<Element<T>>> {
        elem.prev = self.prev;
        self.prev = &mut *elem;
        elem.next = Some(self);
        if elem.prev.is_null() {
                Err(elem)
        } else {
            unsafe {
                let mut before = &mut (*elem.prev);
                before.next = Some(elem);
            }
            Ok(())
        }
    }
}


pub trait Splittable<T,V> {
    fn split_integer_down(&self) -> (T,V,V);
}

impl Splittable<usize,u10> for u40 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u10,u10) {
        // TODO: Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 20) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = u64::from(*self) & 0xFFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u10 = u10::new((low >> 10) as u16) ;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u10 = u10::new((u64::from(*self) & 0x3FF) as u16);
        (i, j, k) 
    }
}

impl Splittable<usize,u8> for i32 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u8,u8) {
        let i: usize = (*self >> 16) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = *self & 0xFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u8 = (low >> 8) as u8;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u8 = (*self & 255) as u8;
        (i,j,k)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_insert_and_pop() {
        let mut l = super::List::new();
        fill_list(&mut l);

        //let len = l.len();
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
        assert_eq!(l.pop_back().unwrap(), -20);

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
        assert_eq!(l.len(), 6);
    }

    #[test]
    pub fn test_insert_before_first() {
        let mut l = super::List::new();
        fill_list(&mut l);
        let elem = Box::new(super::Element::new(23));
        match l.first.take().unwrap().insert_before(elem) {
            Err(elem) => {l.first = Some(elem);},
            _ => {},
        }
        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
        l.increase_len();
        assert_eq!(l.len(), 7);
        
        assert_eq!(l.pop_front().unwrap(), 23);
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);

        fill_list(&mut l);
        let elem = Box::new(super::Element::new(23));
        match l.first.take().unwrap().insert_before(elem) {
            Err(elem) => {l.first = Some(elem);},
            _ => {},
        }
        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
        l.increase_len();
        assert_eq!(l.len(), 7);

        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20);
        assert_eq!(l.pop_back().unwrap(), 23);

        fill_list(&mut l);
        let elem = Box::new(super::Element::new(23));
        match l.first.take().unwrap().insert_before(elem) {
            Err(elem) => {l.first = Some(elem);},
            _ => {},
        }
        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
        
        l.increase_len();
        assert_eq!(l.len(), 7);

        assert_eq!(l.pop_front().unwrap(), 23);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20)
        
        
    }

    #[test]
    pub fn test_insert_after_last() {
        let mut l = super::List::new();
        fill_list(&mut l);
        let mut elem = Box::new(super::Element::new(23));
        let tmp = l.last;
        l.last = &mut *elem;
        unsafe {
            (*tmp).insert_after(elem);
        }
        l.increase_len();

        // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!

        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);
        assert_eq!(l.pop_front().unwrap(), 23);

        fill_list(&mut l);
        let mut elem = Box::new(super::Element::new(23));
        let tmp = l.last;
        l.last = &mut *elem;
        unsafe {
            (*tmp).insert_after(elem);
        }
        l.increase_len();
        assert_eq!(l.pop_back().unwrap(), 23);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20);
        

        fill_list(&mut l);
        let mut elem = Box::new(super::Element::new(23));
        let tmp = l.last;
        l.last = &mut *elem;
        unsafe {
            (*tmp).insert_after(elem);
        }
        l.increase_len();
        assert_eq!(l.pop_back().unwrap(), 23);
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);  
}

}

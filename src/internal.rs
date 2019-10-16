#![allow(dead_code)]  
use std::mem::{self};
use std::ptr;

use uint::{u40, u48};

pub struct List<T> {
    pub first: Option<Box<Element<T>>>,
    pub last: *mut Element<T>,
    pub len: usize,
}

pub trait PredecessorSet<T> {
    fn insert(&mut self,element: T);
    fn delete(&mut self,element: T);
    fn predecessor(&self,number: T) -> Option<T>;
    fn successor(&self,number: T) -> Option<T>; // Optional
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

    pub fn insert_after(&mut self, element: &mut Element<T>, mut to_insert: Box<Element<T>>) {
        to_insert.prev = element;
        match element.next.as_mut() {
            Some(x) => {
                x.prev = &mut *to_insert;
            },
            None => {
                self.last = &mut *to_insert;
            },
        };
        to_insert.next = mem::replace(&mut element.next, None);
        element.next = Some(to_insert);
        self.increase_len();
    }

    // TODO Hier reicht eine Referenz, denn mittels der Referenz kann man auf ref.prev.next zugreifen, was dann den richtigen Wert entspricht.
    pub fn insert_before(&mut self, element: &mut Element<T>, mut to_insert: Box<Element<T>>) {
        to_insert.prev = element.prev;
        if element.prev.is_null() {
            element.prev = &mut *to_insert;
            to_insert.next = self.first.take();
            self.first = Some(to_insert);
        } else {
            unsafe {
                let mut before = &mut (*element.prev);
                element.prev = &mut *to_insert;
                to_insert.next = before.next.take();
                before.next = Some(to_insert);
            }
        }
        
        self.increase_len();
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
            let head = *head;
            self.first = head.next;

            if self.first.is_none() {
                self.last = ptr::null_mut();
            } else {
                self.first.as_mut().unwrap().prev =  ptr::null_mut();
            }
            
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


pub trait Splittable {
    fn split_integer_down(&self) -> (usize,u16,u16);
}

impl Splittable for u40 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        // TODO: Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 20) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = u64::from(*self) & 0xFFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u16 = (low >> 10) as u16 ;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u16 = (u64::from(*self) & 0x3FF) as u16;
        (i, j, k) 
    }
}

impl Splittable for u48 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        // TODO: Achtung funktioniert nicht korrekt mit negativen Zahlen
        let i: usize = u64::from(*self >> 24) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = u64::from(*self) & 0xFFFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u16 = (low >> 12) as u16 ;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u16 = (u64::from(*self) & 0xFFF) as u16;
        (i, j, k) 
    }
}

impl Splittable for u32 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        let i: usize = (*self >> 16) as usize;
        // Die niedrigwertigsten 16 Bits element[16..31]
        let low = *self & 0xFFFF;
        // Bits 16 bis 23 element[8..15]
        let j: u16 = (low >> 8) as u16;
        // Die niedrigwertigsten 8 Bits element[0..7]
        let k: u16 = (*self & 255) as u16;
        (i,j,k)
    }
}

impl Splittable for u64 {
    #[inline]
    fn split_integer_down(&self) -> (usize,u16,u16) {
        let i: usize = (*self >> 32) as usize;
        // Die niedrigwertigsten 32 Bits element[32..63]
        let low = *self & 0xFFFFFFFF;
        // Bits 16 bis 32
        let j: u16 = (low >> 16) as u16;
        // Die niedrigwertigsten 16 Bits element[0..15]
        let k: u16 = (*self & 0xFFFF) as u16;
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
        unsafe {
            let first = &mut *(*(*(*(*(*l.last).prev).prev).prev).prev).prev;
            l.insert_before(first,elem);
        }
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
        unsafe {
            let first = &mut *(*(*(*(*(*l.last).prev).prev).prev).prev).prev;
            l.insert_before(first,elem);
        }
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
        unsafe {
            let first = &mut *(*(*(*(*(*l.last).prev).prev).prev).prev).prev;
            l.insert_before(first,elem);
        }
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
        let elem = Box::new(super::Element::new(23));
        unsafe {
            let last = &mut (*l.last);
            l.insert_after(last,elem);
        }
        
        assert_eq!(l.len(), 7);
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);
        assert_eq!(l.pop_front().unwrap(), 23);


        fill_list(&mut l);
        let elem = Box::new(super::Element::new(23));
        unsafe {
            let last = &mut (*l.last);
            l.insert_after(last,elem);
        }
        assert_eq!(l.len(), 7);
        assert_eq!(l.pop_back().unwrap(), 23);
        assert_eq!(l.pop_back().unwrap(), 50);
        assert_eq!(l.pop_back().unwrap(), 40);
        assert_eq!(l.pop_back().unwrap(), 30);
        assert_eq!(l.pop_back().unwrap(), 20);
        assert_eq!(l.pop_back().unwrap(), 10);
        assert_eq!(l.pop_back().unwrap(), -20);
        

        fill_list(&mut l);
        let elem = Box::new(super::Element::new(23));
        unsafe {
            let last = &mut (*l.last);
            l.insert_after(last,elem);
        }
        assert_eq!(l.len(), 7);
        assert_eq!(l.pop_back().unwrap(), 23);
        assert_eq!(l.pop_front().unwrap(), -20);
        assert_eq!(l.pop_front().unwrap(), 10);
        assert_eq!(l.pop_front().unwrap(), 20);
        assert_eq!(l.pop_front().unwrap(), 30);
        assert_eq!(l.pop_front().unwrap(), 40);
        assert_eq!(l.pop_front().unwrap(), 50);  
}

}


pub enum PointerEnum<T: 'static, E: 'static> {
    First(&'static mut T),
    Second(&'static mut E)
}

/// Dieser Struct beinhaltet einen RAW-Pointer, der entweder auf ein T oder ein E Objekt zeigt. Wichtig ist hierbei, dass T mit einem Vielfachen von 4 alligned werden muss!
pub struct Pointer<T,E> {
    pointer: *mut T,
    phantom: std::marker::PhantomData<E>,
}

impl<T:'static + Clone,E:'static + Clone> Clone for Pointer<T,E> {
    fn clone(&self) -> Self {
        if self.pointer.is_null() {
            Self::null()
        } else {
            match self.get() {
                PointerEnum::First(x) => Self::from_first(Box::new(x.clone())),
                PointerEnum::Second(x) => Self::from_second(Box::new(x.clone())),
            }
        }
    }
}

impl<T,E> Drop for Pointer<T,E> {
    fn drop(&mut self) {
        if self.pointer.is_null() {
            return;
        }

        if (self.pointer as usize % 4) == 0 {
            unsafe { Box::from_raw(self.pointer) };
        } else {
            assert!((self.pointer as usize % 4) == 1);

            unsafe { Box::from_raw((self.pointer as usize -1) as *mut E) };
        }
    }
}

impl<T,E> Pointer<T,E> {
    pub fn from_first(b: Box<T>) -> Self {
        Self {
            pointer: Box::into_raw(b),
            phantom: std::marker::PhantomData
        }
    }

    pub fn from_second(b: Box<E>) -> Self {
        let pointer = Box::into_raw(b);
        assert!((pointer as usize % 4) == 0);

        let pointer = (pointer as usize + 1) as *mut T;
        Self {
            pointer: pointer,
            phantom: std::marker::PhantomData
        }
    }


    pub fn get(&self) -> PointerEnum<T,E> {
        if self.pointer.is_null() {
            panic!("Pointer<T> is null!");
        }

        if (self.pointer as usize % 4) == 0 {
            unsafe {PointerEnum::First(&mut (*self.pointer))}
        } else {
            assert!((self.pointer as usize % 4) == 1);

            unsafe {PointerEnum::Second(&mut *((self.pointer as usize -1) as *mut E))}
        }
    }

    pub fn null() -> Self {
        Self {
            pointer: std::ptr::null_mut(),
            phantom: std::marker::PhantomData
        }
    }

    pub fn is_null(&self) -> bool {
        self.pointer.is_null()
    }
}
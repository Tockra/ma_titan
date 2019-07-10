use std::ptr;
pub struct List<T> {
    pub first: Option<Box<Element<T>>>,
    pub last: *mut Element<T>,
    pub len: usize,
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

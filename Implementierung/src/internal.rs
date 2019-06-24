
mod internal {
    use std::ptr;
    pub struct List<T> {
        first: Option<Box<Element<T>>>,
        last: *mut Element<T>,
        len: usize,
    }

    pub struct Element<T> {
        next: Option<Box<Element<T>>>,
        prev: *mut Element<T>,
        elem: T,
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            List {
                first: None,
                last: ptr::null_mut(),
                len: 0,
            }
        }

        /* Fügt am Ende der Liste ein Element mit Wert 'elem' ein.*/
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
            self.len += 1;
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn pop_front(&mut self) -> Option<T> {
            self.first.take().map(|head| {
                let mut head = *head;
                self.first = head.next;

                if self.first.is_none() {
                    self.last = ptr::null_mut();
                }
                head.prev =  ptr::null_mut();
                head.elem
            })
        }

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

                Some(node.elem)
            }
        }
    }

    impl<T> Element<T> {
        fn new(elem: T) -> Self {
            Element {
                next: None,
                prev: ptr::null_mut(),
                elem,
            }
        }

        /* Verbindet das ehemals erste Element (self) mit dem neuen ersten Element 'elem'. Hier bei
            muss beachtet werden, dass extern die Size und der First-Zeiger angepasst werden müssen.
            self muss vom first Zeiger in diese Methode gemovet werden. */
        pub fn connect_with_first(mut self, elem: *mut Element<T>) {
            unsafe {
                self.prev = &mut *elem;
                (*elem).next = Some(Box::new(self));
            }
        }

        /*Verbindet das ehemals letzte Element (self) mit dem neuen letzten Element 'elem'. Hier bei
            muss beachtet werden, dass extern die Size und der First-Zeiger angepasst werden müssen.*/
        pub fn connect_with_last(&mut self, mut elem: Box<Element<T>>) {
            elem.prev = &mut *self;
            self.next = Some(elem);
        }

        /* Hinter dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
            muss beachtet werden, dass extern die Size angepasst werden müssen. */
        fn insert_after(&mut self, elem: Box<Element<T>>) {
            unimplemented!();
        }

        /* Vor dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
            muss beachtet werden, dass extern die Size angepasst werden müssen.  */
        fn insert_before(&mut self, elem: *mut Element<T>) {
            unimplemented!();
        }
    }

   // #[cfg(test)]
    pub mod tests {
        //#[test]
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

        //#[test]
        pub fn test_connect_with_first() {
            let mut l = super::List::new();
            fill_list(&mut l);
            let mut elem = Box::new(super::Element::new(23));
            l.first.unwrap().connect_with_first(&mut *elem);
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
            l.first.unwrap().connect_with_first(&mut *elem);
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
            l.first.unwrap().connect_with_first(&mut *elem);
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

        pub fn test_connect_with_last() {
            let mut l = super::List::new();
            fill_list(&mut l);
            let mut elem = super::Element::new(23);
            let tmp = l.last;
            l.last = &mut elem;
            unsafe {
                (*tmp).connect_with_last(Box::new(elem));
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
                (*tmp).connect_with_last(Box::new(elem));
            }
            assert_eq!(l.pop_back().unwrap(), 23);
            assert_eq!(l.pop_back().unwrap(), 50);
            assert_eq!(l.pop_back().unwrap(), 40);
            assert_eq!(l.pop_back().unwrap(), 30);
            assert_eq!(l.pop_back().unwrap(), 20);
            assert_eq!(l.pop_back().unwrap(), 10);
            assert_eq!(l.pop_back().unwrap(), -20);
         

            fill_list(&mut l);
            let mut elem = Box::new(super::Element::new(23));
            l.first.unwrap().connect_with_first(&mut *elem);
            // Das kann die Methode aus technischen Gründen nicht übernehmen und muss extern gemacht werden!
            l.first = Some(elem);
            assert_eq!(l.pop_back().unwrap(), 23);
            assert_eq!(l.pop_front().unwrap(), -20);
            assert_eq!(l.pop_front().unwrap(), 10);
            assert_eq!(l.pop_front().unwrap(), 20);
            assert_eq!(l.pop_front().unwrap(), 30);
            assert_eq!(l.pop_front().unwrap(), 40);
            assert_eq!(l.pop_front().unwrap(), 50);
            
            
        }
    }
}
use self::internal::tests;
    fn main() {
        tests::test_connect_with_last();
    }
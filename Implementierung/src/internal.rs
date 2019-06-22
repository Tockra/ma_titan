
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
                    self.first = ptr::null_mut();
                } else {
                    (*node.prev).next = ptr::null_mut();
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
        fn connect_with_first(mut self, elem: *mut Element<T>) {
            unsafe {
                self.prev = &mut *elem;
                (*elem).next = Some(Box::new(self));
            }
        }

        /*Verbindet das ehemals letzte Element (self) mit dem neuen letzten Element 'elem'. Hier bei
            muss beachtet werden, dass extern die Size und der First-Zeiger angepasst werden müssen.*/
        fn connect_with_last(&mut self, mut elem: Box<Element<T>>) {
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
        pub fn test_connect_with_first() {
            let mut l = super::List::new();
            l.insert_at_end(0);
            l.insert_at_end(10);
            l.insert_at_end(20);
            l.insert_at_end(30);
            l.insert_at_end(40);
            l.insert_at_end(50);
            assert_eq!(l.len(), 6);

            let len = l.len();
            assert_eq!(l.pop_front, 0);
            assert_eq!(l.pop_front, 10);
            assert_eq!(l.pop_front, 20);
            assert_eq!(l.pop_front, 30);
            assert_eq!(l.pop_front, 40);
            assert_eq!(l.pop_front, 50);
        }
    }
}
use self::internal::tests;
    fn main() {
        tests::test_connect_with_first();
    }
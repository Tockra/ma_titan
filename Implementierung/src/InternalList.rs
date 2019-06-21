
mod internal {
    use std::ptr;
    pub struct List<T> {
        head: Option<Box<Element<T>>>,
        tail: *mut Element<T>,
        len: usize,
    }

    pub struct Element<T> {
        next: Option<Box<Element<T>>>,
        prev: *mut Element<T>,
        elem: T,
    }



    impl<T> Element<T> {
        fn new(elem: T) -> Self {
            Element {
                next: None,
                prev: ptr::null_mut(),
                elem,
            }
        }

        /* Verbindet das ehemals erste Element (self) mit dem neuen ersten Element elem. Hier bei
            muss beachtet werden, dass extern die Size und der First-Zeiger angepasst werden müssen. */
        fn connect_with_first(elem: *mut Element<T>) {
            unimplemented!();
        }

        /*Verbindet das ehemals letzte Element (self) mit dem neuen ersten Element elem. Hier bei
            muss beachtet werden, dass extern die Size und der First-Zeiger angepasst werden müssen.*/
        fn connect_with_last(elem: Box<Element<T>>) {
            unimplemented!();
        }

        /* Hinter dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
            muss beachtet werden, dass extern die Size angepasst werden müssen. */
        fn insert_after(elem: Box<Element<T>>) {
            unimplemented!();
        }

        /* Vor dem Element (self) wird das Element elem in die Liste eingefügt. Hier bei
            muss beachtet werden, dass extern die Size angepasst werden müssen.  */
        fn insert_before(elem: *mut Element<T>) {
            unimplemented!();
        }
    }

}
    fn main() {
        
    }
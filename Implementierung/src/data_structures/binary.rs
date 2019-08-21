use uint::u40;
use crate::help::internal::{PredecessorSetStatic};

pub type Int = u40;
pub struct BinarySearch {
    element_list: Box<[Int]>
}

impl PredecessorSetStatic<Int> for BinarySearch {
    fn new(elements: Vec<Int>) -> Self {
        Self {
            element_list: elements.into_boxed_slice(),
        }
    }

    fn predecessor(&self,number: Int) -> Option<Int> {
        if self.element_list.len() == 0 {
            None
        } else {
            self.pred(number, 0, self.element_list.len()-1)
        }
    }

    fn sucessor(&self,number: Int) -> Option<Int>{
        if self.element_list.len() == 0 {
            None
        } else {
            self.succ(number, 0, self.element_list.len()-1)
        }
    }
    
    fn minimum(&self) -> Option<Int>{
        if self.element_list.len() == 0 {
            None
        } else {
            Some(self.element_list[0])
        }
    }

    fn maximum(&self) -> Option<Int>{
        if self.element_list.len() == 0 {
            None
        } else {
            Some(self.element_list[self.element_list.len()-1])
        }
    }

    fn contains(&self, number: Int) -> bool {
        self.element_list.contains(&number)
    }

    const TYPE: &'static str = "BinarySearch";
}

impl BinarySearch {
    fn succ(&self, element: Int, l: usize, r: usize) -> Option<Int> {
        let mut l = l;
        let mut r = r;

        while r != l {
            let m = (l+r)/2;
            if self.element_list[m] > element {
                r = m;
            } else {
                l = m+1;
            }
        }
        if self.element_list[l] >= element {
            Some(self.element_list[l])
        } else {
            None
        }
    }

    fn pred(&self, element: Int, l: usize, r: usize) -> Option<Int> {
        let mut l = l;
        let mut r = r;

        while l != r {
            let m = (l+r)/2;
            if self.element_list[m] < element {
                r = m
            } else {
                l = m+1;
            }
        }
 
        if element >= self.element_list[l] {
            Some(self.element_list[l])
        } else {
            None
        }
    }


}

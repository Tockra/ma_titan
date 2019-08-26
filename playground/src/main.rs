use std::time::{Instant};
use predecessor_list::data_structures::statics::STree;
use predecessor_list::help::internal::PredecessorSetStatic;
use uint::u40;
extern crate serde;
extern crate rmp_serde as rmps;

use serde::{Deserialize};
use rmps::{Deserializer};

use rand_pcg::Mcg128Xsl64;
use rand::Rng;

use std::fs::read_dir;
use std::io::BufReader;
use std::fs::File;
use std::rc::Rc;
use std::ops::Add;
use std::fmt::Debug;
use self::bench_data::BinarySearch;

const SEED: u128 = 0xcafef00dd15ea5e5;

fn main() {
    for dir in read_dir(format!("../testdata/u40/")).unwrap() {
        let mut state = Mcg128Xsl64::new(SEED);
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| u40::from(v)).collect::<Vec<u40>>();

        // let test_values: Vec<u64> = ((values[0]+1u32).into()..(values[0]+100u32).into()).choose_multiple(&mut state, 10);
        let mut test_values: Vec<u40> = Vec::with_capacity(1000);

        while test_values.len() != 1000 {
            test_values.push(u40::from(state.gen_range(u64::from(values[0]+1u64),u64::from(values[values.len()-1]))));
        }
        let data_structure: Rc<STree> = Rc::new(STree::new(values));
        let data_strucuture_succ:Rc<STree> = Rc::clone(&data_structure);

        let id = &format!("{}::predecessor",STree::TYPE)[..];
        let cp = test_values.clone();
        let mut sum = 0;
        let now = Instant::now();
        for i in 0..10 {
            let cp = cp.clone();
            let now = Instant::now();
            for elem in cp {
                data_structure.predecessor(elem);
            }
            sum += now.elapsed().as_micros();
        }

        println!("Messung: {} us",sum as f64 / 10.);

    }

   
}

fn test(irgendwa: Vec<u64>) {

}


mod bench_data {
    use uint::u40;
    use predecessor_list::help::internal::{PredecessorSetStatic};

    type Int = u40;
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
                // Todo 1000 Werte iterativ
                if r - l +1 <= 100 {
                    for i in l..(r+1) {
                        if self.element_list[i] >= element  {
                            return Some(self.element_list[i])
                        }
                    }
                } else {
                    let m = (l+r)/2;
                    if self.element_list[m] > element {
                        r = m;
                    } else {
                        l = m+1;
                    }
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
                if r - l <= 1000 {
                    for i in (l..(r+1)).rev() {
                        if self.element_list[i] <= element  {
                            return Some(self.element_list[i])
                        }
                    }
                } else {
                    let m = (l+r)/2;
                    if self.element_list[m] < element {
                        r = m
                    } else {
                        l = m+1;
                    }
                }
            }
    
            if element >= self.element_list[l] {
                Some(self.element_list[l])
            } else {
                None
            }
        }


    }

}
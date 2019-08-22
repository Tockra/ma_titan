#[macro_use]
extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use criterion::Criterion;
use criterion::Bencher;
use criterion::BatchSize;

use serde::{Deserialize};
use rmps::{Deserializer};

use rand_pcg::Mcg128Xsl64;
use rand::seq::IteratorRandom;

use std::fs::read_dir;
use std::io::BufReader;
use std::fs::File;
use std::rc::Rc;
use std::ops::Add;
use std::fmt::Debug;

use predecessor_list::help::internal::PredecessorSetStatic;
use predecessor_list::data_structures::statics::STree;
use self::bench_data::BinarySearch;

use uint::u40;
use uint::Typable;
// TODO: Laufzeit von der Summe aller Succ-Instruktionen messen
// Generierung anpassen in den Benchmarks
const SEED: u128 = 0xcafef00dd15ea5e5;

/// Diese Methode lädt die Testdaten aus ../testdata/{u40,u48,u64}/ und konstruiert mit Hilfe dieser eine
/// Datenstruktur T. Dabei wird die Laufzeit gemessen.
fn static_build_benchmark<E: Typable, T: PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        
       
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| u40::from(v)).collect::<Vec<u40>>();
        c.bench_function(&format!("{}::new <{}>",T::TYPE,values.len())[..], move 
                                    |b| b.iter_batched(|| values.clone(), |data| STree::new(data), BatchSize::SmallInput));
    }
}

/// Lädt die Testdaten aus ../testdata/{u40,u48,u64}/ und erzeugt mit Hilfe dieser die zu testende Datenstruktur T. 
/// Anschließend werden 1000 gültige Vor- bzw. Nachfolger erzeugt und die Laufzeiten der Predecessor- und Sucessor-Methode 
/// werden mit Hilfe dieser gemessen
fn pred_and_succ_benchmark<E: 'static + Typable + Copy + Debug + From<u64> + Into<u64> + Add<u32, Output=E>, T: 'static + PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let mut state = Mcg128Xsl64::new(SEED);
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| E::from(v)).collect::<Vec<E>>();

        // let test_values: Vec<u64> = ((values[0]+1u32).into()..(values[0]+100u32).into()).choose_multiple(&mut state, 10);
        let test_values: Vec<u64> = ((values[0]+1u32).into()..(values[values.len()-1]).into()).choose_multiple(&mut state, 1000);
        let test_values = test_values.into_iter().map(|v| E::from(v)).collect::<Vec<E>>();

        let data_structure: Rc<T> = Rc::new(T::new(values));
        let data_strucuture_succ:Rc<T> = Rc::clone(&data_structure);

        c.bench_function_over_inputs(&format!("{}::predecessor",T::TYPE)[..],move
            |b: &mut Bencher, elem: &E| {
                b.iter(|| data_structure.predecessor(*elem));
            },
            test_values.clone()
        );

        c.bench_function_over_inputs(&format!("{}::sucessor",T::TYPE)[..],move
            |b: &mut Bencher, elem: &E| {
                b.iter(|| data_strucuture_succ.sucessor(*elem));
            },
            test_values
        );
    }
}

criterion_group!(stree_gen, static_build_benchmark<u40,STree>);
criterion_group!(binary_search_gen, static_build_benchmark<u40,BinarySearch>);
criterion_group!(stree_instr, pred_and_succ_benchmark<u40,STree>);
criterion_group!(binary_search_instr, pred_and_succ_benchmark<u40,BinarySearch>);

criterion_main!(stree_gen, binary_search_gen, stree_instr, binary_search_instr);


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
                if r - l +1 <= 1000 {
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
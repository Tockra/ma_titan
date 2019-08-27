#[macro_use]
extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use criterion::Criterion;

use criterion::Bencher;
use criterion::BatchSize;
use criterion::ParameterizedBenchmark;
use criterion::Benchmark;

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
use std::io::BufWriter;
use std::fs::File;
use std::io::prelude::*;

use predecessor_list::help::internal::PredecessorSetStatic;
use predecessor_list::data_structures::statics::STree;
use self::bench_data::BinarySearch;
use std::ptr::write_volatile;
use uint::u40;
use uint::Typable;
// TODO: Cache clears vor den Succ und Pred Instruktionen

const SEED: u128 = 0xcafef00dd15ea5e5;
const SAMPLE_SIZE: usize = 10;
/// Diese Methode lädt die Testdaten aus ../testdata/{u40,u48,u64}/ und konstruiert mit Hilfe dieser eine
/// Datenstruktur T. Dabei wird die Laufzeit gemessen.
fn static_build_benchmark<E: 'static + Typable + Copy + Debug + From<u64>, T: PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("../testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        
       
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| E::from(v)).collect::<Vec<E>>();
        let id = &format!("{}::new <{}>",T::TYPE,values.len())[..];
        c.bench(id ,Benchmark::new(id, move 
                                    |b| b.iter_batched(|| values.clone(), |data| T::new(data), BatchSize::SmallInput)).sample_size(SAMPLE_SIZE));
    }
}

/// Lädt die Testdaten aus ../testdata/{u40,u48,u64}/ und erzeugt mit Hilfe dieser die zu testende Datenstruktur T. 
/// Anschließend werden 1000 gültige Vor- bzw. Nachfolger erzeugt und die Laufzeiten der Predecessor- und Sucessor-Methode 
/// werden mit Hilfe dieser gemessen
fn pred_and_succ_benchmark<E: 'static + Typable + Copy + Debug + From<u64> + Into<u64> + Add<u32, Output=E>, T: 'static + PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("../testdata/{}/", E::TYPE)).unwrap() {
        let mut state = Mcg128Xsl64::new(SEED);
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| E::from(v)).collect::<Vec<E>>();

        // let test_values: Vec<u64> = ((values[0]+1u32).into()..(values[0]+100u32).into()).choose_multiple(&mut state, 10);
        let mut test_values: Vec<E> = Vec::with_capacity(1000);

        while test_values.len() != 1000 {
            test_values.push(E::from(state.gen_range((values[0]+1u32).into(),(values[values.len()-1]).into())));
        }
        let data_structure: Rc<T> = Rc::new(T::new(values));
        let data_strucuture_succ:Rc<T> = Rc::clone(&data_structure);

        let id = &format!("{}::predecessor",T::TYPE)[..];
        let cp = test_values.clone();
        c.bench(id,ParameterizedBenchmark::new(id,move
            |b: &mut Bencher, elems: &Vec<E>| {
                b.iter_batched(|| {
                    let result = Rc::clone(&data_structure);
                    cache_clear();
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    result
                } , |data| {
                    // Laufzeit der For-Schleife wird mitgemessen.
                    for elem in elems {
                        data.predecessor(*elem);
                    }
                }, BatchSize::SmallInput);
            },
            vec![cp]
        ).sample_size(SAMPLE_SIZE));

        let id = &format!("{}::sucessor",T::TYPE)[..];
        c.bench(id,ParameterizedBenchmark::new(id,move
            |b: &mut Bencher, elems: &Vec<E>| {
                b.iter(|| {
                    for elem in elems {
                        data_strucuture_succ.sucessor(*elem);
                    }
                });
            },
            vec![test_values]
        ).sample_size(SAMPLE_SIZE));
    }
}

// Diese Methode löscht (hoffentlich) 12 Mbyte des Caches. 
pub fn cache_clear() {
    let mut data = vec![23u64];

    for i in 1 .. 3_750_000u64 {
        data.push(data[i as usize -1] + i);
    }

    let mut buf = BufWriter::new(File::create("cache").unwrap());
    buf.write_fmt(format_args!("{}", data[data.len()-1])).unwrap();
}


criterion_group!(stree_gen, static_build_benchmark<u40,STree>);
criterion_group!(binary_search_gen, static_build_benchmark<u40,BinarySearch>);
criterion_group!(stree_instr, pred_and_succ_benchmark<u40,STree>);
criterion_group!(binary_search_instr, pred_and_succ_benchmark<u40,BinarySearch>);

fn test(c: &mut Criterion) {
    c.bench_function("with_setup", move |b| {
        // This will avoid timing the to_vec call.
        b.iter_batched(|| {std::thread::sleep(std::time::Duration::from_millis(500));}, |_| {
            let mut v = vec![];
            for i in 0..1000000 {
                v.push(i);
            }
        }, BatchSize::SmallInput)
    });
}
criterion_group!(test1, test);
criterion_main!(test1);


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

}
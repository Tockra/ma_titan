#[macro_use]

extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use criterion::Criterion;

use criterion::Bencher;
use criterion::BatchSize;
use criterion::ParameterizedBenchmark;
use criterion::Benchmark;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use rmps::Deserializer;

use rand_pcg::Mcg128Xsl64;
use rand::Rng;

use std::fs::read_dir;
use std::io::BufReader;
use std::ops::Add;
use std::fmt::Debug;
use std::io::BufWriter;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;

use ma_titan::internal::PredecessorSetStatic;
use ma_titan::default::immutable::STree;
use self::bench_data::BinarySearch;
use uint::u40;
use uint::Typable;

const SEED: u128 = 0xcafef00dd15ea5e5;
const SAMPLE_SIZE: usize = 10;
/// Diese Methode lädt die Testdaten aus ./testdata/{u40,u48,u64}/ und konstruiert mit Hilfe dieser eine
/// Datenstruktur T. Dabei wird die Laufzeit gemessen.
fn static_build_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned, T: PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        
       
        let mut values = Deserializer::new(buf);
        let values: Vec<E> = Deserialize::deserialize(&mut values).unwrap();

        let id = &format!("algo={} method=new size={}",T::TYPE,values.len())[..];
        c.bench(id ,Benchmark::new(id, move 
                                    |b| b.iter_batched(|| values.clone(), |data| T::new(data), BatchSize::SmallInput)).sample_size(SAMPLE_SIZE).warm_up_time(Duration::new(0, 1)));
    }
}

/// Lädt die Testdaten aus ./testdata/{u40,u48,u64}/ und erzeugt mit Hilfe dieser die zu testende Datenstruktur T. 
/// Anschließend werden 10000 gültige Vor- bzw. Nachfolger erzeugt und die Laufzeiten der Predecessor- und Sucessor-Methode 
/// werden mit Hilfe dieser gemessen
fn pred_and_succ_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned + From<u64> + Into<u64> + Add<u32, Output=E>, T: 'static + Clone + PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let mut state = Mcg128Xsl64::new(SEED);
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<E> = Deserialize::deserialize(&mut values).unwrap();

        let len = values.len();

        let mut test_values: Vec<E> = Vec::with_capacity(10000);

        while test_values.len() != 10000 {
            test_values.push(E::from(state.gen_range((values[0]+1u32).into(),(values[values.len()-1]).into())));
        }
        let data_structure = T::new(values);
        let data_strucuture_succ:T = data_structure.clone();

        let id = &format!("{}::predecessor <{}>",T::TYPE, len)[..];
        let cp = test_values.clone();
        c.bench(id,ParameterizedBenchmark::new(id,move
            |b: &mut Bencher, elems: &Vec<E>| {
                b.iter(|| {
                    for elem in elems {
                        data_structure.predecessor(*elem);
                    }
                });
            },
            vec![cp]
        ).sample_size(SAMPLE_SIZE).warm_up_time(Duration::new(0, 1)));

        let id = &format!("{}::sucessor <{}>",T::TYPE, len)[..];
        c.bench(id,ParameterizedBenchmark::new(id,move
            |b: &mut Bencher, elems: &Vec<E>| {
                b.iter(|| {
                    for elem in elems {
                        data_strucuture_succ.sucessor(*elem);
                    }
                });
            },
            vec![test_values]
        ).sample_size(SAMPLE_SIZE).warm_up_time(Duration::new(0, 1)));
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


criterion_group!(stree_gen_u40, static_build_benchmark<u40,STree<u40>>);
criterion_group!(binary_search_gen_u40, static_build_benchmark<u40,BinarySearch>);
criterion_group!(stree_instr_u40, pred_and_succ_benchmark<u40,STree<u40>>);
criterion_group!(binary_search_instr_u40, pred_and_succ_benchmark<u40,BinarySearch>);
criterion_main!(binary_search_gen_u40);

/// Diese Methode darf erst am Ende einer Bench-Methode aufgerufen werden, da ansonsten /target/criterion/ nicht existiert
/// Außerdem muss sichergestellt werden, dass man sich zum Zeitpunkt des Aufrufs im Hauptverzeichnis des Rust-Projects befindet.
fn generate_sql_plot_input() {
    for entry in read_dir("./target/criterion/").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            // Todo checken op der Path path//new/  aussieht.
            let raw = File::open(format!("{}/new/", path.to_str().unwrap())).unwrap();
            // nun ist kann mit Hilfe der raw.csv (raw) das Output-File erzeugt werden
        } 
    }
}


mod bench_data {
    use uint::u40;
    use ma_titan::internal::PredecessorSetStatic;

    // Todo Generics
    type Int = u40;

    #[derive(Clone)]
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

}
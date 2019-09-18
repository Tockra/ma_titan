#[macro_use]

extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use criterion::Criterion;
use criterion::black_box;

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
use std::io::{BufWriter, BufReader};
use std::ops::Add;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use std::fs::OpenOptions;

use ma_titan::internal::PredecessorSetStatic;
use ma_titan::default::immutable::{STree, BinarySearch};
use change_ds::VEBTree;
use uint::u40;
use uint::Typable;
const REPEATS: usize = 10_000;
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
/// Anschließend werden 10000 gültige Vor- bzw. Nachfolger erzeugt und die Laufzeiten der Predecessor-Methode 
/// werden mit Hilfe dieser gemessen
fn pred_and_succ_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned + From<u64> + Into<u64> + Add<u32, Output=E>, T: 'static + Clone + PredecessorSetStatic<E>>(c: &mut Criterion) {
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<E> = Deserialize::deserialize(&mut values).unwrap();
        let values_len = values.len();

        let test_values = get_test_values(values[0]+1u32,values[values_len-1]);

        let data_structure = T::new(values.clone());
        let data_structure_succ = T::new(values);
        

        let id = &format!("algo={}<{}> method=predecessor size={}",T::TYPE,E::TYPE, values_len)[..];
        let cp = test_values.clone();
        c.bench(id,ParameterizedBenchmark::new(id,move
            |b: &mut Bencher, elems: &Vec<E>| {
                b.iter_batched(|| {
                    cache_clear();
                    ()
                }, |_| {
                    for elem in elems {
                        data_structure.predecessor(*elem);
                    }
                }, BatchSize::SmallInput);
            },
            vec![cp]
        ).sample_size(SAMPLE_SIZE));

        let id = &format!("algo={}<{}> method=successor size={}",T::TYPE,E::TYPE, values_len)[..];
        c.bench(id,ParameterizedBenchmark::new(id,move
            |b: &mut Bencher, elems: &Vec<E>| {
                b.iter_batched(|| {
                    cache_clear();
                    ()
                }, |_| {
                    for elem in elems {
                        data_structure_succ.successor(*elem);
                    }
                }, BatchSize::SmallInput);
            },
            vec![test_values]
        ).sample_size(SAMPLE_SIZE));
    }
}

fn get_test_values<E: 'static + Typable + Copy + From<u64> + Into<u64> + Add<u32, Output=E>>(min: E, max: E) -> Vec<E> {
    let mut state = Mcg128Xsl64::new(black_box(SEED));
    let mut test_values: Vec<E> = Vec::with_capacity(REPEATS);

    while test_values.len() != REPEATS {
        test_values.push(E::from(state.gen_range(min.into(),max.into())));
    }
    test_values
}

// Diese Methode löscht (hoffentlich) 12 Mbyte des Caches. 
pub fn cache_clear() {
    let mut data = vec![23u64];

    for i in 1 .. 3_750_000u64 {
        data.push(black_box(data[i as usize -1] + i));
    }

    let mut buf = BufWriter::new(File::create("cache").unwrap());
    buf.write_fmt(format_args!("{}", data[data.len()-1])).unwrap();
}


criterion_group!(stree_gen_u40, static_build_benchmark<u40,STree<u40>>);
criterion_group!(binary_search_gen_u40, static_build_benchmark<u40,BinarySearch<u40>>);
criterion_group!(veb_tree_gen_u40, static_build_benchmark<u40,VEBTree>);
criterion_group!(stree_pred_u40, pred_and_succ_benchmark<u40,STree<u40>>);
criterion_group!(binary_search_pred_u40, pred_and_succ_benchmark<u40,BinarySearch<u40>>);
criterion_group!(veb_tree_pred_u40, pred_and_succ_benchmark<u40,VEBTree<u40>>);


criterion_main!(stree_gen_u40, binary_search_gen_u40, veb_tree_gen_u40, stree_pred_u40, binary_search_pred_u40, veb_tree_pred_u40, generate_sql_plot_input);

/// Diese Methode darf erst am Ende einer Bench-Methode aufgerufen werden, da ansonsten /target/criterion/ nicht existiert
/// Außerdem muss sichergestellt werden, dass man sich zum Zeitpunkt des Aufrufs im Hauptverzeichnis des Rust-Projects befindet.
fn generate_sql_plot_input() {
    let mut result = BufWriter::new(OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open("stats.txt").unwrap());
    for entry in read_dir("./target/criterion/").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            // Todo checken op der Path path//new/  aussieht.
            let raw = BufReader::new(File::open(path.join("new").join("raw.csv")).unwrap());
            // nun ist kann mit Hilfe der raw.csv (raw) das Output-File erzeugt werden
            
            for line in raw.lines().skip(1) {
                let line = line.unwrap();
                let line: Vec<&str> = line.split(",").collect();

                writeln!(result, "RESULT {} time={} unit={} repeats={}",line[0],line[5].parse::<f64>().unwrap()/line[7].parse::<f64>().unwrap(),line[6], REPEATS).unwrap(); 
            } 
            result.flush().unwrap();

        } 
    }
}

mod change_ds {
    use vebtrees::VEBTree as vs;
    use ma_titan::internal::PredecessorSetStatic;
    use ma_titan::default::immutable::{Int};
    struct VEBTree {
        veb_tree: vs
    }

    impl<T: Int> PredecessorSetStatic<T> for VEBTree<T> {
        const TYPE: &'static str = "vEB-Tree";

        fn new(elements: Vec<T>) -> Self {
            let vtree = vs::new(elements.len());
            for elem in elements {
                vtree.insert((elem.into()) as usize);
            }
            Self {
                veb_tree: vtree,
            }
        }

        fn predecessor(&self,number: T) -> Option<T> {
            self.veb_tree.findprev((number.into()) as usize).and_then(|x| Some(T::new(x as u64)))
        }

        fn successor(&self,number: T) -> Option<T> {
            self.veb_tree.findnext((number.into()) as usize).and_then(|x| Some(T::new(x as u64)))
        }

        fn minimum(&self) -> Option<T> {
            self.veb_tree.minimum().and_then(|x| Some(T::new(x as u64)))
        }

        fn maximum(&self) -> Option<T> {
            self.veb_tree.maximum().and_then(|x| Some(T::new(x as u64)))
        } 

        fn contains(&self, number: T) -> bool {
            self.veb_tree.contains((number.into()) as usize)
        }
    }

}
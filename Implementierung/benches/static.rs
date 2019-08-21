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

use predecessor_list::help::internal::PredecessorSetStatic;
use predecessor_list::data_structures::statics::STree;
use predecessor_list::data_structures::binary::BinarySearch;

use uint::u40;

const SEED: u128 = 0xcafef00dd15ea5e5;

fn static_build_benchmark<T: PredecessorSetStatic<u40>>(c: &mut Criterion) {
    for dir in read_dir("testdata/u40/").unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();

        let buf = BufReader::new(File::create(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| u40::from(v)).collect::<Vec<u40>>();
        c.bench_function(&format!("{}::new <{}>",T::TYPE,values.len())[..], move 
                                    |b| b.iter_batched(|| values.clone(), |data| STree::new(data), BatchSize::SmallInput));
    }
}

fn pred_and_succ_benchmark<T: 'static + PredecessorSetStatic<u40>>(c: &mut Criterion) {
    for dir in read_dir("testdata/u40/").unwrap() {
        let mut state = Mcg128Xsl64::new(SEED);
        let dir = dir.unwrap();
        let path = dir.path();

        let buf = BufReader::new(File::create(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| u40::from(v)).collect::<Vec<u40>>();

        
        let test_values: Vec<u64> = (u64::from(values[0]+1u32)..u64::from(values[values.len()-1])).choose_multiple(&mut state, 1000);
        let test_values = test_values.into_iter().map(|v| u40::from(v)).collect::<Vec<u40>>();

        let data_structure: Rc<T> = Rc::new(T::new(values));
        let data_strucuture_succ:Rc<T> = Rc::clone(&data_structure);

        c.bench_function_over_inputs(&format!("{}::predecessor",T::TYPE)[..],move
            |b: &mut Bencher, elem: &u40| {
                b.iter(|| data_structure.predecessor(*elem));
            },
            test_values.clone()
        );

        c.bench_function_over_inputs(&format!("{}::sucessor",T::TYPE)[..],move
            |b: &mut Bencher, elem: &u40| {
                b.iter(|| data_strucuture_succ.sucessor(*elem));
            },
            test_values
        );
    }
}

criterion_group!(stree_gen, static_build_benchmark<STree>);
criterion_group!(binary_search_gen, static_build_benchmark<BinarySearch>);
criterion_group!(stree_instr, pred_and_succ_benchmark<STree>);
criterion_group!(binary_search_instr, pred_and_succ_benchmark<BinarySearch>);


criterion_main!(stree_gen, binary_search_gen, stree_instr, binary_search_instr);

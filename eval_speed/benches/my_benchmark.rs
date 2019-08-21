#[macro_use]
extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use criterion::Criterion;
use criterion::BatchSize;

use serde::{Deserialize};
use rmps::{Deserializer};

use std::fs::read_dir;
use std::io::BufReader;
use std::fs::File;

use predecessor_list::help::internal::PredecessorSetStatic;
use predecessor_list::data_structures::statics::STree;
use uint::u40;


fn stree_static_build_benchmark<T: PredecessorSetStatic<u40>>(c: &mut Criterion) {
    for dir in read_dir("testdata/u40/").unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",dir.path());
        let buf = BufReader::new(File::create(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| u40::from(v)).collect::<Vec<u40>>();
        c.bench_function(&format!("Static Generate::new <{}>",values.len())[..], move 
                                    |b| b.iter_batched(|| values.clone(), |data| STree::new(data), BatchSize::SmallInput));
    }

}

criterion_group!(benches, stree_static_build_benchmark<STree>);
criterion_main!(benches);

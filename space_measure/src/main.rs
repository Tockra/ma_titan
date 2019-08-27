extern crate stats_alloc;
extern crate serde;
extern crate rmp_serde as rmps;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

use std::alloc::System;
use std::fs::read_dir;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;

use stree::u40::stat::STree;
use stree::internal::PredecessorSetStatic;

use uint::u40;
use uint::Typable;

use serde::Deserialize;
use rmps::Deserializer;
#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() {
    
    measure::<u40,STree>();
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    //::std::mem::size_of_val(&x);
}

fn measure<E: 'static + Typable + Copy + Debug + From<u64>, T: PredecessorSetStatic<E>>() {
    for dir in read_dir(format!("../testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        let p = format!("{:?}", path);

        let buf = BufReader::new(File::open(path).unwrap());
        
        
        let mut values = Deserializer::new(buf);
        let values: Vec<u64> = Deserialize::deserialize(&mut values).unwrap();
        let values = values.into_iter().map(|v| E::from(v)).collect::<Vec<E>>();

        let reg = Region::new(&GLOBAL);
        let x = T::new(values);
        println!("{}: {:#?}", p,reg.change());
    }
}

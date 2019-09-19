extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use std::fs::OpenOptions;
use std::time::{Instant};
use std::fmt::Debug;
use std::ops::Add;
use std::io::{BufWriter, BufReader};
use std::fs::read_dir;
use rand_pcg::Mcg128Xsl64;
use rand::Rng;

use uint::Typable;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use rmps::Deserializer;

use crate::internal::PredecessorSetStatic;

use criterion::black_box;

const SAMPLE_SIZE: usize = 10;
const REPEATS: usize = 10_000;
const SEED: u128 = 0xcafef00dd15ea5e5;
/// Diese Methode lädt die Testdaten aus ./testdata/{u40,u48,u64}/ und konstruiert mit Hilfe dieser eine
/// Datenstruktur T. Dabei wird die Laufzeit gemessen.
fn static_build_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned, T: PredecessorSetStatic<E>>() {
    let mut result = BufWriter::new(OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("stats_new_{}.txt",T::TYPE)).unwrap());
    
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        
       
        let mut values = Deserializer::new(buf);
        let values: Vec<E> = Deserialize::deserialize(&mut values).unwrap();

        for _ in 0..SAMPLE_SIZE {
            let values_cloned = values.clone();
            let len = values_cloned.len();
            let now = Instant::now();
            T::new(values_cloned);
            let elapsed_time = now.elapsed().as_nanos();
            writeln!(result, "RESULT algo={} method=new size={} time={} unit=ns repeats={}",T::TYPE, len, elapsed_time, SAMPLE_SIZE).unwrap(); 
        }
        result.flush().unwrap();
        
    }
}

/// Lädt die Testdaten aus ./testdata/{u40,u48,u64}/ und erzeugt mit Hilfe dieser die zu testende Datenstruktur T. 
/// Anschließend werden 10000 gültige Vor- bzw. Nachfolger erzeugt und die Laufzeiten der Predecessor-Methode 
/// werden mit Hilfe dieser gemessen
fn pred_and_succ_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned + From<u64> + Into<u64> + Add<u32, Output=E>, T: 'static + Clone + PredecessorSetStatic<E>>() {
    let mut result = BufWriter::new(OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("stats_pred_succ_{}.txt",T::TYPE)).unwrap());
    for dir in read_dir(format!("testdata/{}/", E::TYPE)).unwrap() {
        let dir = dir.unwrap();
        let path = dir.path();
        println!("{:?}",path);

        let buf = BufReader::new(File::open(path).unwrap());
        let mut values = Deserializer::new(buf);
        let values: Vec<E> = Deserialize::deserialize(&mut values).unwrap();
        let values_len = values.len();

        let test_values = get_test_values(values[0]+1u32,values[values_len-1]);

        let len = values.len();
        let data_structure = T::new(values.clone());
        let data_structure_succ = T::new(values);
        

        for _ in 0..SAMPLE_SIZE {
            cache_clear();
            let now = Instant::now();
            for elem in test_values.iter() {
                data_structure.predecessor(*elem);
            }
            let elapsed_time = now.elapsed().as_nanos();
            writeln!(result, "RESULT algo={} method=predecessor size={} time={} unit=ns repeats={} data={}",T::TYPE, len, elapsed_time, SAMPLE_SIZE,values_len).unwrap();
        }

        for _ in 0..SAMPLE_SIZE {
            cache_clear();
            let now = Instant::now();
            for elem in test_values.iter() {
                data_structure_succ.successor(*elem);
            }
            let elapsed_time = now.elapsed().as_nanos();
            writeln!(result, "RESULT algo={} method=successor size={} time={} unit=ns repeats={} data={}",T::TYPE, len, elapsed_time, SAMPLE_SIZE,values_len).unwrap();
        }
        result.flush().unwrap();
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
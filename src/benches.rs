extern crate criterion;
extern crate serde;
extern crate rmp_serde as rmps;

use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::time::{Instant, SystemTime};
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

const SAMPLE_SIZE: usize = 100;
const REPEATS: usize = 100_000;
const SEED: u128 = 0xcafef00dd15ea5e5;
/// Diese Methode lädt die Testdaten aus ./testdata/{u40,u48,u64}/ und konstruiert mit Hilfe dieser eine
/// Datenstruktur T. Dabei wird die Laufzeit gemessen.
pub fn static_build_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned, T: PredecessorSetStatic<E>>() {
    println!("Starte Evaluierung der Datenstrukturerzeugung");
    let bench_start = Instant::now();
    std::fs::create_dir_all("./output/new/").unwrap();

    let mut result = BufWriter::new(OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("output/new/{}.txt",T::TYPE)).unwrap());
    
    for dir in read_dir(format!("testdata/uniform/{}/", E::TYPE)).unwrap() {
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
            let result_ds = T::new(values_cloned);
            let elapsed_time = now.elapsed().as_nanos();
            println!("{:?}",result_ds.maximum());
            writeln!(result, "RESULT algo={} method=new size={} time={} unit=ns repeats={}",T::TYPE, len, elapsed_time, SAMPLE_SIZE).unwrap(); 
        }
        result.flush().unwrap();
        
    }
    println!("Laufzeitmessung der Datenstrukturerzeugung beendet. Dauer {} Sekunden", bench_start.elapsed().as_secs())
}

/// Lädt die Testdaten aus ./testdata/{u40,u48,u64}/ und erzeugt mit Hilfe dieser die zu testende Datenstruktur T. 
/// Anschließend werden 10000 gültige Vor- bzw. Nachfolger erzeugt und die Laufzeiten der Predecessor-Methode 
/// werden mit Hilfe dieser gemessen
pub fn pred_and_succ_benchmark<E: 'static + Typable + Copy + Debug + DeserializeOwned + From<u64> + Into<u64> + Add<u32, Output=E>, T: 'static + Clone + PredecessorSetStatic<E>>() {
    println!("Starte Evaluierung der Predecessor- und Successor Methoden.");
    let bench_start = Instant::now();
    std::fs::create_dir_all("./output/pred/{}.txt").unwrap();
    let mut result = BufWriter::new(OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("output/pred/{}.txt",T::TYPE)).unwrap());
    for dir in read_dir(format!("testdata/uniform/{}/", E::TYPE)).unwrap() {
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
    println!("Laufzeitmessung der Predecessor- und Successor-Methoden beendet. Dauer {} Sekunden", bench_start.elapsed().as_secs())
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
        let mut sum = 0;
        for j in 0..(i as usize) {
            sum += data[j];
        }
        data.push(black_box(sum));
    }

    let mut buf = BufWriter::new(File::create(format!("cache_{}",SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis())).unwrap());
    buf.write_fmt(format_args!("{}", data[data.len()-1])).unwrap();

    buf.flush().unwrap();
}


use vebtrees::VEBTree as vs;
use crate::default::immutable::Int;

#[derive(Clone,Debug, PartialEq, Eq)]
pub struct VEBTree {
    veb_tree: vs<usize>
}

impl<T: Int> PredecessorSetStatic<T> for VEBTree {
    const TYPE: &'static str = "vEB-Tree";

    fn new(elements: Vec<T>) -> Self {
        let mut vtree = vs::with_capacity(elements.len());
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

#[derive(Clone)]
pub struct BinarySearch<T> {
    element_list: Box<[T]>
}

impl<T: Int>  PredecessorSetStatic<T> for BinarySearch<T> {
    fn new(elements: Vec<T>) -> Self {
        Self {
            element_list: elements.into_boxed_slice(),
        }
    }

    fn predecessor(&self,number: T) -> Option<T> {
        if self.element_list.len() == 0 {
            None
        } else {
            self.pred(number, 0, self.element_list.len()-1)
        }
    }

    fn successor(&self,number: T) -> Option<T>{
        if self.element_list.len() == 0 {
            None
        } else {
            self.succ(number, 0, self.element_list.len()-1)
        }
    }
    
    fn minimum(&self) -> Option<T>{
        if self.element_list.len() == 0 {
            None
        } else {
            Some(self.element_list[0])
        }
    }

    fn maximum(&self) -> Option<T>{
        if self.element_list.len() == 0 {
            None
        } else {
            Some(self.element_list[self.element_list.len()-1])
        }
    }

    fn contains(&self, number: T) -> bool {
        self.element_list.contains(&number)
    }

    const TYPE: &'static str = "BinarySearch";
}

impl<T: Int> BinarySearch<T> {
    fn succ(&self, element: T, l: usize, r: usize) -> Option<T> {
        let mut l = l;
        let mut r = r;

        if element >= self.element_list[r] {
            return None;
        }

        while r != l && element >= self.element_list[l]  {
            let m = (l+r)/2;
            if element >= self.element_list[m] {
                l = m+1;
            } else {
                r = m;
            }
        }
        if element < self.element_list[l] {
            Some(self.element_list[l])
        } else {
            None
        }
    }

    fn pred(&self, element: T, l: usize, r: usize) -> Option<T> {
        let mut l = l;
        let mut r = r;

        if element <= self.element_list[l] {
            return None;
        }

        while l != r && element <= self.element_list[r] {
            let m = (l+r)/2;
            if self.element_list[m] >= element {
                r = m-1;
            } else {
                l = m;
            }
        }

        if element > self.element_list[r] {
            Some(self.element_list[r])
        } else {
            None
        }
    }


}

use std::collections::BTreeMap;

impl<T: Int>  PredecessorSetStatic<T> for BTreeMap<T,T> {
    fn new(elements: Vec<T>) -> Self {
        let mut n: BTreeMap<T,T> = BTreeMap::new();
        for i in elements {
            n.insert(i,i);
        }
        n
    }

    fn predecessor(&self,number: T) -> Option<T> {
        self.range(T::from(0)..number).last().map(|x| *x.0)
    }

    fn successor(&self,number: T) -> Option<T>{
        self.range(number..).next().map(|x| *x.0)
    }
    
    fn minimum(&self) -> Option<T>{
        self.range(T::from(0)..).next().map(|x| *x.0)
    }

    fn maximum(&self) -> Option<T>{
        self.range(T::from(0)..).rev().next().map(|x| *x.0)
    }

    fn contains(&self, number: T) -> bool {
        self.contains_key(&number)
    }

    const TYPE: &'static str = "B-Baum";
}


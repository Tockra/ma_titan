use std::collections::HashMap;

use ux::{u10,u40};
use boomphf::Mphf;

use crate::help::internal::{Splittable};
use crate::data_structures::statics::{FirstLevel, SecondLevel};

type SecondLevelBuild = PerfectHashBuilderLevel<usize>;
type FirstLevelBuild = PerfectHashBuilderLevel<SecondLevelBuild>;
type Int = u40;
const U40_HALF_SIZE: usize = 1<<20;
pub struct PerfectHashBuilder {
    root_table: Box<[FirstLevelBuild]>,
    root_indexs: Vec<usize>,
}

pub struct PerfectHashBuilderLevel<T> {
    pub hash_map: std::collections::HashMap<u10,T>,
    pub objects: Vec<u10>,
    pub maximum: usize,
    pub minimum: usize,
    pub lx_top: Vec<u64>,
}
impl<T> PerfectHashBuilderLevel<T> {
    #[inline]
    pub fn new(level: usize) -> PerfectHashBuilderLevel<T> {
        PerfectHashBuilderLevel {
            hash_map: (HashMap::<u10,T>::default()),
            objects: vec![],
            maximum: 0,
            minimum: 0,
            lx_top: vec![0;level],
        }
    }
}
impl PerfectHashBuilder {
    pub fn new(objects: Vec<Int>) ->  PerfectHashBuilder{
        let mut root_indexs = vec![];

        let mut tmp: Vec<FirstLevelBuild> = Vec::with_capacity(U40_HALF_SIZE);
        for _ in 0..tmp.capacity() {
            tmp.push(FirstLevelBuild::new((1<<10)/64));
        }
        let mut root_table: Box<[FirstLevelBuild]> = tmp.into_boxed_slice();
    
        for element in objects {
            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&element);

            if !root_indexs.contains(&i) {
                root_indexs.push(i);
            }
            
            if !root_table[i].hash_map.contains_key(&j) {
                root_table[i].objects.push(j);
                root_table[i].hash_map.insert(j,SecondLevelBuild::new((1<<10)/64));
            }
            
            // Hier ist keine Prüfung notwendig, da die Elemente einmalig sind.
            root_table[i].hash_map.get_mut(&j).unwrap().objects.push(k);
        }
        PerfectHashBuilder {root_table: root_table, root_indexs: root_indexs}
    }

    pub fn build(&self) -> Box<[FirstLevel]> {
        let mut tmp: Vec<FirstLevel> = Vec::with_capacity(U40_HALF_SIZE);
        for i in 0..tmp.capacity() {
            tmp.push(FirstLevel::new((1<<10)/64, Some(self.root_table[i].objects.clone())));
        }
        let mut result: Box<[FirstLevel]> = tmp.into_boxed_slice();

        for i in self.root_indexs.clone() {
            for _ in self.root_table[i].objects.clone() {
                result[i].objects.push(SecondLevel::new(1<<10, None));
            }

            for key in self.root_table[i].objects.clone() {
                let len = self.root_table[i].hash_map.get(&key).unwrap().objects.len();
                build_lx_top(&mut result[i].lx_top, key);
                let keys = self.root_table[i].hash_map.get(&key).unwrap().objects.clone();
                result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].hasher = 
                    Some(Mphf::new_parallel(2.0,&keys, None));
                result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].keys = keys;
                    
                for _ in 0..len {
                    for sub_key in self.root_table[i].hash_map.get(&key).unwrap().objects.clone() {
                        build_lx_top(&mut result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].lx_top,sub_key);
                    }
                    result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].objects.push(None);
                } 
            }

        }
        result
    }

    pub fn build_root_top(&self) -> (Box<[u64; U40_HALF_SIZE/64]>,Box<[u64; U40_HALF_SIZE/64/64]>) {
        let mut root_top: [u64; U40_HALF_SIZE/64] = [0; U40_HALF_SIZE/64];
        let mut root_top_sub: [u64; U40_HALF_SIZE/64/64] = [0; U40_HALF_SIZE/64/64];
        for i in self.root_indexs.clone() {
            // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
            let bit = i/64;
            let bit_in_mask: u64  = 1<<(63-(i%64));
            root_top[bit] = root_top[bit] | bit_in_mask;

            // Berechnung des Indexs (sub_bit) im root_top_sub array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
            let sub_bit = bit/64;
            let sub_bit_in_mask: u64 = 1<<(63-(bit%64));
            root_top_sub[sub_bit] = root_top_sub[sub_bit] | sub_bit_in_mask;
        }
        (Box::new(root_top),Box::new(root_top_sub))
    }
}

// Annahme: Größe des lx_top-Arrays 2^10 Elemente
fn build_lx_top(lx_top: &mut Vec<u64>, key: u10) {
    let key = u16::from(key);

    let index = (key/64) as usize;
    let in_index_mask = 1<<(63-(key % 64));
    lx_top[index] = lx_top[index] | in_index_mask;
}
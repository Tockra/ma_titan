use std::collections::HashMap;
use std::mem::{self, MaybeUninit};
use std::ptr;

use ux::{u10,u40};
use boomphf::Mphf;

use crate::help::internal::{Splittable,root_size};
use crate::data_structures::statics::{FirstLevel, SecondLevel};

type SecondLevelBuild = PerfectHashBuilderLevel<usize>;
type FirstLevelBuild = PerfectHashBuilderLevel<SecondLevelBuild>;
type Int = u40;
pub struct PerfectHashBuilder {
    root_table: [FirstLevelBuild; root_size::<Int>()],
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
        let mut root_table = {
            let mut data: [MaybeUninit<FirstLevelBuild>; root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut data[..] {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), FirstLevelBuild::new((1<<10)/64)); 
                }
            }

            unsafe { 
                mem::transmute::<_, [FirstLevelBuild; root_size::<Int>()]>(data) 
            }
        };
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

    pub fn build(&self) -> [FirstLevel; root_size::<Int>()] {
        let mut result: [FirstLevel; root_size::<Int>()] = {
            let mut data: [MaybeUninit<FirstLevel>; root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for (i, elem) in data.iter_mut().enumerate() {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), FirstLevel::new((1<<10)/64, Some(self.root_table[i].objects.clone()))); 
                }
            }

            unsafe { 
                mem::transmute::<_, [FirstLevel; root_size::<Int>()]>(data) 
            }
        };
        for i in self.root_indexs.clone() {
            for _ in self.root_table[i].objects.clone() {
                result[i].objects.push(SecondLevel::new(1<<10, None));
            }

            for key in self.root_table[i].objects.clone() {
                let len = self.root_table[i].hash_map.get(&key).unwrap().objects.len();

                result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].hasher = 
                    Some(Mphf::new_parallel(2.0,&self.root_table[i].hash_map.get(&key).unwrap().objects.clone(), None));
                for _ in 0..len {
                    result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].objects.push(None);
                } 
            }

        }
        result
    }

    pub fn build_root_top(&self) -> ([u64; root_size::<Int>()/64],[u64; root_size::<Int>()/64/64]){
        let mut root_top: [u64; root_size::<Int>()/64] = [0; root_size::<Int>()/64];
        let mut root_top_sub: [u64; root_size::<Int>()/64/64] = [0; root_size::<Int>()/64/64];
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
        (root_top,root_top_sub)
    }
}
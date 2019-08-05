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

    pub fn build(self) -> [FirstLevel; root_size::<Int>()] {
        let mut result: [FirstLevel; root_size::<Int>()] = {
            let mut data: [MaybeUninit<FirstLevel>; super::internal::root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for (i, elem) in data.iter_mut().enumerate() {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), FirstLevel::new((1<<10)/64, Some(self.root_table[i].objects.clone()))); 
                }
            }

            unsafe { 
                mem::transmute::<_, [FirstLevel; super::internal::root_size::<Int>()]>(data) 
            }
        };
        for i in self.root_indexs {
            for _ in self.root_table[i].objects.clone() {
                result[i].objects.push(SecondLevel::new(1<<10, None));
            }

            for key in self.root_table[i].objects.clone() {
                result[i].objects[result[i].hasher.as_ref().unwrap().hash(&key) as usize].hasher = 
                    Some(Mphf::new_parallel(2.0,&self.root_table[i].hash_map.get(&key).unwrap().objects.clone(), None));
            }

        }
        result
    }
}
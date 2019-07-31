#![allow(dead_code)]  
pub type Int = i32;
type SecondLevel = Level<*mut Element<Int>,Int>;
type FirstLevel = Level<SecondLevel,Int>;

use super::dynamics::{Level,STree};
use super::internal::{List,Element};

use std::ptr;
use std::mem::{self, MaybeUninit};
#[test]
pub fn locate() {
    test_locate_root_top();
}

/**
 * Zum Testen der locate_top-Methoden werden hier einige u64 Arrays testweise angelegt. Dabei gibt es immer Varianten a,b und c:unimplemented!
 * Jede Variante wird am Anfang vom untersuchten Array [0], im Untersuchten Array [0<x<len] und am Ende des Arrays [len-1] eingefügt.
 * a: Letztes Bit ist gesetzt
 * b: erstes Bit und "andere" sind gesetzt
 * c: mitten im u64 ist ein Bit gesetzt
 */
pub fn test_locate_root_top() {
    // root_top mit Inhalt in [0]
    let mut root_top_sub_start = [0 as u64; root_size::<Int>()/64/64];
    root_top_sub_start[0] = 0b10000000000000000000000000000000_00000000000000000000000000000000;

    // locate(i) mit i<= 63 sollte 63 zurückgeben.
    let mut root_top_start_a = [0 as u64; root_size::<Int>()/64];
    root_top_start_a[0] = 1;
    
    let _stree_start_a = new_stree(root_top_start_a, root_top_sub_start);

    // locate(i) mit i=0 sollte 0 zurückgeben.
    let mut root_top_start_b = [0 as u64; root_size::<Int>()/64];
    root_top_start_b[0] = 0b10000000000000000000000000000000_10000000000000000000000000000000;

    // locate(i) mit i<= 32 sollte 32 zurückgeben.
    let mut root_top_start_c = [0 as u64; root_size::<Int>()/64];
    root_top_start_c[0] = 0b00000000000000000000000000000000_10000000000000000000000000000000;

    // root_top mit Inhalt in der Mitte[512]
    let mut root_top_sub_middle = [0 as u64; root_size::<Int>()/64/64];
    root_top_sub_middle[8] = 0b10000000000000000000000000000000_00000000000000000000000000000000;
    // locate(i) mit i<= 32831 sollte 32831 zurückgeben.
    let mut root_top_middle_a = [0 as u64; root_size::<Int>()/64];
    root_top_middle_a[root_size::<Int>()/64/2] = 1;

    // locate(i) mit i<=32768 sollte 768 zurückgeben.
    let mut root_top_middle_b = [0 as u64; root_size::<Int>()/64];
    root_top_middle_b[root_size::<Int>()/64/2] = 0b10000000000000000000000000000000_10000000000000000000000000000000;

    // locate(i) mit i<= 32.800‬ sollte 32.800‬ zurückgeben.
    let mut root_top_middle_c = [0 as u64; root_size::<Int>()/64];
    root_top_middle_c[root_size::<Int>()/64/2] = 0b00000000000000000000000000000000_10000000000000000000000000000000;

    
    // root_top mit Inhalt am Ende [1023]
    let mut root_top_sub_end = [0 as u64; root_size::<Int>()/64/64];
    root_top_sub_end[15] = 1;
    // locate(i) mit i<= 65.535 sollte 65.535 zurückgeben.
    let mut root_top_end_a = [0 as u64; root_size::<Int>()/64];
    root_top_end_a[root_size::<Int>()/64 -1 ] = 1;

    // locate(i) mit i=65.472 sollte 65.472 zurückgeben.
    let mut root_top_end_b = [0 as u64; root_size::<Int>()/64];
    root_top_end_b[root_size::<Int>()/64/2 - 1] = 0b10000000000000000000000000000000_10000000000000000000000000000000;

    // locate(i) mit i<= 65.504 sollte 65.504 zurückgeben.
    let mut root_top_end_c = [0 as u64; root_size::<Int>()/64];
    root_top_end_c[root_size::<Int>()/64/2 - 1] = 0b00000000000000000000000000000000_10000000000000000000000000000000;
}

fn new_stree(root_top: [u64; root_size::<Int>()/64], root_top_sub: [u64; root_size::<Int>()/64/64]) -> STree {
    let data = {
            let mut data: [MaybeUninit<FirstLevel>; root_size::<Int>()] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for elem in &mut data[..] {
                unsafe { 
                    ptr::write(elem.as_mut_ptr(), FirstLevel::new((1<<8)/64)); 
                }
            }

            unsafe { 
                mem::transmute::<_, [FirstLevel; root_size::<Int>()]>(data) 
            }
        };
    STree {
            element_list: List::new(),
            root_top: root_top,
            root_top_sub: root_top_sub,
            root_table: data,
    }
}

const fn root_size<T>() -> usize {
    1 << 8*mem::size_of::<T>() / 2
}

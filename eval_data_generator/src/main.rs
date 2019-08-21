extern crate serde;
extern crate rmp_serde as rmps;

use serde::{ Serialize};
use rmps::{ Serializer};
use std::fs::File;
use std::io::BufWriter;
use rand_pcg::Mcg128Xsl64;
use rand::seq::IteratorRandom;

use uint::Int;
use uint::Typable;


const SEED: u128 = 0xcafef00dd15ea5e5;
const TWO: u32 = 2;
fn main() {
    let mut state = Mcg128Xsl64::new(SEED);

    let mut result: Vec<u64> = (0..((1u64<<40))).choose_multiple(&mut state, ((1u64<<34)) as usize);
    let mut clone = result.clone();
    clone.sort();
    write_to_file(format!("testdata/u40/2^{}.data", 40),&clone);
    for i in (0..40).rev() {
        let cut = result.len() - ((1u64<<40) - (1<<i)) as usize; 
        result = result.split_off(cut);
        let mut clone = result.clone();
        clone.sort();
        write_to_file(format!("testdata/u40/2^{}.data", i),&clone);
    }
}

// (1u64<<34)
fn generate_test_data<T: Typable + Into<u64>>(max_value: usize) {
    let mut state = Mcg128Xsl64::new(SEED);

    let mut result: Vec<u64> = (0u64..(T::max_value()).into()).choose_multiple(&mut state, max_value);
    let mut clone = result.clone();
    clone.sort();
    write_to_file(format!("../testdata/{}/2^{}.data", T::TYPE,40),&clone);
    for i in (0..40).rev() {
        let cut = result.len() - (max_value - (1<<i) as usize); 
        result = result.split_off(cut);
        let mut clone = result.clone();
        clone.sort();
        write_to_file(format!("../testdata/{}/2^{}.data",T::TYPE, i),&clone);
    }
}

/// Generiert `n` Normalverteilte Werte im u40 BereichDateien und speichert diese in der Datei "`n`.data"
/*fn generate_values(n: usize) {
    let mut values = vec![];
    let mut state = Mcg128Xsl64::new(SEED);
    for _ in 0..n {
        let mut x = state.gen_range(0, u64::from(u40::max_value()));
        while values.contains(&u40::from(x)) {
            x = state.gen_range(0, u64::from(u40::max_value()));
        }
        values.push(u40::from(x));
    }
    write_to_file(format!("testdata/u40/{}.data", n), &values);
}*/

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file(name: String, val: &Vec<u64>) {
    let mut buf = BufWriter::new(File::create(name).unwrap());
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
}
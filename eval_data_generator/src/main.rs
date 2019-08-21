extern crate serde;
extern crate rmp_serde as rmps;

use serde::{ Serialize};
use rmps::{ Serializer};
use std::fs::File;
use std::io::BufWriter;
use rand_pcg::Mcg128Xsl64;
use rand::seq::IteratorRandom;

use uint::u40;
use uint::Typable;


const SEED: u128 = 0xcafef00dd15ea5e5;
fn main() {
    generate_test_data::<u40>(18);
}

fn generate_test_data<T: Typable + Into<u64>>(exponent: u64) {
    let mut state = Mcg128Xsl64::new(SEED);
    let max_value = (1u64<<exponent) as usize;
    let mut result: Vec<u64> = (0u64..(T::max_value()).into()).choose_multiple(&mut state, max_value);

    for i in 0..exponent {
        let cut = result.len() - (max_value - (1<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();
        write_to_file(format!("../testdata/{}/2^{}.data",T::TYPE, i),&result.to_vec());
    }

    result.sort();
    write_to_file(format!("../testdata/{}/2^{}.data",T::TYPE, exponent),&result);
}

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file(name: String, val: &Vec<u64>) {
    let mut buf = BufWriter::new(File::create(name).unwrap());
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
}
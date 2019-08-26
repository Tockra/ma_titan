extern crate serde;
extern crate rmp_serde as rmps;

use serde::{Serialize};
use rmps::{Serializer};
use std::fs::File;
use std::io::BufWriter;
use rand_pcg::Mcg128Xsl64;
use rand::seq::IteratorRandom;
use std::cmp::Ord;

use uint::u40;
use uint::Typable;


const SEED: u128 = 0xcafef00dd15ea5e5;

fn main() {
    generate_test_data::<u40>(18);
}

/// Diese Methode generiert 2^`exponent`viele unterschiedliche sortierte Zahlen vom Typ u40, u48 und u64.AsMut
/// Dabei werden Dateien von 2^0 bis hin zu 2^`exponent` angelegt.
fn generate_test_data<T: Typable + Serialize + Ord + Copy + Into<u64> + From<u64>>(exponent: u64) {
    let mut state = Mcg128Xsl64::new(SEED);
    let max_value = (1u64<<exponent) as usize;
    let mut result: Vec<u64> = (0u64..(T::max_value()).into()).choose_multiple(&mut state, max_value);
    let mut result = result.into_iter().map(|v| T::from(v)).collect::<Vec<T>>();
    for i in 0..exponent {
        let cut = result.len() - (max_value - (1u64<<i) as usize); 
        let result = &mut result[..cut];
        result.sort();
        write_to_file(format!("../testdata/{}/2^{}.data",T::TYPE, i),&result.to_vec());
    }

    result.sort();
    write_to_file(format!("../testdata/{}/2^{}.data",T::TYPE, exponent),&result);
}

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file<T: Typable + Serialize>(name: String, val: &Vec<T>) {
    let mut buf = BufWriter::new(File::create(name).unwrap());
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
}
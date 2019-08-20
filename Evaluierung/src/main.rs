extern crate serde;
extern crate rmp_serde as rmps;

use serde::{ Serialize};
use rmps::{ Serializer};
use std::fs::File;
use std::io::BufWriter;
use rand::Rng;
use rand_pcg::Mcg128Xsl64;
use uint::u40;

const SEED: u128 = 0xcafef00dd15ea5e5;
fn main() {
    let two: u64 = 2;
    for i in 0..41 {
        generate_values(two.pow(i) as usize);
        println!("Fortschritt: {}%",((i as f32/41.) *100.) as u8);
    }
}

/// Generiert `n` Normalverteilte Werte im u40 BereichDateien und speichert diese in der Datei "`n`.data"
fn generate_values(n: usize) {
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
}

/// Serializiert den übergebenen Vector und schreibt diesen in eine Datei namens `name`.
fn write_to_file(name: String, val: &Vec<u40>) {
    let mut buf = BufWriter::new(File::create(name).unwrap());
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
}
use criterion::criterion_main;

use ma_titan::benches::static_build_benchmark;
use std::collections::BTreeMap;
use uint::u40;

criterion_main!(static_build_benchmark<u40,BTreeMap<u40,u40>>);

use criterion::criterion_main;

use ma_titan::benches::pred_and_succ_benchmark;
use std::collections::BTreeMap;
use uint::u40;

criterion_main!(pred_and_succ_benchmark<u40,BTreeMap<u40,u40>>);

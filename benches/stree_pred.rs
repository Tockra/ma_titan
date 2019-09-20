use criterion::criterion_main;

use ma_titan::benches::pred_and_succ_benchmark;
use ma_titan::default::immutable::STree;
use uint::u40;

criterion_main!(pred_and_succ_benchmark<u40,STree<u40>>);

use criterion::criterion_main;

use ma_titan::benches::static_build_benchmark;
use ma_titan::default::immutable::STree;
use uint::u40;

criterion_main!(static_build_benchmark<u40,STree<u40>>);
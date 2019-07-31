/*use internal::{ PredecessorSet};
use ux::i40;

pub type Int = i40;
type SecondLevel = Level<*mut Element<Int>,Int>;
type FirstLevel = Level<SecondLevel,Int>;

pub struct STree {
    root_table: [FirstLevel; root_size::<Int>()],
    // Da die Größe in in Bytes von size_of zurückgegeben wird, mal 8. Durch 64, da 64 Bits in einen u64 passen.
    root_top: [u64; root_size::<Int>()/64],
    root_top_sub: [u64; root_size::<Int>()/64/64], //Hier nur ein Element, da 2^16/64/64 nur noch 16 Bit sind, die alle in ein u64 passen!
    element_list: Vec<Int>,
}*/
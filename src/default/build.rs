use crate::default::immutable::Int;



#[inline]
pub fn build_lx_top(lx_top: &mut [u64], key: u16) {
    let key = u16::from(key);

    let index = (key/64) as usize;
    let in_index_mask = 1<<(63-(key % 64));
    lx_top[index] = lx_top[index] | in_index_mask;
}

pub fn build_root_top(root_top: &mut Box<[Box<[u64]>]>, bit: &usize) {
    // Berechnung des Indexs (bits) im root_top array und des internen Offsets bzw. der Bitmaske mit einer 1 ander richtigen Stelle
    for i in 0..root_top.len() {
        let curr_bit_repr = bit/(1<<(i*6));
        let index = curr_bit_repr/64;
        let bit_mask: u64  = 1<<(63-(curr_bit_repr%64));
        root_top[i][index] = root_top[i][index] | bit_mask;
    }
}

/// Baut das Root-Top-Array mit Hilfe der sich in der Datenstruktur befindenden Werte.
#[inline]
pub fn create_root_top<T:Int>() -> Box<[Box<[u64]>]>{
    // root_top_deep verwenden um die richtige Tiefe von root_top zu bestimmen
    let mut root_top_deep = 0;
    while T::root_array_size()/(1<<root_top_deep*6) > 256 {
        root_top_deep +=1;
    }

    let mut root_top: Vec<Box<Vec<u64>>> = Vec::with_capacity(root_top_deep);

    for i in 0..root_top.capacity() {
        root_top.push(Box::new(vec![0;T::root_array_size()/(1<<i*6)]));
    }

    let root_top: Box<[Box<[u64]>]> = root_top.into_iter().map(|x| x.into_boxed_slice()).collect::<Vec<_>>().into_boxed_slice();

    root_top
}
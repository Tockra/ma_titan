use crate::default::immutable::Int;
use crate::default::immutable::{Level,L3Ebene};
use crate::internal::{Splittable,PointerEnum};
#[inline]
pub fn insert_l3_level<T: Int + Into<u64>>(l3_level: &mut L3Ebene<T>,index: usize, k: u8, elements: &[T]) {
    if l3_level.is_null() {
        *l3_level = L3Ebene::from_usize(Box::new(index));
    } else {
        match l3_level.get() {
            PointerEnum::First(l3_level) => {

                debug_assert!(!l3_level.lx_top.is_set(k as usize));
                l3_level.lx_top.set_bit(k as usize);
            
                //Maximasetzung auf der zweiten Ebene
                l3_level.maximum = index;

                l3_level.hash_map.insert(k, index);
            },

            PointerEnum::Second(e) => {
                let (_,_,k2) = Splittable::split_integer_down(&elements[*e]);
                let mut l3_level_n = Level::new();

                debug_assert!(k2!=k);

                    // Minima- und Maximasetzung auf der zweiten Ebene
                l3_level_n.minimum = *e;
                l3_level_n.maximum = index;

                l3_level_n.hash_map.insert(k2, *e);
                l3_level_n.hash_map.insert(k, index);
                l3_level_n.lx_top.set_bit(k as usize);
                l3_level_n.lx_top.set_bit(k2 as usize);
            
                
                *l3_level = L3Ebene::from_level(Box::new(l3_level_n));
            }
        }
    }
}

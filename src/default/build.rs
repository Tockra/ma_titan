use crate::default::immutable::Int;
use crate::default::immutable::{Level,L3Ebene};
use crate::internal::{Splittable,PointerEnum};
#[inline]
pub fn insert_l3_level<T: Int + Into<u64>>(l3_level: &mut L3Ebene<T>,elem: &T, k: u16, elements: &[T]) {
    if l3_level.is_null() {
        *l3_level = L3Ebene::from_usize(elem as *const T);
    } else {
        match l3_level.get() {
            PointerEnum::First(l3_level) => {

                debug_assert!(!l3_level.lx_top.is_set(k as usize));
                l3_level.lx_top.set_bit(k as usize);
            
                //Maximasetzung auf der zweiten Ebene
                l3_level.maximum = elem as *const T;

                l3_level.hash_map.insert(k, elem as *const T);
            },

            PointerEnum::Second(e) => {
                let (_,_,k2) = Splittable::split_integer_down(elem);
                let mut l3_level_n = Level::new();

                debug_assert!(k2!=k);

                    // Minima- und Maximasetzung auf der zweiten Ebene
                l3_level_n.minimum = e as *const T;
                l3_level_n.maximum = elem  as *const T;

                l3_level_n.hash_map.insert(k2, e as *const T);
                l3_level_n.hash_map.insert(k, elem as *const T);
                l3_level_n.lx_top.set_bit(k as usize);
                l3_level_n.lx_top.set_bit(k2 as usize);
            
                
                *l3_level = L3Ebene::from_level(Box::new(l3_level_n));
            }
        }
    }
}

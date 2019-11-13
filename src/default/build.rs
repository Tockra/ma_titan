use crate::default::immutable::Int;
use crate::default::immutable::{Level,L3Ebene, L2Ebene, LXKey};
use crate::internal::{Splittable,PointerEnum};
#[inline]
pub fn insert_l3_level<T: Int + Into<u64>>(l3_level: &mut L3Ebene<T>,elem: &T, k: LXKey, elements: &[T]) {
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
                let (_,_,_,k2) = Splittable::split_integer_down(e);
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

pub fn insert_l2_level<T: Int + Into<u64>>(l2_level: &mut L2Ebene<T>,elem: &T, elements: &[T]) {
    let (_,_,j,k) = Splittable::split_integer_down(elem);
    if l2_level.is_null() {
        *l2_level = L2Ebene::from_usize(elem as *const T);
    } else {
        match l2_level.get() {
            PointerEnum::First(l2_object) => {
                l2_object.maximum = elem as *const T;

                if !l2_object.lx_top.is_set(j as usize) {
                    let mut l3_level = L3Ebene::from_null();
                    insert_l3_level(&mut l3_level,elem,k,&elements);

                    l2_object.hash_map.insert(j,l3_level);
                    l2_object.lx_top.set_bit(j as usize);
                } else {
                    // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                    insert_l3_level(l2_object.hash_map.get_mut(j).unwrap(),elem,k,&elements);
                }
            },

            PointerEnum::Second(elem_index) => {
                let mut l2_object = Level::new();
                let (_,_,j2,k2) = Splittable::split_integer_down(elem_index);
                
                // Da die Elemente sortiert sind
                l2_object.minimum = elem_index as *const T;
                l2_object.maximum = elem as *const T;

                l2_object.lx_top.set_bit(j as usize);

                let mut l3_level = L3Ebene::from_null();

                if j2 != j {
                    let mut l3_level = L3Ebene::from_null();
                    insert_l3_level(&mut l3_level,elem_index,k2,&elements);

                    l2_object.hash_map.insert(j2,l3_level);
                    l2_object.lx_top.set_bit(j2 as usize)
                } else {
                    insert_l3_level(&mut l3_level,elem_index,k2,&elements);
                }

                insert_l3_level(&mut l3_level,elem,k,&elements);
                l2_object.hash_map.insert(j,l3_level);

                *l2_level = L2Ebene::from_level(Box::new(l2_object));
            }
        }
    }
}

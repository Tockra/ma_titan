use crate::default::immutable::Int;
use crate::default::immutable::{Level,L3Ebene, L2Ebene, LXEbene, LYEbene};
use crate::internal::{Splittable,PointerEnum};
#[inline]
pub fn insert_l3_level<T: Int + Into<u64>>(l3_level: &mut L3Ebene<T>,elem: &T, elements: &[T], k: u8) {
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
                let (_,_,_,_,_,k2) = Splittable::split_integer_down(e);
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

pub fn insert_ly_level<T: Int + Into<u64>>(ly_level: &mut LYEbene<T>,elem: &T, elements: &[T], y: u8, k: u8) {
    if ly_level.is_null() {
        *ly_level = LYEbene::from_usize(elem as *const T);
    } else {
        match ly_level.get() {
            PointerEnum::First(ly_object) => {
                ly_object.maximum = elem as *const T;

                if !ly_object.lx_top.is_set(y as usize) {
                    let mut l3_level = L3Ebene::from_null();
                    insert_l3_level(&mut l3_level,elem,&elements, k);

                    ly_object.hash_map.insert(y,l3_level);
                    ly_object.lx_top.set_bit(y as usize);
                } else {
                    // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                    insert_l3_level(ly_object.hash_map.get_mut(&y).unwrap(),elem,&elements, k);
                }
            },

            PointerEnum::Second(elem_index) => {
                let mut ly_object = Level::new();
                let elem2 = *elem_index;
                let (_,_,_, _, y2, k2) = Splittable::split_integer_down(&elem2);
                
                // Da die Elemente sortiert sind
                ly_object.minimum = elem_index as *const T;
                ly_object.maximum = elem as *const T;

                ly_object.lx_top.set_bit(y as usize);

                let mut l3_level = L3Ebene::from_null();

                if y2 != y {
                    let mut l3_level = L3Ebene::from_null();
                    insert_l3_level(&mut l3_level,elem_index,&elements, k2);

                    ly_object.hash_map.insert(y2,l3_level);
                    ly_object.lx_top.set_bit(y2 as usize)
                } else {
                    insert_l3_level(&mut l3_level,elem_index,&elements, k2);
                }

                insert_l3_level(&mut l3_level,elem,&elements, k);
                ly_object.hash_map.insert(y,l3_level);

                *ly_level = LYEbene::from_level(Box::new(ly_object));
            }
        }
    }
}

pub fn insert_lx_level<T: Int + Into<u64>>(lx_level: &mut LXEbene<T>,elem: &T, elements: &[T], x: u8, y: u8, k: u8) {
    if lx_level.is_null() {
        *lx_level = LXEbene::from_usize(elem as *const T);
    } else {
        match lx_level.get() {
            PointerEnum::First(lx_object) => {
                lx_object.maximum = elem as *const T;

                if !lx_object.lx_top.is_set(x as usize) {
                    let mut ly_level = LYEbene::from_null();
                    insert_ly_level(&mut ly_level,elem,&elements, y, k);

                    lx_object.hash_map.insert(x,ly_level);
                    lx_object.lx_top.set_bit(x as usize);
                } else {
                    // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                    insert_ly_level(lx_object.hash_map.get_mut(&x).unwrap(),elem,&elements, y, k);
                }
            },

            PointerEnum::Second(elem_index) => {
                let mut lx_object = Level::new();
                let elem2 = *elem_index;
                let (_,_,_, x2, y2, k2) = Splittable::split_integer_down(&elem2);
                
                // Da die Elemente sortiert sind
                lx_object.minimum = elem_index as *const T;
                lx_object.maximum = elem as *const T;

                lx_object.lx_top.set_bit(x as usize);

                let mut ly_level = LYEbene::from_null();

                if x2 != x {
                    let mut ly_level = LYEbene::from_null();
                    insert_ly_level(&mut ly_level,elem_index,&elements, y2, k2);

                    lx_object.hash_map.insert(x2,ly_level);
                    lx_object.lx_top.set_bit(x2 as usize)
                } else {
                    insert_ly_level(&mut ly_level,elem_index,&elements, y2, k2);
                }

                insert_ly_level(&mut ly_level,elem,&elements, y, k);
                lx_object.hash_map.insert(x,ly_level);

                *lx_level = LXEbene::from_level(Box::new(lx_object));
            }
        }
    }
}

pub fn insert_l2_level<T: Int + Into<u64>>(l2_level: &mut L2Ebene<T>,elem: &T, elements: &[T], j: u8, x: u8, y: u8, k: u8 ) {
    if l2_level.is_null() {
        *l2_level = L2Ebene::from_usize(elem as *const T);
    } else {
        match l2_level.get() {
            PointerEnum::First(l2_object) => {
                l2_object.maximum = elem as *const T;

                if !l2_object.lx_top.is_set(j as usize) {
                    let mut lx_level = LXEbene::from_null();
                    insert_lx_level(&mut lx_level,elem,&elements, x, y, k);

                    l2_object.hash_map.insert(j,lx_level);
                    l2_object.lx_top.set_bit(j as usize);
                } else {
                    // Hier fängt das unwrap() Implementierungsfehler ab, die den keys-Vektor nicht äquivalent zur Hashmap befüllen *outdated*
                    insert_lx_level(l2_object.hash_map.get_mut(&j).unwrap(),elem,&elements, x, y, k);
                }
            },

            PointerEnum::Second(elem_index) => {
                let mut l2_object = Level::new();
                let elem2 = *elem_index;
                let (_,_,j2,x2,y2,k2) = Splittable::split_integer_down(&elem2);
                
                // Da die Elemente sortiert sind
                l2_object.minimum = elem_index as *const T;
                l2_object.maximum = elem as *const T;

                l2_object.lx_top.set_bit(j as usize);

                let mut lx_level = LXEbene::from_null();

                if j2 != j {
                    let mut lx_level = LXEbene::from_null();
                    insert_lx_level(&mut lx_level,elem_index,&elements, x2, y2, k2);

                    l2_object.hash_map.insert(j2,lx_level);
                    l2_object.lx_top.set_bit(j2 as usize)
                } else {
                    insert_lx_level(&mut lx_level,elem_index,&elements, x2, y2, k2);
                }

                insert_lx_level(&mut lx_level,elem,&elements, x, y, k);
                l2_object.hash_map.insert(j,lx_level);

                *l2_level = L2Ebene::from_level(Box::new(l2_object));
            }
        }
    }
}

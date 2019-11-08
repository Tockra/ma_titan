
use crate::default::immutable::STree;
use crate::internal::{PointerEnum, Splittable};

/// Größe der LX-Top-Arrays 40 Bit
const LX_ARRAY_SIZE_U40: usize = 1 << 10;

/// Größe der LX-Top-Arrays 48 Bit
const LX_ARRAY_SIZE_U64: usize = 1 << 16;

/// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
#[test]
fn test_u64_new_hashfunctions() {
    // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
    // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
    let mut data: Vec<u64> = vec![0; LX_ARRAY_SIZE_U64];

    for i in 0..data.len() {
        data[i] = i as u64;
    }

    let check = data.clone();
    let data_structure: STree<u64> = STree::new(data.into_boxed_slice());

    assert_eq!(data_structure.len(), check.len());
    assert_eq!(data_structure.minimum().unwrap(), 0_u64);
    assert_eq!(
        data_structure.maximum().unwrap(),
        check.len() as u64 - 1
    );
    for val in check {
        let (i, l, j, k) = Splittable::split_integer_down(&val);
        match data_structure.root_table[i].get() {
            PointerEnum::First(level) => {
                let second_level = level.get(l);
                let saved_val = match second_level.get() {
                    PointerEnum::First(l) => {
                        let l2_level = l.get(j);
                        match l2_level.get() {
                            PointerEnum::First(l) => {
                                *(*l).get(k)
                            }
                            PointerEnum::Second(e) => *e,
                        }

                    }
                    PointerEnum::Second(e) => *e,
                };
                assert_eq!(data_structure.element_list[saved_val], val);
            }

            PointerEnum::Second(e) => {
                assert_eq!(data_structure.element_list[*e], val);
            }
        };
    }
}

/// Die locate_or_succ-Funktion wird getestet. Dabei werden beliebige Werte in ein STree gegeben und anschließend wird
/// `locate_or_succ(x) mit allen x zwischen STree.min() und STree.max() getestet.
#[test]
fn test_u64_locate_or_succ_bruteforce() {
    let data_v1: Vec<u64> = vec![
        0, 1, 3, 23, 123, 232, 500, 20000, 30000, 50000, 100000, 200000, 200005, 1065983, 1865983,
    ];

    let data_structure: STree<u64> = STree::new(data_v1.clone().into_boxed_slice());
    for (index, _) in data_v1.iter().enumerate() {
        if index < data_v1.len() - 1 {
            for i in data_v1[index] + 1..data_v1[index + 1] + 1 {
                let locate = data_structure.locate_or_succ(i as u64).unwrap();
                assert_eq!(
                    data_structure.element_list[locate],
                    data_v1[index + 1]
                );
            }
        }
    }
}

#[test]
fn test_u64_locate_or_succ_eqc_bruteforce_test() {
    let data: Vec<u64> = vec![
        0b000000000000000000000000_000000000000_000000000001,
        0b000000000000000000000000_000000000000_000001110000,
        0b000000000000000000000000_000000000000_111111111111,
        0b000000000000000000000000_000001110000_000000000000,
        0b000000000000000000000000_000001110000_000001110000,
        0b000000000000000000000000_000001110000_111111111111,
        0b000000000000000000000000_111111111111_000000000001,
        0b000000000000000000000000_111111111111_000001110000,
        0b000000000000000000000000_111111111111_111111111111,
        0b000000001100000000000011_000000000000_000000000001,
        0b000000001100000000000011_000000000000_000001110000,
        0b000000001100000000000011_000000000000_111111111111,
        0b000000001100000000000011_000001110000_000000000000,
        0b000000001100000000000011_000001110000_000001110000,
        0b000000001100000000000011_000001110000_111111111111,
        0b000000001100000000000011_111111111111_000000000001,
        0b000000001100000000000011_111111111111_000001110000,
        0b000000001100000000000011_111111111111_111111111111,
        0b111111111111111111111111_000000000000_000000000001,
        0b111111111111111111111111_000000000000_000001110000,
        0b111111111111111111111111_000000000000_111111111111,
        0b111111111111111111111111_000001110000_000000000000,
        0b111111111111111111111111_000001110000_000001110000,
        0b111111111111111111111111_000001110000_111111111111,
        0b111111111111111111111111_111111111111_000000000001,
        0b111111111111111111111111_111111111111_000001110000,
        0b111111111111111111111111_111111111111_111111111110,
    ];
    let data_structure: STree<u64> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(
        data_structure.locate_or_succ(
            0b111111111111111111111111_111111111111_111111111111_u64
        ),
        None
    );

    for (i, &elem) in data.iter().enumerate() {
        if i > 0 {
            for j in 0..16877216 {
                if elem >= j as u64 {
                    let index = elem - j;
                    if index > data_structure.element_list[i - 1] {
                        assert_eq!(
                            data_structure.element_list
                                [data_structure.locate_or_succ(index).unwrap() as usize],
                            elem
                        );
                    }
                }
            }
        } else {
            assert_eq!(
                data_structure.element_list[data_structure.locate_or_succ(elem).unwrap() as usize],
                elem
            );
            assert_eq!(
                data_structure.element_list
                    [data_structure.locate_or_succ(elem - 1).unwrap() as usize],
                elem
            );
        }
    }
}

/// Die locate_or_pred-Funktion wird getestet. Dabei werden beliebige (fest gewählte) Werte in ein STree gegeben und anschließend wird
/// `locate_or_pred(x) mit allen x zwischen STree.min() und STree.max() getestet.
#[test]
fn test_u64_locate_or_pred_bruteforce() {
    let data: Vec<u64> = vec![
        0, 1, 3, 23, 123, 232, 500, 20000, 30000, 50000, 100000, 200000, 200005, 1065983,
    ];

    let data_structure: STree<u64> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(
        1065983_u64,
        data_structure.element_list[data_structure.locate_or_pred(1065983).unwrap()]
    );
    for (index, _) in data.iter().enumerate().rev() {
        if index > 0 {
            for i in (data[index - 1]..data[index]).rev() {
                let locate = data_structure.locate_or_pred(i).unwrap();
                assert_eq!(
                    data[index - 1],
                    data_structure.element_list[locate]
                );
            }
        }
    }
}

use num::Bounded;

#[test]
fn test_u64_locate_or_pred_eqc_bruteforce_test() {
    let data: Vec<u64> = vec![
        0b00000000000000000000_0000000000_0000000001,
        0b00000000000000000000_0000000000_0000111000,
        0b00000000000000000000_0000000000_1111111111,
        0b00000000000000000000_0001110000_0000000000,
        0b00000000000000000000_0001110000_0000111000,
        0b00000000000000000000_0001110000_1111111111,
        0b00000000000000000000_1111111111_0000000000,
        0b00000000000000000000_1111111111_0000111000,
        0b00000000000000000000_1111111111_1111111111,
        0b00000000001111000000_0000000000_0000000000,
        0b00000000001111000000_0000000000_0000111000,
        0b00000000001111000000_0000000000_1111111111,
        0b00000000001111000000_0001110000_0000000000,
        0b00000000001111000000_0001110000_0000111000,
        0b00000000001111000000_0001110000_1111111111,
        0b00000000001111000000_1111111111_0000000000,
        0b00000000001111000000_1111111111_0000111000,
        0b00000000001111000000_1111111111_1111111111,
        0b11111111111111111111_0000000000_0000000000,
        0b11111111111111111111_0000000000_0000111000,
        0b11111111111111111111_0000000000_1111111111,
        0b11111111111111111111_0001110000_0000000000,
        0b11111111111111111111_0001110000_0000111000,
        0b11111111111111111111_0001110000_1111111111,
        0b11111111111111111111_1111111111_0000000000,
        0b11111111111111111111_1111111111_0000111000,
        0b11111111111111111111_1111111111_1111111110,
    ];

    let data_structure: STree<u64> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(data_structure.locate_or_pred(0), None);

    for (i, &elem) in data.iter().enumerate().rev() {
        if i < data.len() - 1 {
            for j in 0..16877216 {
                if u64::max_value() > elem && j < u64::max_value() - elem {
                    let index = elem + j;
                    if index < data_structure.element_list[i + 1] {
                        assert_eq!(
                            data_structure.element_list
                                [data_structure.locate_or_pred(index).unwrap() as usize],
                            elem
                        );
                    }
                }
            }
        } else {
            assert_eq!(
                data_structure.element_list[data_structure.locate_or_pred(elem).unwrap() as usize],
                elem
            );
            assert_eq!(
                data_structure.element_list
                    [data_structure.locate_or_pred(elem + 1).unwrap() as usize],
                elem
            );
        }
    }
}

use crate::default::immutable::TopArray;
use rand_distr::{Distribution, Uniform};
/*#[test]*/
/// Fügt einige Bits in eine ArrayTop-Struktur und prüft anschließend, ob die Bits gesetted sind.
/// (Deaktiviert, da der Test sehr lange dauert)
fn test_top_array_set_bit() {
    let between = Uniform::from(0u64..(1 << 10));
    let mut rng = rand::thread_rng();

    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..230 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();

    let mut lxtop = TopArray::<u64, u16>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1 << 10) {
        assert_eq!(bits_set.contains(&i), lxtop.is_set(i));
    }

    for i in 0..bits_set.len() - 1 {
        assert_eq!(
            bits_set[i + 1],
            lxtop.get_next_set_bit(bits_set[i]).unwrap()
        );
    }

    for i in 1..bits_set.len() {
        assert_eq!(
            bits_set[i - 1],
            lxtop.get_prev_set_bit(bits_set[i]).unwrap()
        );
    }

    let between = Uniform::from(0u64..(1 << 12));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..230 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();

    let mut lxtop = TopArray::<u64, u16>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1 << 12) {
        assert_eq!(bits_set.contains(&i), lxtop.is_set(i));
    }

    for i in 0..bits_set.len() - 1 {
        assert_eq!(
            bits_set[i + 1],
            lxtop.get_next_set_bit(bits_set[i]).unwrap()
        );
    }

    for i in 1..bits_set.len() {
        assert_eq!(
            bits_set[i - 1],
            lxtop.get_prev_set_bit(bits_set[i]).unwrap()
        );
    }

    let between = Uniform::from(0u64..(1 << 16));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    let mut lxtop = TopArray::<u64, u16>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1 << 16) {
        assert_eq!(bits_set.contains(&i), lxtop.is_set(i));
    }

    for i in 0..bits_set.len() - 1 {
        assert_eq!(
            bits_set[i + 1],
            lxtop.get_next_set_bit(bits_set[i]).unwrap()
        );
    }

    for i in 1..bits_set.len() {
        assert_eq!(
            bits_set[i - 1],
            lxtop.get_prev_set_bit(bits_set[i]).unwrap()
        );
    }

    let between = Uniform::from(0u64..(1 << 20));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    let mut lxtop = TopArray::<u64, usize>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1 << 20) {
        assert_eq!(bits_set.contains(&i), lxtop.is_set(i));
    }

    for i in 0..bits_set.len() - 1 {
        assert_eq!(
            bits_set[i + 1],
            lxtop.get_next_set_bit(bits_set[i]).unwrap()
        );
    }

    for i in 1..bits_set.len() {
        assert_eq!(
            bits_set[i - 1],
            lxtop.get_prev_set_bit(bits_set[i]).unwrap()
        );
    }

    let between = Uniform::from(0u64..(1 << 22));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    let mut lxtop = TopArray::<u64, usize>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1 << 22) {
        assert_eq!(bits_set.contains(&i), lxtop.is_set(i));
    }

    for i in 0..bits_set.len() - 1 {
        assert_eq!(
            bits_set[i + 1],
            lxtop.get_next_set_bit(bits_set[i]).unwrap()
        );
    }

    for i in 1..bits_set.len() {
        assert_eq!(
            bits_set[i - 1],
            lxtop.get_prev_set_bit(bits_set[i]).unwrap()
        );
    }

    let between = Uniform::from(0u64..(1 << 32));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    println!("Did it");
    bits_set.sort();
    bits_set.dedup();
    println!("Did sort");
    let mut lxtop = TopArray::<u64, usize>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }
    println!("Did set");

    // Abgespeckt da das verdammt lange dauert!
    for i in 0..(1 << 22) {
        assert_eq!(bits_set.contains(&i), lxtop.is_set(i));
    }

    for i in 0..bits_set.len() - 1 {
        assert_eq!(
            bits_set[i + 1],
            lxtop.get_next_set_bit(bits_set[i]).unwrap()
        );
    }

    for i in 1..bits_set.len() {
        assert_eq!(
            bits_set[i - 1],
            lxtop.get_prev_set_bit(bits_set[i]).unwrap()
        );
    }
}

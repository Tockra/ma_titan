use uint::{u40,u48};
use crate::default::immutable::{STree};
use crate::default::immutable::LXKey;
use crate::internal::{PointerEnum,Splittable};

/// Größe der LX-Top-Arrays 40 Bit
const LX_ARRAY_SIZE_U40: usize = 1 << 10;

/// Größe der LX-Top-Arrays 48 Bit
const LX_ARRAY_SIZE_U48: usize = 1 << 12;

// u64 Tests werden ausgespart, da der STree (leer) nach Initialisierung 2^32 * 8 Byte = 34 Gbyte RAM benötigt
// Diese Tests sind nicht auf gängigen Laptop ausführbar. (Zukunft, ich rede von 2019 :p).

/// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
#[test]
fn test_u40_new_hashfunctions() {

    // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
    // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
    let mut data: Vec<u40> = vec![u40::new(0);LX_ARRAY_SIZE_U40];
    
    for i in 0..data.len() {
        data[i] = u40::new(i as u64);
    }

    let check = data.clone();
    let data_structure: STree<u40> = STree::new(data.into_boxed_slice());

    assert_eq!(data_structure.len(),check.len());
    assert_eq!(data_structure.minimum().unwrap(),u40::new(0));
    assert_eq!(data_structure.maximum().unwrap(),u40::new(check.len() as u64 - 1));
    for val in check {
        let (i,j,k) = Splittable::split_integer_down(&val);

        match data_structure.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l.get(j);
                let saved_val = match second_level.get() {
                    PointerEnum::First(l) => {
                        *(*l).get(k)
                    },
                    PointerEnum::Second(e) => {
                        *e
                    }
                };
                assert_eq!(data_structure.element_list[saved_val],val);
            },

            PointerEnum::Second(e) => {
                assert_eq!(data_structure.element_list[*e],val);
            }
        };

    }
}


/// Die internen (perfekten) Hashfunktionen werden nach dem Einfügen der Elemente auf die Funktionsfähigkeit geprüft.
#[test]
fn test_u48_new_hashfunctions() {
    // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
    // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
    let mut data: Vec<u48> = vec![u48::new(0);LX_ARRAY_SIZE_U48];
    
    for i in 0..data.len() {
        data[i] = u48::new(i as u64);
    }

    let check = data.clone();
    let data_structure: STree<u48> = STree::new(data.into_boxed_slice());

    assert_eq!(data_structure.len(),check.len());
    assert_eq!(data_structure.minimum().unwrap(),u48::new(0));
    assert_eq!(data_structure.maximum().unwrap(),u48::new(check.len() as u64 - 1));
    for val in check {
        let (i,j,k) = Splittable::split_integer_down(&val);
        match data_structure.root_table[i].get() {
            PointerEnum::First(l) => {
                let second_level = l.get(j);
                let saved_val = match second_level.get() {
                    PointerEnum::First(l) => {
                        *(*l).get(k)
                    },
                    PointerEnum::Second(e) => {
                        *e
                    }
                };
                assert_eq!(data_structure.element_list[saved_val],val);
            },

            PointerEnum::Second(e) => {
                assert_eq!(data_structure.element_list[*e],val);
            }
        };

    }
}

/// Die Top-Arrays werden geprüft. Dabei wird nur grob überprüft, ob sinnvolle Werte gesetzt wurden.
/// Dieser Test ist ein Kandidat zum Entfernen oder Erweitern.
#[test]
fn test_u40_top_arrays() {
    let data: Vec<u40> = vec![u40::new(0b00000000000000000000_1010010010_0101010101),u40::new(0b00000000000000000000_1010010010_0101010111),u40::new(0b11111111111111111111_1010010010_0101010101_u64)];
    let check = data.clone();
    let data_structure: STree<u40> = STree::new(data.into_boxed_slice());

    assert_eq!(data_structure.len(),check.len());
    assert_eq!(data_structure.minimum().unwrap(),u40::new(0b00000000000000000000_1010010010_0101010101));
    assert_eq!(data_structure.maximum().unwrap(),u40::new(0b11111111111111111111_1010010010_0101010101_u64));

    for val in check {
        let (i,j,k) = Splittable::split_integer_down(&val);
        if data_structure.root_table[i].minimum() != data_structure.root_table[i].maximum() {
            let second_level = match data_structure.root_table[i].get() {
                    PointerEnum::First(l) => {
                        l.get(j)
                    },
                    _ => {
                        panic!("Das sollte nicht geschehen");
                    }
            };
            if second_level.minimum() != second_level.maximum() {
                let saved_val = match second_level.get() {
                    PointerEnum::First(l) => {
                        l.get(k)
                    },
                    _ => {
                        panic!("Das sollte nicht geschehen");
                    }
                };
                assert_eq!(data_structure.element_list[*saved_val],val);
            } else {
                assert_eq!(data_structure.element_list[second_level.minimum()],val);
            }

        } else {
            assert_eq!(data_structure.element_list[data_structure.root_table[i].minimum()],val);
        }

    }
    
}

/// Die Top-Arrays werden geprüft. Dabei wird nur grob überprüft, ob sinnvolle Werte gesetzt wurden.
/// Dieser Test ist ein Kandidat zum Entfernen oder Erweitern.
#[test]
fn test_u48_top_arrays() {
    let data: Vec<u48> = vec![u48::new(0b10010010_00000000000000000000_1010010010_0101010101_u64),u48::new(0b10010010_00000000000000000000_1010010010_0101010111_u64),u48::new(0b11111111_11111111111111111111_1010010010_0101010101_u64)];
    let check = data.clone();
    let data_structure: STree<u48> = STree::new(data.into_boxed_slice());

    assert_eq!(data_structure.len(),check.len());
    assert_eq!(data_structure.minimum().unwrap(),u48::new(0b10010010_00000000000000000000_1010010010_0101010101_u64));
    assert_eq!(data_structure.maximum().unwrap(),u48::new(0b11111111_11111111111111111111_1010010010_0101010101_u64));

    for val in check {
        let (i,j,k) = Splittable::split_integer_down(&val);
        if data_structure.root_table[i].minimum() != data_structure.root_table[i].maximum() {
            let second_level = match data_structure.root_table[i].get() {
                    PointerEnum::First(l) => {
                        l.get(j)
                    },
                    _ => {
                        panic!("Das sollte nicht geschehen");
                    }
            };
            if second_level.minimum() != second_level.maximum() {
                let saved_val = match second_level.get() {
                    PointerEnum::First(l) => {
                        l.get(k)
                    },
                    _ => {
                        panic!("Das sollte nicht geschehen");
                    }
                };
                assert_eq!(data_structure.element_list[*saved_val],val);
            } else {
                assert_eq!(data_structure.element_list[second_level.minimum()],val);
            }

        } else {
            assert_eq!(data_structure.element_list[data_structure.root_table[i].minimum()],val);
        }

    }

    
}


/// Die locate_or_succ-Funktion wird getestet. Dabei werden beliebige Werte in ein STree gegeben und anschließend wird
/// `locate_or_succ(x) mit allen x zwischen STree.min() und STree.max() getestet.
#[test]
fn test_u40_locate_or_succ_bruteforce() {
    let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
    let mut data: Vec<u40> = vec![];
    for val in data_v1.iter() {
        data.push(u40::new(*val));
    }

    let data_structure: STree<u40> = STree::new(data.into_boxed_slice());
    for (index,_) in data_v1.iter().enumerate() {
        if index < data_v1.len()-1 {
            for i in data_v1[index]+1..data_v1[index+1]+1 {
                let locate = data_structure.locate_or_succ(u40::new(i)).unwrap();
                assert_eq!(data_structure.element_list[locate], u40::new(data_v1[index+1]));
            }
        }
    }
}

/// Die locate_or_succ-Funktion wird getestet. Dabei werden beliebige Werte in ein STree gegeben und anschließend wird
/// `locate_or_succ(x) mit allen x zwischen STree.min() und STree.max() getestet.
#[test]
fn test_u48_locate_or_succ_bruteforce() {
    let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983,1865983];
    let mut data: Vec<u48> = vec![];
    for val in data_v1.iter() {
        data.push(u48::new(*val));
    }
    
    let data_structure: STree<u48> = STree::new(data.into_boxed_slice());
    for (index,_) in data_v1.iter().enumerate() {
        if index < data_v1.len()-1 {
            for i in data_v1[index]+1..data_v1[index+1]+1 {
                let locate = data_structure.locate_or_succ(u48::new(i)).unwrap();
                assert_eq!(data_structure.element_list[locate], u48::new(data_v1[index+1]));
            }
        }
    }
}

/// # Äquivalenzklassentest mit Bruteforce
/// `locate_or_succ` wird getestet. Dabei werden in jeder Ebene die gesuchten Elemente einmal im Minimum, im Maximum und irgendwo dazwischen liegen.
#[test]
fn test_u40_locate_or_succ_eqc_bruteforce_test() {
    let data_raw: Vec<u64> = vec![
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

    let mut data: Vec<u40> = vec![];
    for val in data_raw.iter() {
        data.push(u40::new(*val));
    }
    let data_structure: STree<u40> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(data_structure.locate_or_succ(u40::new(0b11111111111111111111_1111111111_1111111111_u64)), None);
    
    for (i,&elem) in data.iter().enumerate() {
        if i > 0 {
            for j in 0..16877216 {
                if u64::from(elem)>=j as u64 {
                    let index = elem - u40::new(j);
                    if index > data_structure.element_list[i-1] {
                        assert_eq!(data_structure.element_list[data_structure.locate_or_succ(index).unwrap() as usize], elem);
                    }
                }
            }
        } else {
            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem).unwrap() as usize], elem);
            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem-u40::new(1)).unwrap() as usize], elem);
        }
    }
}

#[test]
fn test_u48_locate_or_succ_eqc_bruteforce_test() {
    let data_raw: Vec<u64> = vec![
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

    let mut data: Vec<u48> = vec![];
    for val in data_raw.iter() {
        data.push(u48::new(*val));
    }
    let data_structure: STree<u48> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(data_structure.locate_or_succ(u48::new(0b111111111111111111111111_111111111111_111111111111_u64)), None);
    
    for (i,&elem) in data.iter().enumerate() {
        if i > 0 {
            for j in 0..16877216 {
                if u64::from(elem)>=j as u64 {
                    let index = elem - u48::new(j);
                    if index > data_structure.element_list[i-1] {
                        assert_eq!(data_structure.element_list[data_structure.locate_or_succ(index).unwrap() as usize], elem);
                    }
                }
            }
        } else {
            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem).unwrap() as usize], elem);
            assert_eq!(data_structure.element_list[data_structure.locate_or_succ(elem-u48::new(1)).unwrap() as usize], elem);
        }
    }
}

/// Die locate_or_pred-Funktion wird getestet. Dabei werden beliebige (fest gewählte) Werte in ein STree gegeben und anschließend wird
/// `locate_or_pred(x) mit allen x zwischen STree.min() und STree.max() getestet.
#[test]
fn test_u40_locate_or_pred_bruteforce() {
    let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
    let mut data: Vec<u40> = vec![];
    for val in data_v1.iter() {
        data.push(u40::new(*val));
    }
    
    let data_structure: STree<u40> = STree::new(data.into_boxed_slice());
    assert_eq!(u40::new(1065983), data_structure.element_list[data_structure.locate_or_pred(u40::new(1065983)).unwrap()]);
    for (index,_) in data_v1.iter().enumerate().rev() {
        if index > 0 {
            for i in (data_v1[index-1]..data_v1[index]).rev() {
                let locate = data_structure.locate_or_pred(u40::new(i)).unwrap();
                assert_eq!(u40::new(data_v1[index-1]), data_structure.element_list[locate]);
            }
        }
    }
}

    /// Die locate_or_pred-Funktion wird getestet. Dabei werden beliebige (fest gewählte) Werte in ein STree gegeben und anschließend wird
/// `locate_or_pred(x) mit allen x zwischen STree.min() und STree.max() getestet.
#[test]
fn test_u48_locate_or_pred_bruteforce() {
    let data_v1: Vec<u64> = vec![0,1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 1065983];
    let mut data: Vec<u48> = vec![];
    for val in data_v1.iter() {
        data.push(u48::new(*val));
    }
    
    let data_structure: STree<u48> = STree::new(data.into_boxed_slice());
    assert_eq!(u48::new(1065983), data_structure.element_list[data_structure.locate_or_pred(u48::new(1065983)).unwrap()]);
    for (index,_) in data_v1.iter().enumerate().rev() {
        if index > 0 {
            for i in (data_v1[index-1]..data_v1[index]).rev() {
                let locate = data_structure.locate_or_pred(u48::new(i)).unwrap();
                assert_eq!(u48::new(data_v1[index-1]), data_structure.element_list[locate]);
            }
        }
    }
}

use num::Bounded;
    /// # Äquivalenzklassentest mit Bruteforce
/// `locate_or_pred` wird getestet. Dabei werden in jeder Ebene die gesuchten Elemente einmal im Minimum, im Maximum und irgendwo dazwischen liegen.
#[test]
fn test_u40_locate_or_pred_eqc_bruteforce_test() {
    let data_raw: Vec<u64> = vec![
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

    let mut data: Vec<u40> = vec![];
    for val in data_raw.iter() {
        data.push(u40::new(*val));
    }
    let data_structure: STree<u40> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(data_structure.locate_or_pred(u40::new(0)), None);

    for (i,&elem) in data.iter().enumerate().rev() {
        if i < data.len()-1 {
            for j in 0..16877216 {
                if u40::max_value() > elem && u40::new(j) < u40::max_value() - elem {
                    let index = elem + u40::new(j);
                    if index < data_structure.element_list[i+1] {
                        assert_eq!(data_structure.element_list[data_structure.locate_or_pred(index).unwrap() as usize], elem);
                    }
                }
            }
        } else {
            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem).unwrap() as usize], elem);
            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem+u40::new(1)).unwrap() as usize], elem);
        }
    }
}

#[test]
fn test_u48_locate_or_pred_eqc_bruteforce_test() {
    let data_raw: Vec<u64> = vec![
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

    let mut data: Vec<u48> = vec![];
    for val in data_raw.iter() {
        data.push(u48::new(*val));
    }
    let data_structure: STree<u48> = STree::new(data.clone().into_boxed_slice());
    assert_eq!(data_structure.locate_or_pred(u48::new(0)), None);

    for (i,&elem) in data.iter().enumerate().rev() {
        if i < data.len()-1 {
            for j in 0..16877216 {
                if u48::max_value() > elem && u48::new(j) < u48::max_value() - elem {
                    let index = elem + u48::new(j);
                    if index < data_structure.element_list[i+1] {
                        assert_eq!(data_structure.element_list[data_structure.locate_or_pred(index).unwrap() as usize], elem);
                    }
                }
            }
        } else {
            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem).unwrap() as usize], elem);
            assert_eq!(data_structure.element_list[data_structure.locate_or_pred(elem+u48::new(1)).unwrap() as usize], elem);
        }
    }
}

use rand_distr::{Distribution, Uniform};
use crate::default::immutable::TopArray;
/*#[test]*/
/// Fügt einige Bits in eine ArrayTop-Struktur und prüft anschließend, ob die Bits gesetted sind.
/// (Deaktiviert, da der Test sehr lange dauert)
fn test_top_array_set_bit() {
    let between = Uniform::from(0u64..(1<<10));
    let mut rng = rand::thread_rng();

    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..230 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();

    let mut lxtop = TopArray::<u40, LXKey>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1<<10) {
        assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
    }

    for i in 0..bits_set.len()-1 {
        assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
    }

    for i in 1..bits_set.len() {
        assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
    }

    let between = Uniform::from(0u64..(1<<12));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..230 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    
    let mut lxtop = TopArray::<u48, LXKey>::new();
    
    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1<<12) {
        assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
    }

    for i in 0..bits_set.len()-1 {
        assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
    }

    for i in 1..bits_set.len() {
        assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
    } 

    let between = Uniform::from(0u64..(1<<16));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    let mut lxtop = TopArray::<u64, LXKey>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1<<16) {
        assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
    }

    for i in 0..bits_set.len()-1 {
        assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
    }

    for i in 1..bits_set.len() {
        assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
    }
    
    let between = Uniform::from(0u64..(1<<20));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    let mut lxtop = TopArray::<u40, usize>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1<<20) {
        assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
    }

    for i in 0..bits_set.len()-1 {
        assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
    }

    for i in 1..bits_set.len() {
        assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
    }

    let between = Uniform::from(0u64..(1<<22));
    let mut bits_set: Vec<usize> = vec![];
    for _ in 0..20000 {
        bits_set.push(between.sample(&mut rng) as usize);
    }
    bits_set.sort();
    bits_set.dedup();
    let mut lxtop = TopArray::<u48, usize>::new();

    for &i in bits_set.iter() {
        lxtop.set_bit(i);
    }

    for i in 0..(1<<22) {
        assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
    }

    for i in 0..bits_set.len()-1 {
        assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
    }

    for i in 1..bits_set.len() {
        assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
    }

    let between = Uniform::from(0u64..(1<<32));
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
    for i in 0..(1<<22) {
        assert_eq!(bits_set.contains(&i),lxtop.is_set(i));
    }

    for i in 0..bits_set.len()-1 {
        assert_eq!(bits_set[i+1],lxtop.get_next_set_bit(bits_set[i]).unwrap());
    }

    for i in 1..bits_set.len() {
        assert_eq!(bits_set[i-1],lxtop.get_prev_set_bit(bits_set[i]).unwrap());
    }
}

#![allow(dead_code)]  
use ux::{u40,u10};
use boomphf::Mphf;
use super::internal::root_size;
pub type Int = u40;
// In der statischen Variante werden die Indizes des Vektors für die Minima und Maxima gespeichert.
pub type SecondLevel = Level<usize>;
pub type FirstLevel = Level<SecondLevel>;

pub struct STree {
    root_table: [FirstLevel; root_size::<Int>()],
    // Da die Größe in in Bytes von size_of zurückgegeben wird, mal 8. Durch 64, da 64 Bits in einen u64 passen.
    root_top: [u64; root_size::<Int>()/64],
    root_top_sub: [u64; root_size::<Int>()/64/64], //Hier nur ein Element, da 2^16/64/64 nur noch 16 Bit sind, die alle in ein u64 passen!
    element_list: Vec<Int>,
}

impl STree {
    // Annahme: items enthält unique i40 Werte in sortierter Reihenfolge!
    pub fn new(items: Vec<Int>) -> STree {
        let mut result = STree{
            root_table: super::internal::PerfectHashBuilder::new(items.clone()).build(),
            root_top: [0; root_size::<Int>()/64],
            root_top_sub: [0; root_size::<Int>()/64/64], 
            element_list: items.clone(),
        };
        for (index,element) in items.iter().enumerate() {
            let (i,j,k) = super::internal::split_integer_down(*element);
            // Dadurch das die Reihenfolge sortiert ist, wird das letzte hinzugefügte Element das größte und das erste das kleinste sein.
            if result.root_table[i].minimum.is_none() {
                result.root_table[i].minimum = Some(index);
            }
            result.root_table[i].maximum = Some(index);
            let first_key = result.root_table[i].hasher.as_ref().unwrap().try_hash(&j).unwrap() as usize;
            let second_key = result.root_table[i].objects[first_key].hasher.as_ref().unwrap().try_hash(&k).unwrap() as usize;
        }
        result
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.element_list.len()
    }

    #[inline]
    pub fn minimum(&self) -> Option<usize> {
        if self.len() == 0 {
            return None;
        }
        Some(0)
    }

    #[inline]
    pub fn maximum(&self) -> Option<usize> {
        if self.len() == 0 {
            return None;
        }
        Some(self.len() - 1)
    }

    pub fn locate(&self) -> Option<usize> {
        unimplemented!();
    }

}

pub struct Level<T> {
    pub hasher: Option<Mphf<u10>>,
    pub objects: Vec<T>,
    pub maximum: Option<usize>,
    pub minimum: Option<usize>,
    pub lx_top: Vec<u64>,
}

impl<T> Level<T> {
    #[inline]
    pub fn new(level: usize, keys: Option<Vec<u10>>) -> Level<T> {
        match keys {
            Some(x) => Level {
                hasher: Some(Mphf::new_parallel(2.0,&x,None)),
                objects: vec![],
                maximum: None,
                minimum: None,
                lx_top: vec![0;level],
            },
            None => Level {
                hasher: None,
                objects: vec![],
                maximum: None,
                minimum: None,
                lx_top: vec![0;level],
            }
        }
    }

    // Die Hashtabelle beinhaltet viele Werte, die abhängig der nächsten 8 Bits der Binärdarstellung der zu lokalisierenden Zahl sind
    // Der lx_top-Vektor hält die Information, ob im Wert 0 bis 2^8 ein Wert steht. Da 64 Bit in einen u64 passen, hat der Vektor nur 4 Einträge mit jeweils 64 Bit (u64)
    #[inline]
    pub fn locate_top_level(&mut self, bit: u10) -> Option<u10> {
        unimplemented!();
    }
}






// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen, 
// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.
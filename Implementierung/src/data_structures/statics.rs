#![allow(dead_code)]  
use ux::{u40,u10};
use boomphf::Mphf;
use crate::help::internal::{Splittable};
use crate::help::builder::PerfectHashBuilder;
pub type Int = u40;
// In der statischen Variante werden die Indizes des Vektors für die Minima und Maxima gespeichert.
pub type SecondLevel = Level<Option<usize>>;
pub type FirstLevel = Level<SecondLevel>;



pub struct STree {
    root_table: Box<[FirstLevel]>,
    // Da die Größe in in Bytes von size_of zurückgegeben wird, mal 8. Durch 64, da 64 Bits in einen u64 passen.
    root_top: Box<[u64; (1<<20)/64]>,
    root_top_sub: Box<[u64; (1<<20)/64/64]>, //Hier nur ein Element, da 2^16/64/64 nur noch 16 Bit sind, die alle in ein u64 passen!
    element_list: Vec<Int>,
}

impl STree {
    // Annahme: items enthält unique u40 Werte in sortierter Reihenfolge!
    /**
     *  Diese Methode verwendet die Builder-Hilfs-Klasse um die perfekten Hashfunktionen zu setzen. Anschließend werden die richtigen
     *  Zeiger für die Werte eingefügt und die Maxima- und Minima-Zeiger werden eingefügt (hier Indizes des Arrays). Zum Schluss
     *  werden die Top-Level-Datenstrukturen angepasst.
     * */ 
    pub fn new(items: Vec<Int>) -> STree {
        let builder = PerfectHashBuilder::new(items.clone());
        let (root_top,root_top_sub) = builder.build_root_top();
        let mut result = STree{
            root_table: builder.build(),
            root_top: root_top,
            root_top_sub: root_top_sub, 
            element_list: items.clone(),
        };
        for (index,element) in items.iter().enumerate() {
            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(element);//super::internal::split_integer_down(*element);
            // Dadurch das die Reihenfolge sortiert ist, wird das letzte hinzugefügte Element das größte und das erste das kleinste sein.

            let root = &mut result.root_table[i];
            root.minimum.get_or_insert(index);
            root.maximum = Some(index);

            let first_key = root.hasher.as_ref().unwrap().hash(&j) as usize;
            let first = &mut root.objects[first_key];

            // Minima- und Maximasetzung auf der ersten Ebene
            first.minimum.get_or_insert(index);
            first.maximum = Some(index);

            let second_key = first.hasher.as_ref().unwrap().hash(&k) as usize;
            // Werte korrekt auf das Array zeigen lassen:Level
            first.objects[second_key] = Some(index);
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
    pub fn locate_top_level(&mut self, _bit: u10) -> Option<u10> {
        unimplemented!();
    }
}






// Gamma=2 wegen Empfehlung aus dem Paper. Wenn Hashen schneller werden soll, dann kann man bis gegen 5 gehen, 
// Wenn die Struktur kleiner werden soll, kann man mal gamme=1 ausprobieren.

#[cfg(test)]
mod tests {
    use ux::{u40,u10};
    use super::STree;
    use crate::help::internal::{Splittable};

    #[test]
    fn test_new_hashfunctions() {

        // Alle u40 Werte sollten nach dem Einfügen da sein, die Hashfunktionen sollten alle dann beim "suchen" funktionieren
        // und alle Top-Level-Datenstrukturen sollten mit 1 belegt sein.
        let mut data: Vec<u40> = vec![u40::new(0);1<<2];
        
        for i in 0..data.len() {
            data[i] = u40::new(i as u64);
        }

        let check = data.clone();
        let data_structure: STree = STree::new(data);

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),0);
        assert_eq!(data_structure.maximum().unwrap(),check.len()-1);
        for val in check {
            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&val);
            let second_level = &data_structure.root_table[i].objects[data_structure.root_table[i].hasher.as_ref().unwrap().hash(&j) as usize];
            let saved_val = second_level.objects[second_level.hasher.as_ref().unwrap().hash(&k) as usize].unwrap();
            assert_eq!(data_structure.element_list[saved_val],val);
        }
    }

    #[test]
    fn test_top_arrays() {
        let data: Vec<u40> = vec![u40::new(0b00000000000000000000_1010010010_0101010101),u40::new(0b00000000000000000000_1010010010_0101010111),u40::new(0b11111111111111111111_1010010010_0101010101)];
        let check = data.clone();
        let data_structure: STree = STree::new(data);

        assert_eq!(data_structure.len(),check.len());
        assert_eq!(data_structure.minimum().unwrap(),0);
        assert_eq!(data_structure.maximum().unwrap(),check.len()-1);

        for val in check {
            let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&val);
            let second_level = &data_structure.root_table[i].objects[data_structure.root_table[i].hasher.as_ref().unwrap().hash(&j) as usize];
            let saved_val = second_level.objects[second_level.hasher.as_ref().unwrap().hash(&k) as usize].unwrap();
            assert_eq!(data_structure.element_list[saved_val],val);
        }
        // Root_TOP
        // 1+61x0 = 9223372036854775808
        assert_eq!(data_structure.root_top[0],9223372036854775808);
        for i in 1..16383 {
            assert_eq!(data_structure.root_top[i],0);
        }
        assert_eq!(data_structure.root_top[16383],1);

        // ROOT_TOP_SUB
        assert_eq!(data_structure.root_top_sub[0], 9223372036854775808);
        for i in 1..255 {
            assert_eq!(data_structure.root_top_sub[i],0);
        }
        assert_eq!(data_structure.root_top_sub[255], 1);
        
    }
}
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

        /** 
     * Gibt den kleinstne Wert j mit element <= j zurück. 
     * Kann verwendet werden, um zu prüfen ob element in der Datenstruktur enthalten ist. 
     * Gibt anderenfalls den Nachfolger zurück, falls dieser existiert.
     */
    #[inline]
    pub fn locate(&self, element: Int) -> Option<usize> {
        let (i,j,k) = Splittable::<usize,u10>::split_integer_down(&element);

        // Paper z.1 
        if self.len() < 1 || element > self.element_list[self.maximum().unwrap()] {
            return None;
        } 

        // Paper z. 3 
        if self.root_table[i].maximum.is_none() || self.element_list[self.root_table[i].maximum.unwrap()] < element {
            return self.locate_top_level(u40::new(i as u64 + 1),0)
                .map(|x| self.root_table[u64::from(x) as usize].minimum.unwrap());
        }
       
        // Paper z. 4
        if self.root_table[i].maximum == self.root_table[i].minimum {
            return Some(self.root_table[i].minimum.unwrap());
        }

        // Paper z. 6
        if self.root_table[i].get(&j).is_none() || self.element_list[self.root_table[i].get(&j).unwrap().maximum.unwrap()] < element {
           
            let new_j = self.root_table[i].locate_top_level(&(j+u10::new(1)));
            return new_j
                .and_then(|x| self.root_table[i].get(&(x)))
                .map(|x| x.minimum.unwrap());
        }
    

        // Paper z.7
        if self.root_table[i].get(&j).unwrap().maximum == self.root_table[i].get(&j).unwrap().minimum {
            return Some(self.root_table[i].get(&j).unwrap().minimum.unwrap());
        }

        // Paper z.8
        let new_k = self.root_table[i].get(&j).unwrap().locate_top_level(&k);
        return new_k
            .map(|x| self.root_table[i].get(&j).unwrap().get(&x).unwrap().unwrap());

    }

     /**
     * Gibt das kleinste j zurück, so dass element <= j und k_level[j]=1
     * Hierbei beachten, dass j zwar Bitweise adressiert wird, die Level-Arrays allerdings ganze 64-Bit-Blöcke besitzen. Somit ist z.B: root_top[5] nicht das 6. 
     * Bit sondern, der 6. 64-Bit-Block. Die Methode gibt aber die Bit-Position zurück!
     */ 

    //TODO: Fix that Shit
    pub fn locate_top_level(&self, bit: Int, level: u8) -> Option<Int> {
        let bit = u64::from(bit);
        let index = bit as usize/64;
        let in_index = bit%64;
        // Da der Index von links nach rechts gezählt wird, aber 2^i mit i=index von rechts nach Links gilt, muss 64-in_index gerechnet werden.
        // Diese Bit_Maske dient dem Nullen der Zahlen hinter in_index
        let bit_mask: u64 = u64::max_value() >> in_index; // genau falschherum
        // Siehe Paper, irgendwo muss noch Fill Zeros implementiert werden
        
        if level != 0 {
            for i in index..self.root_top_sub.len() {
                if self.root_top_sub[i] != 0 {
                    let nulls = self.root_top_sub[i].leading_zeros();
                    return Some(u40::new(i as u64 + nulls as u64));
                }
            }
            return None;
        }
        
        let nulls = (self.root_top[index] & bit_mask).leading_zeros();
        
        // Leading Zeros von root_top[index] bestimmen und mit in_index vergleichen. Die erste führende 1 muss rechts von in_index liegen oder an Position in_index.
        if nulls != 64 {
            return Some(u40::new(index as u64 *64+nulls as u64));
        }
        
        // Wenn Leading Zeros=64, dann locate_top_level(element,level+1)
        let new_index = self.locate_top_level(u40::new(index as u64 +1) ,level+1);

        new_index.and_then(|x|
            match self.root_top[u64::from(x) as usize *64].leading_zeros() {
                64 => None,
                val => Some(u40::new(u64::from(x)*64 + val as u64))
            }
        )
        
    }

}

pub struct Level<T> {
    pub hasher: Option<Mphf<u10>>,
    pub keys: Vec<u10>,
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
                keys: x,
                objects: vec![],
                maximum: None,
                minimum: None,
                lx_top: vec![0;level],
            },
            None => Level {
                hasher: None,
                objects: vec![],
                keys: vec![],
                maximum: None,
                minimum: None,
                lx_top: vec![0;level],
            }
        }
    }

    /**
     * Diese Funktion dient der Abfrage eines Wertes aus der Hashtabelle des Levels
     * 
     */
    #[inline]
    pub fn get(&self, key: &u10) -> Option<&T> {
        let hash = self.hasher.as_ref().unwrap().try_hash(&key)? as usize;
        if self.keys.contains(key) {
            self.objects.get(hash)
        } else {
            None
        }
    }

    // Die Hashtabelle beinhaltet viele Werte, die abhängig der nächsten 10 Bits der Binärdarstellung der zu lokalisierenden Zahl sind
    // Der lx_top-Vektor hält die Information, ob im Wert 0 bis 2^10 ein Wert steht. Da 64 Bit in einen u64 passen, hat der Vektor nur 4 Einträge mit jeweils 64 Bit (u64)
    #[inline]
    pub fn locate_top_level(&self, bit: &u10) -> Option<u10> {
        let bit = u16::from(*bit);
        let index = bit as usize/64;

        if self.lx_top[index] != 0 {
            let in_index = bit%64;
            let bit_mask: u64 = u64::max_value() >> in_index;
            let num_zeroes = (self.lx_top[index] & bit_mask).leading_zeros();

            if num_zeroes != 64 {
                return Some(u10::new(index as u16 *64 + num_zeroes as u16));
            }
        }
        for i in index+1..self.lx_top.len() {
            let val = self.lx_top[i];
            if val != 0 {
                let num_zeroes = val.leading_zeros();
                return Some(u10::new(i as u16 *64 + num_zeroes as u16));
            }
        }
        None
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
        let mut data: Vec<u40> = vec![u40::new(0);1<<10];
        
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

    #[test]
    fn test_locate() {
        
        let data_v1: Vec<u64> = vec![1,3,23,123,232,500,20000, 30000, 50000, 100000, 200000, 200005, 200005200005];
        let mut data: Vec<u40> = vec![];
        for val in data_v1.iter() {
            data.push(u40::new(*val));
        }
        
        let data_structure: STree = STree::new(data);
        println!("Max: {}", data_structure.element_list[data_structure.root_table[0].maximum.unwrap()]);
        for (index,_) in data_v1.iter().enumerate() {
            if index < data_v1.len()-1 {
                for i in data_v1[index]+1..data_v1[index+1]+1 {
                   // println!("Index: {}", i);
                    let locate = data_structure.locate(u40::new(i)).unwrap();
                    assert_eq!(data_structure.element_list[locate], u40::new(data_v1[index+1]));
                }
            }
        }
        
 
        
    }
}
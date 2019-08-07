use std::time::{Instant};
fn main() {
    println!("Starte initialisierung");
    let start_init = Instant::now();

    let size = (u32::max_value() as usize) + (10000 as usize);
    println!("Size {}",size);
    let mut tmp: Vec<u64> = vec![0;size];
    
    println!("Init dauer: {}", start_init.elapsed().as_secs());
    println!("Starte einfügen");
    let start_loop = Instant::now();
    for i in 0..u32::max_value() {
        tmp[i as usize] = i as u64;
    }

    test(tmp);
    let l2 = vec![2 as u64];
    println!("Ende {} Sekunden sind vergangen und der letzte Wert vom Vektor ist: {}",start_loop.elapsed().as_secs(),l2[(u32::max_value() - 1)  as usize]);
    println!("Max-Int: {}", u32::max_value());
}

fn test(irgendwa: Vec<u64>) {

}

use std::time::{Instant};
fn main() {
    println!("Starte initialisierung");
    let start_init = Instant::now();
    let mut tmp = vec![0;u32::max_value() as usize];
    
    println!("Init dauer: {}", start_init.elapsed().as_secs());
    println!("Starte einfügen");
    let start_loop = Instant::now();
    for i in 0..u32::max_value() {
        tmp[i as usize] = i;
    }
    println!("Ende {} Sekunden sind vergangen und der letzte Wert vom Vektor ist: {}",start_loop.elapsed().as_secs(),tmp[(u32::max_value() - 1)  as usize]);
    println!("Max-Int: {}", u32::max_value());
}

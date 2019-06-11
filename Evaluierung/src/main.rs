use std::mem;
use std::collections::HashMap;

fn main() {
    println!("Hello, world! {}",mem::size_of::<Option<HashMap<u8,HashMap<u8,u32>>>>());
}

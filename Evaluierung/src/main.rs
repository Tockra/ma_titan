#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
fn main() {
    unsafe {
        let all_bytes_zero = _mm256_setzero_si256();
        println!("{:?}", all_bytes_zero);
    }
}

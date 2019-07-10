pub type Int = i32;

#[test]
pub fn locate() {
    test_locate_root_top();
}

/**
 * Zum Testen der locate_top-Methoden werden hier einige u64 Arrays testweise angelegt. Dabei gibt es immer Varianten a,b und c:unimplemented!
 * Jede Variante wird am Anfang vom untersuchten Array [0], im Untersuchten Array [0<x<len] und am Ende des Arrays [len-1] eingefügt.
 * a: Letztes Bit ist gesetzt
 * b: erstes Bit und "andere" sind gesetzt
 * c: mitten im u64 ist ein Bit gesetzt
 */
#[inline]
pub fn test_locate_root_top() {
    // root_top mit Inhalt in [0]
    let mut root_top_sub_start: [u64] = [0; root_size::<Int>()/64/64];
    root_top_sub_start[0] = 0b10000000000000000000000000000000_00000000000000000000000000000000;

    // locate(i) mit i<= 63 sollte 63 zurückgeben.
    let mut root_top_start_a: [u64] = [0; root_size::<Int>()/64];
    root_top_start[0] = 1;

    // locate(i) mit i=0 sollte 0 zurückgeben.
    let mut root_top_start_b: [u64] = [0; root_size::<Int>()/64];
    root_top_start[0] = 0b10000000000000000000000000000000_10000000000000000000000000000000;

    // locate(i) mit i<= 32 sollte 32 zurückgeben.
    let mut root_top_start_c: [u64] = [0; root_size::<Int>()/64];
    root_top_start[0] = 0b00000000000000000000000000000000_10000000000000000000000000000000;

    // root_top mit Inhalt in der Mitte[512]
    let mut root_top_sub_middle: [u64] = [0; root_size::<Int>()/64/64];
    root_top_sub_middle[8] = 0b10000000000000000000000000000000_00000000000000000000000000000000;
    // locate(i) mit i<= 32831 sollte 32831 zurückgeben.
    let mut root_top_middle_a: [u64] = [0; root_size::<Int>()/64];
    root_top_middle_a[root_size::<Int>()/64/2] = 1;

    // locate(i) mit i<=32768 sollte 768 zurückgeben.
    let mut root_top_middle_b: [u64] = [0; root_size::<Int>()/64];
    root_top_middle_b[root_size::<Int>()/64/2] = 0b10000000000000000000000000000000_10000000000000000000000000000000;

    // locate(i) mit i<= 32.800‬ sollte 32.800‬ zurückgeben.
    let mut root_top_middle_c: [u64] = [0; root_size::<Int>()/64];
    root_top_middle_c[root_size::<Int>()/64/2] = 0b00000000000000000000000000000000_10000000000000000000000000000000;

    
    // root_top mit Inhalt am Ende [1023]
    let mut root_top_sub_end: [u64] = [0; root_size::<Int>()/64/64];
    root_top_sub_end[15] = 1;
    // locate(i) mit i<= 65.535 sollte 65.535 zurückgeben.
    let mut root_top_end_a: [u64] = [0; root_size::<Int>()/64];
    root_top_end_c[root_size::<Int>()/64 -1 ] = 1;

    // locate(i) mit i=65.472 sollte 65.472 zurückgeben.
    let mut root_top_end_b: [u64] = [0; root_size::<Int>()/64];
    root_top_end_c[root_size::<Int>()/64/2 - 1] = 0b10000000000000000000000000000000_10000000000000000000000000000000;

    // locate(i) mit i<= 65.504 sollte 65.504 zurückgeben.
    let mut root_top_end_c: [u64] = [0; root_size::<Int>()/64];
    root_top_end_c[root_size::<Int>()/64/2 - 1] = 0b00000000000000000000000000000000_10000000000000000000000000000000;

    

    
}

const fn root_size<T>() -> usize {
    1 << 8*mem::size_of::<T>() / 2
}

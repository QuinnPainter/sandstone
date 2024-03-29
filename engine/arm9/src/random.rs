use ironds::sync::NdsMutex;
use randomize::{PCG32, Gen32};

// could improve this by making a version of LazyStatic / LazyCell for NdsMutex / NdsCell
static RAND_GENERATOR: NdsMutex<PCG32> = NdsMutex::new(PCG32::seed(0, 0));

// Taken from randomize source
const DEFAULT_PCG_INC: u128 = 34172814569070222299;

pub fn seed(seed: u64) {
    *RAND_GENERATOR.lock() = PCG32::seed(seed, DEFAULT_PCG_INC as u64);
}

pub fn rand_u32() -> u32 {
    RAND_GENERATOR.lock().next_u32()
}

pub fn rand_i32_in_range(lower: i32, upper: i32) -> i32 {
    assert!(upper >= lower);
    let range = upper - lower;
    let rand = RAND_GENERATOR.lock().next_bounded(range as u32);
    rand as i32 + lower
}


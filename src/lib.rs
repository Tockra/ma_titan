pub mod internal;
pub mod default;


use stats_alloc::{StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
pub static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;
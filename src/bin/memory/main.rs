use runner::benches::rank;
use std::alloc::System;
use stats_alloc::StatsAlloc;
use stats_alloc_helper::{memory_measured, LockedAllocator};
use vers_vecs::{BitVec, RsVec};
use runner::benches::rank::BitVecState;

#[global_allocator]
static GLOBAL: LockedAllocator<System> = LockedAllocator::new(StatsAlloc::system());

fn main() {
    let mut rank = rank::create_benchmark();

    let sizes = rank.sizes().collect::<Vec<_>>();
    for size in sizes {
        for runner in rank.runners() {
            runner.initialize_measurement(size);
            // initialized with dummy to keep the compiler happy
            let mut state: BitVecState = BitVecState::Vers(RsVec::from_bit_vec(BitVec::from_bits(&[])));

            let stats = memory_measured(&GLOBAL, || {
                // leak the state out of the closure to prevent deallocation
                state = runner.create_state();
            });
            println!("{} allocated {} and deallocated {}", runner.name(), stats.bytes_allocated, stats.bytes_deallocated);
        }
    }
}
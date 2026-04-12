use runner::benches::rank;
use std::alloc::System;
use criterion::black_box;
use stats_alloc::StatsAlloc;
use stats_alloc_helper::{memory_measured, LockedAllocator};
use vers_vecs::{BitVec, RsVec};
use runner::benches::rank::BitVecState;

#[global_allocator]
static GLOBAL: LockedAllocator<System> = LockedAllocator::new(StatsAlloc::system());

fn main() {
    let mut rank = rank::create_benchmark();

    let sizes = rank.sizes().collect::<Vec<_>>();

    // the initial allocation counter has some noise that only appears in the first measurement, so
    // get rid of initial noise in the allocator by doing one measurement and throwing it away.
    // We count how much memory is deallocated by overwriting the dummy object, so we can
    // correct the deallocation counters of later measurements.
    let mut state: BitVecState = BitVecState::Vers(RsVec::from_bit_vec(BitVec::from_bits(&[])));

    let stats = memory_measured(&GLOBAL, || {
        // leak the state out of the closure to prevent deallocation
        state = rank.runners().nth(0).unwrap().create_state();
    });
    let dummy_deallocation = stats.bytes_deallocated;

    for size in sizes {
        for runner in rank.runners() {
            runner.initialize_measurement(size);
            // initialized with dummy to keep the compiler happy
            let mut state: BitVecState = BitVecState::Vers(RsVec::from_bit_vec(BitVec::from_bits(&[])));

            // perform measurement
            let stats = memory_measured(&GLOBAL, || {
                // leak the state out of the closure to prevent deallocation
                state = runner.create_state();
            });

            let deallocated = stats.bytes_deallocated - dummy_deallocation;
            let memory_requirement = stats.bytes_allocated - deallocated;

            println!("{} requires {} bytes and {} bytes of scratch space", runner.name(), memory_requirement, deallocated);
            black_box(state);
        }

        println!()
    }
}
use crate::benchmark::Benchmark;
use crate::measure::Measurement;
use crate::runner;
use rand::Rng;
use rsdict::RsDict;
use std::hint::black_box;
use vers_vecs::{BitVec, RsVec};
use BitVecState::*;

pub(crate) enum BitVecState {
    Vers(RsVec),
    RsD(RsDict),
}

runner!(VersRunner,
    create_context = |size| { Vers(RsVec::from(BitVec::from_zeros(size))) },

    prepare_params = |number, len| {
        let mut rng = rand::thread_rng();
        let mut vec = Vec::with_capacity(number);
        for _ in 0..number {
            vec.push(rng.gen_range(0..len as u64));
        }
        vec.into_boxed_slice()
    },

    execute = |rs: BitVecState, idx: u64| {
        if let Vers(rs) = rs {
            black_box(rs.rank0(*idx as usize));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(RsDictRunner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let mut rs_dict = RsDict::with_capacity(size);
        for _ in 0..size {
            rs_dict.push(rng.gen_bool(0.5));
        }
        RsD(rs_dict)
    },

    prepare_params = |number, len| {
        let mut rng = rand::thread_rng();
        let mut vec = Vec::with_capacity(number);
        for _ in 0..number {
            vec.push(rng.gen_range(0..len as u64));
        }
        vec.into_boxed_slice()
    },

    execute = |rs: BitVecState, idx: u64| {
        if let RsD(rs) = rs {
            black_box(rs.rank(*idx, false));
        } else {
            panic!("Invalid state");
        }
    }
);

pub(crate) fn benchmark() {
    let mut benchmark = Benchmark::<BitVecState, u64>::new("Rank");
    benchmark.add_measurement(Measurement::new("Vers", &VersRunner));
    benchmark.add_measurement(Measurement::new("RsDict", &RsDictRunner));
    benchmark.benchmark();
}

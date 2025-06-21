use crate::benchmark::Benchmark;
use crate::measure::Measurement;
use crate::runner;
use rand::{Rng, RngCore};
use rsdict::RsDict;
use std::hint::black_box;
use std::ops::{Div, Rem};
use std::path::Path;
use vers_vecs::{BitVec, RsVec};
use BitVecState::*;

pub(crate) enum BitVecState {
    Vers(RsVec),
    RsD(RsDict),
}

runner!(
    VersRunner,
    create_context = |size| {
        let mut bitvec = BitVec::with_capacity(size);
        let full_limbs = size.div(64);
        let mut rng = rand::thread_rng();
        for _ in 0..full_limbs {
            bitvec.append_word(rng.gen());
        }

        let remainder = size.rem(64);
        if remainder > 0 {
            bitvec.append_bit(rng.next_u64() % 2)
        }

        Vers(bitvec.into())
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
        if let Vers(rs) = rs {
            black_box(rs.rank0(*idx as usize));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    RsDictRunner,
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

pub(crate) fn benchmark(output_dir: &Path) {
    let mut benchmark = Benchmark::<BitVecState, u64>::new(
        "Rank",
        vec![
            1 << 7,
            1 << 8,
            1 << 9,
            1 << 10,
            1 << 11,
            1 << 12,
            1 << 13,
            1 << 14,
            1 << 15,
            1 << 16,
            1 << 17,
            1 << 18,
            1 << 19,
            1 << 20,
            1 << 21,
            1 << 22,
            1 << 23,
            1 << 24,
            1 << 25,
            1 << 26,
            1 << 27,
            1 << 28,
            1 << 29,
            1 << 30,
            1 << 31,
            1 << 32,
        ],
    );
    benchmark.add_measurement(Measurement::new("Vers", &VersRunner));
    benchmark.add_measurement(Measurement::new("RsDict", &RsDictRunner));
    benchmark.benchmark(output_dir);
}

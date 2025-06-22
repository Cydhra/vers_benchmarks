use crate::benchmark::Benchmark;
use crate::measure::Measurement;
use crate::runner;
use bio::data_structures::rank_select::RankSelect as BioRsVec;
use bitm::{BitAccess, BitVec as BitmBV, CombinedSampling, RankSelect101111 as BitmVec, Select0};
use bv::BitVec as BVBitVec;
use fid::{BitVector as FidVec, FID};
use indexed_bitvec::IndexedBits as IndexedVec;
use rand::distributions::Uniform;
use rand::Rng;
use rsdict::RsDict;
use std::hint::black_box;
use std::ops::Div;
use std::path::Path;
use sucds::bit_vectors::darray::DArray as SucDsDVec;
use sucds::bit_vectors::rank9sel::Rank9Sel as SucDsR9Vec;
use sucds::bit_vectors::{BitVector as SucBitVec, Select};
use sux::prelude::{BitVec as SuxVec, SelectZeroAdapt as SuxR9Select, SelectZero as SuxSelect, AddNumBits, Block32Counters, RankSmall as SuxSmallVec};
use sux::rank_small;
use vers_vecs::{BitVec, RsVec};
use BitVecState::*;

pub(crate) enum BitVecState {
    Vers(RsVec),
    RsD(RsDict),
    Bio(BioRsVec),
    Fid(FidVec),
    IndexedBV(IndexedVec<Vec<u8>>),
    SucDsR9(SucDsR9Vec),
    SucDsDA(SucDsDVec),
    Bitm(BitmVec<CombinedSampling, CombinedSampling>),
    SuxR9(SuxR9Select<AddNumBits<SuxVec>>),
    SuxSmall(SuxR9Select<AddNumBits<SuxSmallVec<1, 10, sux::bits::BitVec, Box<[usize]>, Box<[Block32Counters<1, 10>]>>>>),
}

fn create_params(number: usize, len: usize) -> Box<[u64]> {
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(number);
    for _ in 0..number {
        // TODO we need to ensure we don't generate invalid requests, as some libraries crash,
        //  but dividing by 3 is very crude
        vec.push(rng.gen_range(0..len as u64 / 3));
    }
    vec.into_boxed_slice()
}

runner!(
    VersRunner,
    create_context = |size| {
        let mut bitvec = BitVec::with_capacity(size);
        let sample = Uniform::new(0, u64::MAX);
        let full_limbs = size.div(64);
        let mut rng = rand::thread_rng();
        for _ in 0..full_limbs {
            bitvec.append_word(rng.sample(sample));
        }

        Vers(bitvec.into())
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let Vers(bv) = bv {
            black_box(bv.select0(*idx as usize));
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
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let RsD(bv) = bv {
            black_box(bv.select0(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    BioRunner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let mut bio_vec = BVBitVec::new_fill(false, size as u64);
        for i in 0..size {
            bio_vec.set(i as u64, rng.gen_bool(0.5));
        }
        // k chosen to be a fair comparison to vers. We can also adapt it with growing size
        // as it is intended, but the library is so slow, it doesn't matter in any case.
        Bio(BioRsVec::new(bio_vec, 512 / 32))
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let Bio(bv) = bv {
            black_box(bv.select_0(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    FidRunner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let mut fid_vec = FidVec::new();
        for _ in 0..size {
            fid_vec.push(rng.gen_bool(0.5));
        }
        Fid(fid_vec)
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let Fid(bv) = bv {
            black_box(bv.select0(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    IndexedBitVecRunner,
    create_context = |size| {
        let rng = rand::thread_rng();
        let sample = Uniform::new(0, u8::MAX);
        let vec = rng.sample_iter(sample).take(size / 8).collect::<Vec<u8>>();
        IndexedBV(IndexedVec::build_from_bytes(vec, size as u64).unwrap())
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let IndexedBV(bv) = bv {
            black_box(bv.select_zeros(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    SucDsR9Runner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let sample = Uniform::new(0, u64::MAX);
        let mut suc_bv = SucBitVec::with_capacity(size);
        for _ in 0..size.div(64) {
            suc_bv
                .push_bits(rng.sample(sample) as usize, 64)
                .expect("Failed to push bits into sucds bitvector");
        }

        SucDsR9(SucDsR9Vec::new(suc_bv).select0_hints())
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let SucDsR9(bv) = bv {
            black_box(bv.select0(*idx as usize));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    SucDsDARunner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let sample = Uniform::new(0, u64::MAX);
        let mut suc_bv = SucBitVec::with_capacity(size);
        for _ in 0..size.div(64) {
            suc_bv
                .push_bits(rng.sample(sample) as usize, 64)
                .expect("Failed to push bits into sucds bitvector");
        }

        SucDsDA(SucDsDVec::from_bits(suc_bv.iter()).enable_rank().enable_select0())
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let SucDsDA(bv) = bv {
            black_box(bv.select0(*idx as usize));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    BitmRunner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let mut bv = Box::<[u64]>::with_zeroed_bits(size);
        for i in 0..size {
            if rng.gen_bool(0.5) {
                bv.set_bit(i);
            }
        }

        Bitm(bv.into())
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let Bitm(bv) = bv {
            black_box(bv.select0(*idx as usize));
        }
    }
);

runner!(
    SuxR9Runner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let mut bit_vec = SuxVec::new(size);
        for i in 0..size {
            if rng.gen_bool(0.5) {
                bit_vec.set(i, true)
            }
        }

        SuxR9(SuxR9Select::new(bit_vec.into(), 4))
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let SuxR9(bv) = bv {
            black_box(bv.select_zero(*idx as usize));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    SuxSmallRunner,
    create_context = |size| {
        let mut rng = rand::thread_rng();
        let mut bit_vec = sux::prelude::BitVec::new(size);
        for i in 0..size {
            if rng.gen_bool(0.5) {
                bit_vec.set(i, true)
            }
        }

        let rank_support = rank_small![2; bit_vec];

        SuxSmall(SuxR9Select::new(rank_support.into(), 4))
    },
    prepare_params = |number, len| {
        create_params(number, len)
    },
    execute = |bv: BitVecState, idx: u64| {
        if let SuxSmall(bv) = bv {
            black_box(bv.select_zero(*idx as usize));
        } else {
            panic!("Invalid state");
        }
    }
);

pub(crate) fn benchmark(output_dir: &Path) {
    let mut benchmark = Benchmark::<BitVecState, u64>::new(
        "Select",
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
    benchmark.add_measurement(Measurement::new("Bio", &BioRunner));
    benchmark.add_measurement(Measurement::new("FID", &FidRunner));
    benchmark.add_measurement(Measurement::new("IBV", &IndexedBitVecRunner));
    benchmark.add_measurement(Measurement::new("SDSR9", &SucDsR9Runner));
    benchmark.add_measurement(Measurement::new("SDSDA", &SucDsDARunner));
    benchmark.add_measurement(Measurement::new("Bitm", &BitmRunner));
    benchmark.add_measurement(Measurement::new("SuxR9", &SuxR9Runner));
    benchmark.add_measurement(Measurement::new("SuxSmall", &SuxSmallRunner));
    benchmark.benchmark(output_dir);
}

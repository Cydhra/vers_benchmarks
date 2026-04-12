use crate::benchmark::Benchmark;
use crate::measure::Measurement;
use crate::runner;
use cseq::elias_fano::Sequence as CseqEliasSeq;
use elias_fano::EliasFano as EfEliasSeq;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::hint::black_box;
use std::path::Path;
use sucds::mii_sequences::{EliasFano as SDSEliasSeq, EliasFanoBuilder as SDSEliasBuilder};
use sux::bits::BitVec;
use sux::dict::{EliasFano, EliasFanoBuilder as SuxEliasBuilder};
use sux::rank_sel::SelectAdaptConst;
use sux::traits::IndexedSeq;
use sux::traits::TryIntoUnaligned;
use sux::traits::Unaligned;

type SuxEliasSeq = Unaligned<EliasFano<usize, SelectAdaptConst<BitVec<Box<[usize]>>>>>;
use vers_vecs::EliasFanoVec;
use EfState::*;

const UNIVERSE_SIZE: u64 = 4_000_000_000u64;

pub enum EfState {
    Vers(EliasFanoVec),
    EliasFano(RefCell<EfEliasSeq>),
    CseqEliasFano(CseqEliasSeq),
    SDSEliasFano(SDSEliasSeq),
    SuxEliasFano(SuxEliasSeq),
}

fn generate_random_sequence(size: usize) -> Vec<u64> {
    let mut sequence = thread_rng()
        .sample_iter(Uniform::new(0u64, UNIVERSE_SIZE))
        .take(size)
        .collect::<Vec<u64>>();
    sequence.sort_unstable();
    sequence
}

fn create_params(number: usize, len: usize) -> Box<[usize]> {
    let sample = Uniform::new(0, len);
    thread_rng().sample_iter(sample).take(number).collect()
}

runner!(
    VersRunner,
    create_context = |size| {
        let sequence = generate_random_sequence(size);
        Vers(EliasFanoVec::from_slice(&sequence))
    },
    prepare_params = |number, len| { create_params(number, len) },
    execute = |ef: EfState, idx: usize| {
        if let Vers(ef) = ef {
            black_box(ef.get_unchecked(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    SDSRunner,
    create_context = |size| {
        let sequence = generate_random_sequence(size);
        let universe = *sequence.last().unwrap() + 1;
        let mut builder = SDSEliasBuilder::new(universe as usize, sequence.len()).unwrap();
        builder
            .extend(sequence.into_iter().map(|i| i as usize))
            .unwrap();
        SDSEliasFano(builder.build())
    },
    prepare_params = |number, len| { create_params(number, len) },
    execute = |ef: EfState, idx: usize| {
        if let SDSEliasFano(ef) = ef {
            black_box(ef.iter(*idx).next().unwrap());
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    SuxRunner,
    create_context = |size| {
        let sequence = generate_random_sequence(size);
        let universe = *sequence.last().unwrap() + 1;
        let mut builder = SuxEliasBuilder::new(sequence.len() as usize, universe as usize);
        builder.extend(sequence.into_iter().map(|e| e as usize));
        SuxEliasFano(builder.build_with_seq().try_into_unaligned().unwrap())
    },
    prepare_params = |number, len| { create_params(number, len) },
    execute = |ef: EfState, idx: usize| {
        if let SuxEliasFano(ef) = ef {
            black_box(ef.get(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    CseqRunner,
    create_context = |size| {
        let sequence = generate_random_sequence(size);
        CseqEliasFano(CseqEliasSeq::with_items_from_slice(&sequence))
    },
    prepare_params = |number, len| { create_params(number, len) },
    execute = |ef: EfState, idx: usize| {
        if let CseqEliasFano(ef) = ef {
            black_box(ef.get(*idx));
        } else {
            panic!("Invalid state");
        }
    }
);

runner!(
    EliasFanoRunner,
    create_context = |size| {
        let sequence = generate_random_sequence(size);
        let universe = *sequence.last().unwrap() + 1;
        let mut elias_seq = EfEliasSeq::new(universe, sequence.len() as u64);
        elias_seq.compress(sequence.iter());
        EliasFano(RefCell::new(elias_seq))
    },
    prepare_params = |number, len| { create_params(number, len) },
    execute = |ef: EfState, idx: usize| {
        if let EliasFano(ef) = ef {
            black_box(ef.try_borrow_mut().unwrap().visit(*idx as u64).unwrap());
        } else {
            panic!("Invalid state");
        }
    }
);

pub fn create_benchmark() -> Benchmark::<'static, EfState, usize> {
    let mut benchmark = Benchmark::<EfState, _>::from_powers("Elias Fano Random Access", 7..24);
    benchmark.add_measurement(Measurement::new("Vers", &VersRunner));
    benchmark.add_measurement(Measurement::new("SDS", &SDSRunner));
    benchmark.add_measurement(Measurement::new("sux", &SuxRunner));
    benchmark.add_measurement(Measurement::new("cseq", &CseqRunner));
    benchmark.add_measurement(Measurement::new("elias-fano", &EliasFanoRunner));

    benchmark
}

pub fn benchmark(output_dir: &Path) {
    let mut benchmark = create_benchmark();

    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_dir.join("elias_fano_access.tsv"))
        .expect("failed to create output file for elias fano");

    benchmark.benchmark(&mut output);
}

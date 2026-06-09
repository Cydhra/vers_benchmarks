use crate::benches::elias_fano::generate_random_sequence;
use crate::benchmark::Benchmark;
use crate::measure::Measurement;
use crate::runner;
use cseq::elias_fano::Sequence as CseqEliasSeq;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::fs::OpenOptions;
use std::hint::black_box;
use std::path::Path;
use sucds::mii_sequences::{EliasFano as SDSEliasSeq, EliasFanoBuilder as SDSEliasBuilder};
use sux::dict::{elias_fano::EfSeqDict as SuxEliasSeq, EliasFanoBuilder as SuxEliasBuilder};
use sux::traits::Succ;
use vers_vecs::EliasFanoVec;
use EfState::*;

const UNIVERSE_SIZE: u64 = 4_000_000_000u64;

pub(crate) enum EfState {
    Vers(EliasFanoVec),
    CseqEliasFano(CseqEliasSeq),
    SDSEliasFano(SDSEliasSeq),
    SuxEliasFano(SuxEliasSeq),
}

fn create_params(number: usize) -> Box<[u64]> {
    let sample = Uniform::new(0, UNIVERSE_SIZE);
    thread_rng().sample_iter(sample).take(number).collect()
}

runner!(
    VersRunner,
    create_context = |size| {
        let sequence = generate_random_sequence(size);
        Vers(EliasFanoVec::from_slice(&sequence))
    },
    prepare_params = |number, _len| { create_params(number) },
    execute = |ef: EfState, value: u64| {
        if let Vers(ef) = ef {
            black_box(ef.successor(*value));
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
        SDSEliasFano(builder.build().enable_rank())
    },
    prepare_params = |number, _len| { create_params(number) },
    execute = |ef: EfState, val: u64| {
        if let SDSEliasFano(ef) = ef {
            black_box(ef.successor(*val as usize));
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
        SuxEliasFano(builder.build_with_seq_and_dict())
    },
    prepare_params = |number, _len| { create_params(number) },
    execute = |ef: EfState, val: u64| {
        if let SuxEliasFano(ef) = ef {
            black_box(ef.succ(*val as usize));
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
    prepare_params = |number, _len| { create_params(number) },
    execute = |ef: EfState, val: u64| {
        if let CseqEliasFano(ef) = ef {
            black_box(ef.geq_cursor(*val));
        } else {
            panic!("Invalid state");
        }
    }
);

pub fn benchmark(output_dir: &Path) {
    let mut benchmark = Benchmark::<EfState, _>::from_powers("Elias Fano Successor", 7..24);
    benchmark.add_measurement(Measurement::new("Vers", &VersRunner));
    benchmark.add_measurement(Measurement::new("SDS", &SDSRunner));
    benchmark.add_measurement(Measurement::new("sux", &SuxRunner));
    benchmark.add_measurement(Measurement::new("cseq", &CseqRunner));

    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_dir.join("elias_fano_successor.tsv"))
        .expect("failed to create output file for elias fano successor");

    benchmark.benchmark(&mut output);
}

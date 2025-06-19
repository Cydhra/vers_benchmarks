use bio::data_structures::rank_select::RankSelect as BioRsVec;
use bitm::{BitAccess, BitVec, CombinedSampling, Rank as BitmRank, RankSelect101111, Select0};
use bv::BitVec as BioVec;
use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use fid::{BitVector as FidVec, FID};
use indexed_bitvec::IndexedBits;
use rand::distributions::{Distribution, Standard, Uniform};
use rand::rngs::ThreadRng;
use rand::Rng;
use rsdict::RsDict;
use succinct::select::Select0Support;
use succinct::{BinSearchSelect, BitRankSupport, BitVecPush, BitVector as SuccinctVec, Rank9};
use sucds::bit_vectors::darray::DArray as SucDArray;
use sucds::bit_vectors::rank9sel::Rank9Sel as SucRank9Vec;
use sucds::bit_vectors::{BitVector as SucBitVec, Rank, Select};
use sux::prelude::{Block32Counters, Rank9 as Sux9, RankSmall, RankZero};
use sux::rank_small;

mod common;

fn construct_rsdict_vec(rng: &mut ThreadRng, len: usize) -> RsDict {
    let mut rs_dict = RsDict::with_capacity(len);
    for _ in 0..len {
        rs_dict.push(rng.gen_bool(0.5));
    }
    rs_dict
}

fn construct_bio_vec(rng: &mut ThreadRng, len: usize) -> BioRsVec {
    let mut bio_vec = BioVec::new_fill(false, len as u64);
    for i in 0..len {
        bio_vec.set(i as u64, rng.gen_bool(0.5));
    }
    // k chosen to be succinct after Cray's definition as outlined in the documentation
    BioRsVec::new(
        bio_vec,
        ((len.trailing_zeros() * len.trailing_zeros()) / 32) as usize,
    )
}

fn construct_fair_bio_vec(rng: &mut ThreadRng, len: usize) -> BioRsVec {
    let mut bio_vec = BioVec::new_fill(false, len as u64);
    for i in 0..len {
        bio_vec.set(i as u64, rng.gen_bool(0.5));
    }
    // k chosen to be a fair comparison to vers
    BioRsVec::new(bio_vec, 512 / 32)
}

fn construct_fid_vec(rng: &mut ThreadRng, len: usize) -> FidVec {
    let mut fid_vec = FidVec::new();
    for _ in 0..len {
        fid_vec.push(rng.gen_bool(0.5));
    }
    fid_vec
}

fn construct_ind_bit_vec(rng: &mut ThreadRng, len: usize) -> IndexedBits<Vec<u8>> {
    let vec = rng.sample_iter(Standard).take(len / 8).collect::<Vec<u8>>();
    IndexedBits::build_from_bytes(vec, len as u64).unwrap()
}

fn construct_rank9_vec(rng: &mut ThreadRng, len: usize) -> Rank9<SuccinctVec<u64>> {
    let mut bit_vec = SuccinctVec::with_capacity(len as u64);
    for _ in 0..len {
        bit_vec.push_bit(rng.sample(Standard))
    }
    Rank9::new(bit_vec)
}

fn construct_rank9_select_vec(
    rng: &mut ThreadRng,
    len: usize,
) -> BinSearchSelect<Rank9<SuccinctVec<u64>>> {
    BinSearchSelect::new(construct_rank9_vec(rng, len))
}

fn construct_sucds_vec(rng: &mut ThreadRng, len: usize) -> SucRank9Vec {
    let mut suc_bv = SucBitVec::with_capacity(len);
    for _ in 0..len / 64 {
        suc_bv
            .push_bits(rng.sample(Standard), 64)
            .expect("Failed to push bits into sucds bitvector");
    }

    SucRank9Vec::new(suc_bv).select0_hints()
}

fn construct_sucds_darray(rng: &mut ThreadRng, len: usize) -> SucDArray {
    let mut suc_bv = SucBitVec::with_capacity(len);
    for _ in 0..len / 64 {
        suc_bv
            .push_bits(rng.sample(Standard), 64)
            .expect("Failed to push bits into sucds bitvector");
    }

    SucDArray::from_bits(suc_bv.iter()).enable_rank()
}

fn construct_bitm_vec(rng: &mut ThreadRng, len: usize) -> RankSelect101111<CombinedSampling> {
    let mut bv = Box::<[u64]>::with_zeroed_bits(len);
    for i in 0..len {
        if rng.gen_bool(0.5) {
            bv.set_bit(i);
        }
    }

    bv.into()
}

fn construct_sux_rank9(rng: &mut ThreadRng, len: usize) -> Sux9 {
    let mut bit_vec = sux::prelude::BitVec::new(len);
    for i in 0..len {
        if rng.gen_bool(0.5) {
            bit_vec.set(i, true)
        }
    }

    Sux9::new(bit_vec)
}

fn construct_sux_small(rng: &mut ThreadRng, len: usize) -> RankSmall<1, 10, sux::bits::BitVec, Box<[usize]>, Box<[Block32Counters<1, 10>]>> {
    let mut bit_vec = sux::prelude::BitVec::new(len);
    for i in 0..len {
        if rng.gen_bool(0.5) {
            bit_vec.set(i, true)
        }
    }

    rank_small![2; bit_vec]
}

fn compare_ranks(b: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut group = b.benchmark_group("Rank: Randomized Input");
    group.plot_config(common::plot_config());

    for l in common::SIZES {
        let vers_vec = common::construct_vers_vec(&mut rng, l);
        let rsdict = construct_rsdict_vec(&mut rng, l);
        let bio_vec = construct_bio_vec(&mut rng, l);
        let fair_bio_vec = construct_fair_bio_vec(&mut rng, l);
        let fid_vec = construct_fid_vec(&mut rng, l);
        let ind_bit_vec = construct_ind_bit_vec(&mut rng, l);
        let rank9_vec = construct_rank9_vec(&mut rng, l);
        let sucds_vec = construct_sucds_vec(&mut rng, l);
        let sucds_darray = construct_sucds_darray(&mut rng, l);
        let bitm_vec = construct_bitm_vec(&mut rng, l);
        let sux_rank9 = construct_sux_rank9(&mut rng, l);
        let sux_small = construct_sux_small(&mut rng, l);

        let sample = Uniform::new(0, l);

        group.bench_with_input(BenchmarkId::new("vers", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(vers_vec.rank0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("rsdict", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(rsdict.rank(e, false)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("bio", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(bio_vec.rank_0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("fair bio", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(fair_bio_vec.rank_0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("fid", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(fid_vec.rank0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("indexed bitvector", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(ind_bit_vec.rank_zeros(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("succinct rank9", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(rank9_vec.rank0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("sucds-rank9", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(sucds_vec.rank0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("sucds-darray", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(sucds_darray.rank0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("bitm", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(bitm_vec.rank0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("sux-r9", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(sux_rank9.rank_zero(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("sux-small", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(sux_small.rank_zero(e)),
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

fn compare_selects(b: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let mut group = b.benchmark_group("Select: Randomized Input");
    group.plot_config(common::plot_config());

    for l in common::SIZES {
        let vers_vec = common::construct_vers_vec(&mut rng, l);
        let rsdict = construct_rsdict_vec(&mut rng, l);
        let bio_vec = construct_bio_vec(&mut rng, l);
        let fair_bio_vec = construct_fair_bio_vec(&mut rng, l);
        let fid_vec = construct_fid_vec(&mut rng, l);
        let ind_bit_vec = construct_ind_bit_vec(&mut rng, l);
        let rank9_vec = construct_rank9_select_vec(&mut rng, l);
        let sucds_vec = construct_sucds_vec(&mut rng, l);
        let sucds_darray = construct_sucds_darray(&mut rng, l);
        let bitm_vec = construct_bitm_vec(&mut rng, l);

        let sample = Uniform::new(0,
                                  [
                                      vers_vec.rank0(l),
                                      rsdict.rank(l as u64 - 1, false) as usize,
                                      bio_vec.rank_0(l as u64 - 1).unwrap() as usize,
                                      fair_bio_vec.rank_0(l as u64 - 1).unwrap() as usize,
                                      fid_vec.rank0(l as u64) as usize,
                                      ind_bit_vec.rank_zeros(l as u64 - 1).unwrap() as usize,
                                      rank9_vec.rank0(l as u64 - 1) as usize,
                                      sucds_vec.rank0(l).unwrap(),
                                      sucds_darray.rank0(l).unwrap(),
                                      bitm_vec.rank0(l - 1),
                                  ].iter().min().unwrap_or(&0));


        group.bench_with_input(BenchmarkId::new("vers", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(vers_vec.select0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("rsdict", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(rsdict.select(e, false)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("bio", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(bio_vec.select_0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("fair bio", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(fair_bio_vec.select_0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("fid", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(fid_vec.select0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("indexed bitvector", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(ind_bit_vec.select_zeros(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("succinct rank9", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng) as u64,
                |e| black_box(rank9_vec.select0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("sucds-rank9", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(sucds_vec.select0(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("sucds-darray", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(sucds_darray.select1(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("bitm", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(bitm_vec.select0(e)),
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, compare_ranks, compare_selects);
criterion_main!(benches);

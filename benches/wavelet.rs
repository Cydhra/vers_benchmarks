use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId};
use qwt::{AccessUnsigned, RankUnsigned};
use rand::distributions::{Standard, Uniform, Distribution};
use rand::{thread_rng, Rng};

mod common;

fn bench_access(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("Wavelet Matrix: Random Access");
    group.plot_config(common::plot_config());
    let mut rng = rand::thread_rng();

    for l in common::SIZES {
        let sequence = thread_rng()
            .sample_iter(Standard)
            .take(l >> 2)
            .collect::<Vec<u64>>();
        let sample = Uniform::new(0, sequence.len());

        // use the 64 bit limbs as 4 words at a time
        let vers_bit_vec = vers_vecs::BitVec::pack_sequence_u64(&sequence, 64);
        let vers_wavelet = vers_vecs::WaveletMatrix::from_bit_vec(&vers_bit_vec, 16);
        drop(vers_bit_vec);

        let cseq_wavelet = cseq::wavelet_matrix::Sequence::from_bits(&sequence, l, 16);

        let mut wav_wavelet = wavelet_matrix::WaveletMatrixBuilder::new();
        for &e in sequence.iter() {
            wav_wavelet.push(e & 0xFFFF);
            wav_wavelet.push((e >> 16) & 0xFFFF);
            wav_wavelet.push((e >> 32) & 0xFFFF);
            wav_wavelet.push((e >> 48) & 0xFFFF);
        }
        let wav_wavelet = wav_wavelet.build();

        let qwt_wavelet = qwt::QWT512::from_iter(sequence.iter().flat_map(|&e| [e & 0xFFFF, (e >> 16) & 0xFFFF, (e >> 32) & 0xFFFF, (e >> 48) & 0xFFFF]));

        drop(sequence);

        group.bench_with_input(BenchmarkId::new("vers wavelet access", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(vers_wavelet.get_u64(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("cseq wavelet access", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(cseq_wavelet.get(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("wm wavelet access", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(wav_wavelet.lookup(e)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("qwt wavelet access", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(qwt_wavelet.get(e)),
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn bench_rank(c: &mut criterion::Criterion) {
    let mut group = c.benchmark_group("Wavelet Matrix: Rank");
    group.plot_config(common::plot_config());
    let mut rng = rand::thread_rng();

    for l in common::SIZES {
        let sequence = thread_rng()
            .sample_iter(Standard)
            .take(l >> 2)
            .collect::<Vec<u64>>();
        let sample = Uniform::new(0, sequence.len());

        // use the 64 bit limbs as 4 words at a time
        let vers_bit_vec = vers_vecs::BitVec::pack_sequence_u64(&sequence, 64);
        let vers_wavelet = vers_vecs::WaveletMatrix::from_bit_vec(&vers_bit_vec, 16);
        drop(vers_bit_vec);

        let cseq_wavelet = cseq::wavelet_matrix::Sequence::from_bits(&sequence, l, 16);

        let mut wav_wavelet = wavelet_matrix::WaveletMatrixBuilder::new();
        for &e in sequence.iter() {
            wav_wavelet.push(e & 0xFFFF);
            wav_wavelet.push((e >> 16) & 0xFFFF);
            wav_wavelet.push((e >> 32) & 0xFFFF);
            wav_wavelet.push((e >> 48) & 0xFFFF);
        }
        let wav_wavelet = wav_wavelet.build();

        let qwt_wavelet = qwt::QWT512::from_iter(sequence.iter().flat_map(|&e| [e & 0xFFFF, (e >> 16) & 0xFFFF, (e >> 32) & 0xFFFF, (e >> 48) & 0xFFFF]));

        drop(sequence);

        group.bench_with_input(BenchmarkId::new("vers wavelet rank", l), &l, |b, _| {
            b.iter_batched(
                || (sample.sample(&mut rng), rng.gen_range(0..=0xFFFFu64)),
                |(e, symbol)| black_box(vers_wavelet.rank_u64(e, symbol)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("cseq wavelet rank", l), &l, |b, _| {
            b.iter_batched(
                || (sample.sample(&mut rng), rng.gen_range(0..=0xFFFFu64)),
                |(e, symbol)| black_box(cseq_wavelet.rank(e, symbol)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("wm wavelet rank", l), &l, |b, _| {
            b.iter_batched(
                || (sample.sample(&mut rng), rng.gen_range(0..=0xFFFFu64)),
                |(e, symbol)| black_box(wav_wavelet.rank(e, symbol)),
                BatchSize::SmallInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("qwt wavelet rank", l), &l, |b, _| {
            b.iter_batched(
                || (sample.sample(&mut rng), rng.gen_range(0..=0xFFFFu64)),
                |(e, symbol)| black_box(qwt_wavelet.rank(symbol, e)),
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, bench_rank);
criterion_main!(benches);

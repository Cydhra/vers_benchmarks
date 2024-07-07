use std::time::{Duration, Instant};

use criterion::{BatchSize, BenchmarkId, black_box, Criterion, criterion_group, criterion_main};
use cseq::elias_fano::Builder;
use elias_fano::EliasFano;
use rand::{Rng, thread_rng};
use rand::distributions::{Distribution, Standard, Uniform};
use sucds::mii_sequences::EliasFanoBuilder;
use vers_vecs::EliasFanoVec;

mod common;

fn elias_fano_random_access(b: &mut Criterion) {
    let mut group = b.benchmark_group("Elias-Fano: random-access");
    group.plot_config(common::plot_config());

    let mut rng = thread_rng();

    for l in common::SIZES {
        let mut sequence = thread_rng()
            .sample_iter(Standard)
            .take(l)
            .collect::<Vec<u64>>();
        sequence.sort_unstable();
        let sample = Uniform::new(0, sequence.len());

        let ef_vec = EliasFanoVec::from_slice(&sequence);
        group.bench_with_input(BenchmarkId::new("vers vector", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(ef_vec.get_unchecked(e)),
                BatchSize::SmallInput,
            )
        });
        drop(ef_vec);

        let mut elias_fano_vec =
            EliasFano::new(sequence[sequence.len() - 1], sequence.len() as u64);
        elias_fano_vec.compress(sequence.iter());
        group.bench_with_input(BenchmarkId::new("elias-fano vector", l), &l, |b, _| {
            b.iter_batched(
                || sample.sample(&mut rng),
                |e| black_box(elias_fano_vec.visit(e as u64)),
                BatchSize::SmallInput,
            )
        });
        drop(elias_fano_vec);

        let mut sucds_ef_vec =
            EliasFanoBuilder::new(*sequence.last().unwrap() as usize + 1, sequence.len())
                .expect("Failed to create sucds Elias-Fano builder");

        sucds_ef_vec
            .extend(sequence.iter().map(|e| *e as usize))
            .expect("Failed to extend sucds Elias-Fano builder");
        let sucds_ef_vec = sucds_ef_vec.build();
        group.bench_with_input(
            BenchmarkId::new("sucds elias fano vector", l),
            &l,
            |b, _| {
                b.iter_batched(
                    || sample.sample(&mut rng),
                    |e| black_box(sucds_ef_vec.select(e)),
                    BatchSize::SmallInput,
                )
            },
        );
        drop(sucds_ef_vec);

        let mut cseq_ef_vec = Builder::new(l, *sequence.last().unwrap() + 1);
        cseq_ef_vec.push_all(sequence.iter().copied());
        let cseq_ef_vec = cseq_ef_vec.finish();

        group.bench_with_input(
            BenchmarkId::new("cseq elias fano vector", l),
            &l,
            |b, _| {
                b.iter_batched(
                    || sample.sample(&mut rng),
                    |e| black_box(cseq_ef_vec.get(e)),
                    BatchSize::SmallInput,
                )
            },
        );
        drop(cseq_ef_vec);

    }
    group.finish();
}

fn elias_fano_in_order(b: &mut Criterion) {
    let mut group = b.benchmark_group("Elias-Fano: in-order-access");
    group.plot_config(common::plot_config());

    for l in common::SIZES {
        let mut sequence = thread_rng()
            .sample_iter(Standard)
            .take(l)
            .collect::<Vec<u64>>();
        sequence.sort_unstable();

        // cseq cannot handle u64::MAX
        let mut i = sequence.len() - 1;
        while sequence[i] == u64::MAX {
            sequence[i] = u64::MAX - 1;
            i -= 1;
        }

        let ef_vec = EliasFanoVec::from_slice(&sequence);
        group.bench_with_input(BenchmarkId::new("vers vector", l), &l, |b, _| {
            b.iter_custom(|iters| {
                let mut time = Duration::new(0, 0);
                let mut i = 0;

                while i < iters {
                    let iter = ef_vec.iter().take((iters - i) as usize);
                    let start = Instant::now();
                    for e in iter {
                        black_box(e);
                        i += 1;
                    }
                    time += start.elapsed();
                }

                time
            })
        });
        drop(ef_vec);

        let mut elias_fano_vec =
            EliasFano::new(sequence[sequence.len() - 1], sequence.len() as u64);
        elias_fano_vec.compress(sequence.iter());
        group.bench_with_input(BenchmarkId::new("elias-fano vector", l), &l, |b, _| {
            b.iter_custom(|iters| {
                let mut time = Duration::new(0, 0);
                let mut i = 0;

                while i < iters {
                    elias_fano_vec.reset();
                    let start = Instant::now();
                    while elias_fano_vec.next().is_ok() && i < iters {
                        i += 1;
                    }
                    time += start.elapsed();
                }

                time
            })
        });
        drop(elias_fano_vec);

        let mut sucds_ef_vec =
            EliasFanoBuilder::new(*sequence.last().unwrap() as usize + 1, sequence.len())
                .expect("Failed to create sucds Elias-Fano builder");
        sucds_ef_vec
            .extend(sequence.iter().map(|e| *e as usize))
            .expect("Failed to extend sucds Elias-Fano builder");
        let sucds_ef_vec = sucds_ef_vec.build();
        group.bench_with_input(
            BenchmarkId::new("sucds elias fano vector", l),
            &l,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut time = Duration::new(0, 0);
                    let mut i = 0;

                    while i < iters {
                        let iter = sucds_ef_vec.iter(0).take((iters - i) as usize);
                        let start = Instant::now();
                        for e in iter {
                            black_box(e);
                            i += 1;
                        }
                        time += start.elapsed();
                    }

                    time
                })
            },
        );
        drop(sucds_ef_vec);

        let mut cseq_ef_vec = Builder::new(l, *sequence.last().unwrap() + 1);
        cseq_ef_vec.push_all(sequence.iter().copied());
        let cseq_ef_vec = cseq_ef_vec.finish();

        group.bench_with_input(
            BenchmarkId::new("cseq elias fano vector", l),
            &l,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut time = Duration::new(0, 0);
                    let mut i = 0;

                    while i < iters {
                        let iter = cseq_ef_vec.iter().take((iters - i) as usize);
                        let start = Instant::now();
                        for e in iter {
                            black_box(e);
                            i += 1;
                        }
                        time += start.elapsed();
                    }

                    time
                })
            },
        );
        drop(cseq_ef_vec);
    }
    group.finish();
}

criterion_group!(benches, elias_fano_random_access, elias_fano_in_order);
criterion_main!(benches);

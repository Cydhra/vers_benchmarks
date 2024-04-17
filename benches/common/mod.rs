#![allow(dead_code)]

use criterion::PlotConfiguration;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;
use vers_vecs::{BitVec, RsVec};

pub const SIZES: [usize; 11] = [
    1 << 8,
    1 << 10,
    1 << 12,
    1 << 14,
    1 << 16,
    1 << 18,
    1 << 20,
    1 << 22,
    1 << 24,
    1 << 26,
    1 << 28,
];

pub fn construct_vers_vec(rng: &mut ThreadRng, len: usize) -> RsVec {
    let sample = Uniform::new(0, u64::MAX);

    let mut bit_vec = BitVec::new();
    for _ in 0..len / 64 {
        bit_vec.append_word(sample.sample(rng));
    }

    RsVec::from_bit_vec(bit_vec)
}

pub fn plot_config() -> PlotConfiguration {
    PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic)
}

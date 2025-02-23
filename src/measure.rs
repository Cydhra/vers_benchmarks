use std::marker::PhantomData;
use std::time::Instant;
use crate::runner::Runner;

/// The minimum time any measurement has to run to avoid noise in nanoseconds. Currently, 100 milliseconds.
const MINIMUM_RUNNING_TIME: u64 = 100_000_000;

// How long each chunk of the benchmark should run in nanoseconds. Currently, 10 seconds.
const CHUNK_TIME: u64 = 10_000_000_000;

/// A single benchmark measurement of one function.
/// The function can be benchmarked multiple times in interleave with other benchmarks.
pub(crate) struct Measurement<'a, S, P> {
    func: &'a dyn Runner<Context = S, Param = P>,
    name: &'a str,
    repetitions: u64,
    samples: Vec<u64>
}

impl<'a, S, P> Measurement<'a, S, P> {
    pub(crate) fn new(name: &'a str, func: &'a dyn Runner<Context=S, Param=P>) -> Self {
        Self { func, name, repetitions: 0, samples: Vec::new() }
    }

    pub(crate) fn estimate_timing(&mut self) {
        let mut timing = 0;
        let mut repetitions = 1;

        loop {
            let state = self.func.create_context(128);
            let params = self.func.prepare_params(repetitions as usize, 128);

            let start = Instant::now();
            for _ in 0..repetitions {
                self.func.execute(&state, &params[0]);
            }
            timing = start.elapsed().as_nanos();

            if timing > MINIMUM_RUNNING_TIME as u128 {
                break;
            } else {
                repetitions *= 2;
            }
        }

        let time_per_call = timing as f64 / repetitions as f64;
        let reps_per_measurement = (MINIMUM_RUNNING_TIME as f64 / time_per_call).ceil() as u64;
        println!("{} requires approx. {time_per_call:.2} ns per call and thus we run {reps_per_measurement} repetitions per measurement", self.name);

        self.repetitions = reps_per_measurement;
    }

    pub(crate) fn benchmark_chunk(&mut self) {
        if self.repetitions == 0 {
            eprintln!("Please call estimate_timing() before benchmark_chunk()");
            return;
        }
        let mut samples = Vec::with_capacity(CHUNK_TIME as usize / MINIMUM_RUNNING_TIME as usize);

        println!("measuring chunk for {}", self.name);
        let chunk_start = Instant::now();
        loop {
            let state = self.func.create_context(128);
            let params = self.func.prepare_params(self.repetitions as usize, 128);

            let start = Instant::now();
            for _ in 0..self.repetitions {
                self.func.execute(&state, &params[0]);
            }
            samples.push(start.elapsed().as_nanos() as u64);

            if chunk_start.elapsed().as_nanos() > CHUNK_TIME as u128 {
                break;
            }
        }
        println!("Appending {} samples to {} benchmark", samples.len(), self.name);
        self.samples.extend(samples);

        let mean = (self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64) / self.repetitions as f64;
        let std_deviation = (self.samples.iter().map(|&x| ((x as f64 / self.repetitions as f64) - mean).powf(2.0)).sum::<f64>() / (self.samples.len() - 1) as f64).sqrt();
        let relative_std_deviation = std_deviation / mean;

        println!("Mean: {mean:.2} ns +- {relative_std_deviation:.4}%");
    }
}

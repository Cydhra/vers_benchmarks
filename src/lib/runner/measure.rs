use std::io::{stdout, Write};
use crate::runner::Runner;
use std::time::Instant;

/// The minimum time any measurement has to run to avoid noise in nanoseconds. Currently, 100 milliseconds.
const MINIMUM_RUNNING_TIME: u64 = 100_000_000;

// How long each chunk of the benchmark should run in nanoseconds. Currently, 1 second.
const CHUNK_TIME: u64 = 1_000_000_000;

/// A single benchmark measurement of one function.
/// The function can be benchmarked multiple times in interleave with other benchmarks.
pub struct Measurement<'a, S, P> {
    func: &'a dyn Runner<Context = S, Param = P>,
    pub(crate) name: &'a str,
    repetitions: usize,
    samples: Vec<u64>,
    size: usize,
}

/// Result of a collection of samples obtained through benchmarking. This may be any number of
/// samples, as long as it is enough to determined median, q1, and q3.
#[derive(Debug, Clone)]
pub(crate) struct MeasurementResult {
    pub mean: f64,
    pub std_deviation: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
}

impl<'a, State, Param> Measurement<'a, State, Param> {
    pub(crate) fn new(name: &'a str, func: &'a dyn Runner<Context = State, Param = Param>) -> Self {
        Self {
            func,
            name,
            repetitions: 0,
            samples: Vec::new(),
            size: 0,
        }
    }

    /// Initialize the `Measurement` with a new data structure size, and reset all previously collected
    /// measurements.
    /// A call to [`estimate_timing`] is necessary before [`benchmark`] can be called again.
    pub fn initialize_measurement(&mut self, size: usize) {
        self.samples.clear();
        self.size = size;
        self.repetitions = 0;
    }

    #[inline]
    pub fn create_state(&self) -> State {
        self.func.create_context(self.size)
    }

    #[inline(always)]
    fn run_target_function(&self, state: &State, params: &[Param]) {
        for param in params {
            self.func.execute(state, param);
        }
    }

    pub(crate) fn estimate_timing(&mut self) {
        if self.size == 0 {
            eprintln!("Please call initialize_measurement(size) before starting timing.");
            return;
        }

        let mut timing;
        let mut repetitions = 1;

        loop {
            let state = self.create_state();
            let params = self.func.prepare_params(repetitions as usize, self.size);

            let start = Instant::now();
            self.run_target_function(&state, &params);
            timing = start.elapsed().as_nanos();

            if timing > MINIMUM_RUNNING_TIME as u128 {
                break;
            } else {
                repetitions *= 2;
            }
        }

        let time_per_call = timing as f64 / repetitions as f64;
        let reps_per_measurement = (MINIMUM_RUNNING_TIME as f64 / time_per_call).ceil() as u64;
        if reps_per_measurement > usize::MAX as u64 {
            panic!("cannot handle more than usize::MAX repetitions in benchmark");
        }

        println!("{} requires approx. {time_per_call:.2} ns per call and thus we run {reps_per_measurement} repetitions per measurement", self.name);

        self.repetitions = reps_per_measurement as usize;
    }

    pub(crate) fn benchmark_chunk(&mut self) {
        if self.repetitions == 0 {
            eprintln!("Please call estimate_timing() before benchmark_chunk()");
            return;
        }
        let mut samples = Vec::with_capacity(CHUNK_TIME as usize / MINIMUM_RUNNING_TIME as usize);

        print!("Measuring chunk for {}...", self.name);
        let _ = stdout().flush();

        let mut total_sample_time = 0;
        loop {
            let state = self.func.create_context(self.size);
            let params = self
                .func
                .prepare_params(self.repetitions, self.size);

            // warmup
            let warmup_reps = params.len() / 100;
            self.run_target_function(&state, &params[..warmup_reps]);

            // measure one sample
            let start = Instant::now();
            self.run_target_function(&state, &params);
            let time_sample = start.elapsed().as_nanos();
            samples.push(time_sample as u64);

            // check if chunk time has been reached
            total_sample_time += time_sample;
            if total_sample_time > CHUNK_TIME as u128 {
                break;
            }
        }
        print!(
            "\rMeasured {} samples for {}: ",
            samples.len(),
            self.name
        );
        self.samples.extend(samples);

        let MeasurementResult {
            mean,
            std_deviation,
            ..
        } = self.get_final_measurement();
        let relative_std_deviation = std_deviation / mean * 100.0;
        println!("Mean: {mean:.2} ns +- {relative_std_deviation:.4}%");
    }

    /// Get the mean, standard deviation,
    /// relative standard deviation (i.e., standard deviation divided by mean),
    /// min, and max of all samples that have been measured so far.
    pub(crate) fn get_final_measurement(&self) -> MeasurementResult {
        assert!(self.samples.len() >= 4);

        let mean = (self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64)
            / self.repetitions as f64;
        let std_deviation = (self
            .samples
            .iter()
            .map(|&x| ((x as f64 / self.repetitions as f64) - mean).powf(2.0))
            .sum::<f64>()
            / (self.samples.len() - 1) as f64)
            .sqrt();

        let mut sorted_samples = self.samples.clone();
        sorted_samples.sort();

        let min = sorted_samples[0] as f64 / self.repetitions as f64;
        let max = sorted_samples[sorted_samples.len() - 1] as f64 / self.repetitions as f64;

        let median = if sorted_samples.len().is_multiple_of(2) {
            (sorted_samples[sorted_samples.len() / 2 - 1]
                + sorted_samples[sorted_samples.len() / 2]) as f64
                / 2.0
        } else {
            sorted_samples[sorted_samples.len() / 2] as f64
        } / self.repetitions as f64;

        let q1 = sorted_samples[sorted_samples.len() / 4] as f64 / self.repetitions as f64;
        let q3 = sorted_samples[sorted_samples.len() / 4 * 3] as f64 / self.repetitions as f64;

        MeasurementResult {
            mean,
            std_deviation,
            min,
            max,
            median,
            q1,
            q3,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

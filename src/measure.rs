use crate::runner::Runner;
use std::time::Instant;

/// The minimum time any measurement has to run to avoid noise in nanoseconds. Currently, 100 milliseconds.
/// The benchmark repeats the measured function to reach this target time, and averages out the
/// runtime.
const MINIMUM_RUNNING_TIME: u64 = 100_000_000;

/// How long each chunk of the benchmark should run in nanoseconds. Currently, 5 seconds.
const CHUNK_TIME: u64 = 5_000_000_000;

/// A single benchmark measurement of one function.
/// The function can be benchmarked multiple times interleaved with other benchmarks.
pub(crate) struct Measurement<'a, S, P> {
    func: &'a dyn Runner<Context = S, Param = P>,
    pub(crate) name: &'a str,
    repetitions: u64,
    samples: Vec<u64>,
    size: usize,
}

impl<'a, State, Param> Measurement<'a, State, Param> {
    pub(crate) fn new(name: &'a str, func: &'a dyn Runner<Context=State, Param=Param>) -> Self {
        Self { func, name, repetitions: 0, samples: Vec::new(), size: 0 }
    }

    /// Initialize the `Measurement` with a new data structure size, and reset all previously collected
    /// measurements.
    /// A call to [`estimate_timing`] is necessary before [`benchmark`] can be called again.
    pub(crate) fn initialize_measurement(&mut self, size: usize) {
        self.samples.clear();
        self.size = size;
        self.repetitions = 0;
    }

    pub(crate) fn estimate_timing(&mut self) {
        if self.size == 0 {
            eprintln!("Please call initialize_measurement(size) before starting timing.");
            return;
        }

        let mut timing;
        let mut repetitions = 1;

        loop {
            let state = self.func.create_context(self.size);
            let params = self.func.prepare_params(repetitions as usize, self.size);

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

        println!("Measuring chunk for {}", self.name);
        let chunk_start = Instant::now();
        loop {
            let state = self.func.create_context(self.size);
            let params = self.func.prepare_params(self.repetitions as usize, self.size);

            let start = Instant::now();
            for _ in 0..self.repetitions {
                self.func.execute(&state, &params[0]);
            }
            samples.push(start.elapsed().as_nanos() as u64);

            if chunk_start.elapsed().as_nanos() > CHUNK_TIME as u128 {
                break;
            }
        }
        self.samples.extend(samples);

        let (mean, _, relative_std_deviation, _, _) = self.get_final_measurement();

        println!("Median: {mean:.2} ns +- {:.4}%", relative_std_deviation * 100.0);
    }

    /// Get the mean, standard deviation,
    /// relative standard deviation (i.e., standard deviation divided by median),
    /// min, and max of all samples that have been measured so far.
    ///
    /// The function requires a mutable reference because it sorts the measurements in-place.
    pub(crate) fn get_final_measurement(&mut self) -> (f64, f64, f64, f64, f64) {
        self.samples.sort_unstable();

        // technically not the median as defined by literature, but since this is the median of means
        // anyway, I am not too concerned.
        let median = self.samples[self.samples.len() / 2] as f64 / self.repetitions as f64;
        let std_deviation = (self.samples.iter().map(|&x| ((x as f64 / self.repetitions as f64) - median).powf(2.0)).sum::<f64>() / (self.samples.len() - 1) as f64).sqrt();
        let relative_std_deviation = std_deviation / median;
        let min = self.samples[0] as f64 / self.repetitions as f64;
        let max = self.samples[self.samples.len() - 1] as f64 / self.repetitions as f64;

        (median, std_deviation, relative_std_deviation, min, max)
    }
}

use crate::measure::{Measurement, MeasurementResult};
use std::io::Write;
use std::ops::Range;

const NUM_ROUNDS: usize = 4;

/// A benchmark is a collection of ([`Measurements`]) that are run interleaved.
/// Each measurement defines a [`Runner`] which defines how to execute the benchmarked function.
/// The runner is called repeatedly to create one sample measurement, and several runs of measurements
/// are repeated by the `Measurement` to benchmark the function for a given state size.
/// The measurement can then be repeated for different state sizes to create a series.
///
/// [`Measurements`]: Measurement
/// [`Runner`]: crate::runner::Runner
pub(crate) struct Benchmark<'a, State, Param> {
    name: String,
    runners: Vec<Measurement<'a, State, Param>>,
    sizes: Vec<usize>,
}

impl<'a, State, Param> Benchmark<'a, State, Param> {
    /// Create a new benchmark with a `name` and a list of state sizes. The benchmark is repeated
    /// for all attached [`Measurements`] for each state size.
    ///
    /// [`Measurements`]: Measurement
    pub(crate) fn new(name: &str, sizes: Vec<usize>) -> Self {
        Self {
            name: name.to_string(),
            runners: Vec::new(),
            sizes,
        }
    }

    /// Create a new benchmark with a `name` and a range of input size given as logarithms.
    /// The benchmark is repeated for all attached [`Measurements`] with a state size of
    /// `2^i` for each size `i` in the given `size_range`
    ///
    /// [`Measurements`]: Measurement
    pub(crate) fn from_powers(name: &str, size_range: Range<usize>) -> Self {
        Self::new(name, size_range.into_iter().collect())
    }

    pub(crate) fn add_measurement(&mut self, runner: Measurement<'a, State, Param>) {
        self.runners.push(runner);
    }

    pub(crate) fn benchmark<W: Write>(&mut self, output: &mut W) {
        let mut size_index = 0;

        writeln!(
            output,
            "Benchmark\tInput Size\tRunner\tMean\tStd. Deviation\tMin\tMax\tMedian\tQ1\tQ3"
        )
        .expect("failed to write into output stream");

        // don't borrow the size vec
        while size_index < self.sizes.len() {
            let current_size = self.sizes[size_index];

            for runner in self.runners.iter_mut() {
                runner.initialize_measurement(current_size);
                runner.estimate_timing();
            }

            for _ in 0..NUM_ROUNDS {
                for runner in self.runners.iter_mut() {
                    runner.benchmark_chunk();
                }
            }

            for runner in &self.runners {
                let MeasurementResult {
                    mean,
                    std_deviation,
                    min,
                    max,
                    median,
                    q1,
                    q3,
                } = runner.get_final_measurement();
                writeln!(
                    output,
                    "{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}",
                    self.name,
                    current_size,
                    runner.name,
                    mean,
                    std_deviation,
                    min,
                    max,
                    median,
                    q1,
                    q3
                )
                .expect("failed to write results");
            }

            size_index += 1;
        }
    }
}

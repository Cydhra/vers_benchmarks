use crate::measure::Measurement;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

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
        Self { name: name.to_string(), runners: Vec::new(), sizes }
    }

    pub(crate) fn add_measurement(&mut self, runner: Measurement<'a, State, Param>) {
        self.runners.push(runner);
    }

    pub(crate) fn benchmark(&mut self, output_dir: &Path) {
        let mut size_index = 0;

        let path = output_dir.join(Path::new(&(self.name.to_owned().to_lowercase() + ".csv")));
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .unwrap();

        // don't borrow the size vec
        while size_index < self.sizes.len() {
            let current_size =  self.sizes[size_index];
            println!("Benchmarking {} elements...", current_size);

            for runner in self.runners.iter_mut() {
                runner.initialize_measurement(current_size);
                runner.estimate_timing();
            }

            for i in 0..5 {
                println!("Round {}...", i);
                for runner in self.runners.iter_mut() {
                    runner.benchmark_chunk();
                }
            }

            for runner in self.runners.iter_mut() {
                let (mean, std_dev, rel_std_dev, min, max) = runner.get_final_measurement();
                println!("[{}/{}]\t{}\tMean: {:.6}\t [{:.6}-{:.6}],\t Std. Dev: {:.6} ({:.3}%)", self.name, current_size, runner.name, mean, min, max, std_dev, rel_std_dev * 100.0);
                writeln!(file, "{},{},{},{},{},{},{}", self.name, runner.name, current_size, mean, min, max, std_dev).unwrap_or_else(|e| eprintln!("WARNING: {}", e));
            }

            size_index += 1;
        }
    }
}
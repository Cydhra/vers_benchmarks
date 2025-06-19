use crate::measure::Measurement;

/// A benchmark is a collection of benchmark groups that are run interleave.
/// In the end, the group is converted into a report that can later be merged with more reports
/// for a final benchmark report.
pub(crate) struct Benchmark<'a, State, Param> {
    name: String,
    runners: Vec<Measurement<'a, State, Param>>
}

impl<'a, State, Param> Benchmark<'a, State, Param> {
    pub(crate) fn new(name: &str) -> Self {
        Self { name: name.to_string(), runners: Vec::new() }
    }

    pub(crate) fn add_measurement(&mut self, runner: Measurement<'a, State, Param>) {
        self.runners.push(runner);
    }

    pub(crate) fn benchmark(&mut self) {
        for runner in self.runners.iter_mut() {
            runner.initialize_measurement(128);
            runner.estimate_timing();
        }

        for _ in 0..5 {
            for runner in self.runners.iter_mut() {
                runner.benchmark_chunk();
            }
        }
    }
}
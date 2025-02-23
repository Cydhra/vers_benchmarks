use std::task::Context;
use crate::measure::Measurement;

/// A benchmark is a collection of benchmark groups that are run interleave.
/// In the end, the group is converted into a report that can later be merged with more reports
/// for a final benchmark report.
pub(crate) struct Benchmark<'a, S, P> {
    name: String,
    runners: Vec<Measurement<'a, S, P>>
}

impl<'a, S, P> Benchmark<'a, S, P> {
    pub(crate) fn new(name: &str) -> Self {
        Self { name: name.to_string(), runners: Vec::new() }
    }

    pub(crate) fn add_measurement(&mut self, runner: Measurement<'a, S, P>) {
        self.runners.push(runner);
    }

    pub(crate) fn benchmark(&mut self) {
        for runner in self.runners.iter_mut() {
            runner.estimate_timing();
        }

        for _ in 0..5 {
            for runner in self.runners.iter_mut() {
                runner.benchmark_chunk();
            }
        }
    }
}
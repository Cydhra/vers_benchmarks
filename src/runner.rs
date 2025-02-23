use std::time::Instant;

/// The minimum time any measurement has to run to avoid noise in nanoseconds. Currently, 100 milliseconds.
const MINIMUM_RUNNING_TIME: u64 = 100_000_000;

/// The runtime of one variance test iteration. Currently, 3 seconds.
const VARIANCE_TEST_RUNTIME: u64 = 3_000_000_000;

pub(crate) struct Runner<F: Fn() -> ()> {
    f: F,
}

impl<F: Fn() -> ()> Runner<F> {

    pub(crate) fn new(f: F) -> Self {
        Self { f }
    }

    pub(crate) fn estimate_timing(&self) -> u64 {
        let mut timing = 0;
        let mut repetitions = 1;

        loop {
            let start = Instant::now();
            for _ in 0..repetitions {
                (self.f)();
            }
            timing = start.elapsed().as_nanos();

            if timing > MINIMUM_RUNNING_TIME as u128 {
                break;
            } else {
                repetitions *= 2;
            }
        }

        (timing / repetitions) as u64
    }

    pub(crate) fn estimate_mean_deviation(&self, timing: u64) -> (f64, f64) {
        let start = Instant::now();
        let mut measurement_length = MINIMUM_RUNNING_TIME / timing;
        let mut num_measurements = VARIANCE_TEST_RUNTIME / MINIMUM_RUNNING_TIME;

        let mut samples = Vec::with_capacity(num_measurements as usize);
        let mut deviation_samples = Vec::with_capacity(10);
        let mut last_mean = 0.0;

        loop {
            let mut immediate_samples = Vec::with_capacity(num_measurements as usize);
            for _ in 0..num_measurements {
                let start = Instant::now();
                for _ in 0..measurement_length {
                    (self.f)();
                }
                immediate_samples.push(start.elapsed().as_nanos());
            }

            samples.append(&mut immediate_samples);
            
            samples.sort_unstable();
            let quartile_size = samples.len() / 4;

            let mean = samples[quartile_size..quartile_size * 3].iter().map(|&x| f64::from(x as u32)).sum::<f64>() / (quartile_size * 2) as f64;
            let deviation = (samples[quartile_size..quartile_size * 3].iter().map(|&x| (f64::from(x as u32) - mean) * (f64::from(x as u32) - mean)).sum::<f64>() / (quartile_size * 2 - 1) as f64).sqrt();
            deviation_samples.push(deviation);

            println!("Standard Deviation: {}", deviation);

            if deviation_samples.len() >= 3 {
                let last_deviation = deviation_samples.last().unwrap();
                let relative_change = (last_deviation - deviation_samples[deviation_samples.len() - 2]) / deviation_samples[deviation_samples.len() - 2];
                let relative_change2 = (last_deviation - deviation_samples[deviation_samples.len() - 3]) / deviation_samples[deviation_samples.len() - 3];

                if relative_change.abs() < 0.01 && relative_change2.abs() < 0.01 {
                    println!("Convergence reached.");
                    last_mean = mean;
                    break;
                }
            }
        }

        println!("Analysis took {} ms", start.elapsed().as_millis());
        (last_mean / measurement_length as f64, deviation_samples.last().unwrap() / measurement_length as f64)
    }
}
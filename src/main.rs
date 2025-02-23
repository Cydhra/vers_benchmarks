use std::hint::black_box;
use rand::RngCore;
use vers_vecs::{BitVec, RsVec};

mod runner;

fn main() {
    let mut rng = rand::thread_rng();
    let mut bv = BitVec::with_capacity(1 << 20);
    const L: usize = 100000;

    for _ in 0..(1 << 14) {
        bv.append_word(rng.next_u64());
    }

    let rs = RsVec::from_bit_vec(bv);
    let idxes = (0..L).map(|i| rng.next_u64() as usize % rs.len()).collect::<Vec<_>>();

    let runner = runner::Runner::new(|| {
        for i in 0..L {
            black_box(rs.rank1(idxes[i]));
        }
    });

    let timing = runner.estimate_timing();
    let (mean, variance) = runner.estimate_mean_deviation(timing);
    println!("Estimated time per execution: {} ns", timing);
    println!("Mean time per execution: {} ns", mean);
    println!("Standard Deviation of execution: {} ns", variance);

    // read line to keep the console open
    let mut _line = String::new();
    std::io::stdin().read_line(&mut _line).unwrap();
}

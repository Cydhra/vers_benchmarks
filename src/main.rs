use crate::benches::rank;

mod measure;
mod benchmark;
mod benches;
mod runner;

fn main() {
    rank::benchmark();
}
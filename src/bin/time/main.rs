use runner::benches::*;
use std::fs;
use std::path::Path;
use std::process::exit;



const MEASUREMENTS_DIR: &str = "./measurements";

fn main() {
    let directory = Path::new(MEASUREMENTS_DIR);
    fs::create_dir_all(directory).unwrap_or_else(|e| {
        eprintln!("Could not create measurements directory: {}", e);
        exit(1);
    });

    rank::benchmark(&directory);
    select::benchmark(&directory);
    elias_fano::benchmark(&directory);
}

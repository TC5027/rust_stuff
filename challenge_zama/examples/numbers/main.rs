use challenge_zama::*;

mod dataset;
mod parameters;

use dataset::DATASET;
use parameters::*;

fn main() {
    for number in DATASET.iter() {
        let matrix = Matrix::new(28,28,number);
    }
}
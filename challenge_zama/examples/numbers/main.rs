use challenge_zama::*;

mod dataset;
mod parameters;

use dataset::DATASET;
use parameters::*;

fn custom_print(number: &[f64; 784]) {
    for i in 0..28 {
        let mut string = String::new();
        for j in 0..28 {
            if number[28 * i + j] > 0.5 {
                string.push('.');
            } else {
                string.push(' ');
            }
        }
        println!("{}", string);
    }
}

fn main() {
    for number in DATASET.iter().take(5) {
        println!("number seen : ");
        custom_print(&number);

        let mut weight_1 = Matrix::new(28 * 28, 256, &WEIGHT_1);
        weight_1.transpose();
        let bias_1 = Matrix::new(256, 1, &BIAS_1);

        let mut weight_2 = Matrix::new(256, 32, &WEIGHT_2);
        weight_2.transpose();
        let bias_2 = Matrix::new(32, 1, &BIAS_2);

        let mut weight_3 = Matrix::new(32, 10, &WEIGHT_3);
        weight_3.transpose();
        let bias_3 = Matrix::new(10, 1, &BIAS_3);

        let mut matrix = Matrix::new(28, 28, number);
        matrix.flatten();
        matrix.linear_combination(&weight_1, &bias_1);
        matrix.relu();
        matrix.linear_combination(&weight_2, &bias_2);
        matrix.relu();
        matrix.linear_combination(&weight_3, &bias_3);
        matrix.softmax();

        let mut index = 0;
        let mut maxi = matrix.data[0];
        for i in 1..10 {
            if matrix.data[i] > maxi {
                index = i;
                maxi = matrix.data[i];
            }
        }

        println!(
            "number recognized is {} with value {}",
            index, matrix.data[index]
        );
    }
}

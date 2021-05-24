use challenge_zama::*;

mod dataset;
mod parameters;

use dataset::DATASET;
use parameters::*;

fn custom_print(number : &[f64;784]) {
    for i in 0..28 {
        let mut string = String::new();
        for j in 0..28 {
            if number[28*i+j]>0.5 {
                string.push('.');
            } else {
                string.push(' ');
            }
        }
        println!("{}",string);
    }
}

fn main() {
    for number in DATASET[45..].iter().take(1) {
        println!("number seen : ");
        custom_print(&number);

        let mut matrix = Matrix::new(28,28,number.iter().map(|&x| x));
        matrix.flatten();
        matrix = linear_combination(matrix,Matrix::new(256,28*28,WEIGHT_1.iter().map(|&x| x)),Matrix::new(256,1,BIAS_1.iter().map(|&x| x)));
        matrix.relu();
        matrix = linear_combination(matrix,Matrix::new(32,256,WEIGHT_2.iter().map(|&x| x)),Matrix::new(32,1,BIAS_2.iter().map(|&x| x)));
        matrix.relu();
        matrix = linear_combination(matrix,Matrix::new(10,32,WEIGHT_3.iter().map(|&x| x)),Matrix::new(10,1,BIAS_3.iter().map(|&x| x)));
        matrix.softmax();

        let mut index = 0; let mut maxi = matrix.data[0];
        for i in 1..10 {
            if matrix.data[i] > maxi {
                index = i;
                maxi = matrix.data[i];
            }
        }

        println!("number recognized is {} with value {}",index,matrix.data[index]);
    }
}
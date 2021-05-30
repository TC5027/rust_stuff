use rayon::prelude::*;
use std::iter::Sum;
use std::ops::{Add, Mul};

use crate::Matrix;

impl<T: Copy + Add<Output = T> + Mul<Output = T> + Sum + Send + Sync> Matrix<T> {
    pub fn par_linear_combination(&mut self, weights: &Matrix<T>, bias: &Matrix<T>) {
        assert!(self.nb_col == 1 && bias.nb_col == 1);
        assert!(self.nb_row == weights.nb_col && bias.nb_row == weights.nb_row);

        // we simply add into_par_iter() !
        self.data = (0..weights.nb_row)
            .into_par_iter()
            .map(|i| {
                self.data
                    .iter()
                    .enumerate()
                    .map(|(index, &x)| x * weights.data[i * weights.nb_col + index])
                    .sum::<T>()
                    + bias.data[i]
            })
            .collect();

        self.nb_row = bias.nb_row;
    }
}

impl Matrix<f64> {
    pub fn par_tanh(&mut self) {
        self.data.par_iter_mut().for_each(|x| *x = x.tanh());
    }

    pub fn par_relu(&mut self) {
        self.data.par_iter_mut().for_each(|x| *x = x.max(0.0));
    }

    pub fn par_softmax(&mut self) {
        let sum: f64 = self.data.par_iter().map(|&x| x.exp()).sum();
        self.data.par_iter_mut().for_each(|x| *x = x.exp() / sum);
    }
}

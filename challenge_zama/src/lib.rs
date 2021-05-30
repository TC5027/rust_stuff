use std::iter::Sum;
use std::ops::{Add, Mul};

mod advanced;

/// Matrix object, defined as a contiguous sequence of data
/// correclty interpreted with the nb_col and nb_row fields.
/// Accessing matrix[i,j] can be done looking at data[i*nb_col+j]
/// (with i and j starting at 0)
#[derive(PartialEq, Debug)]
pub struct Matrix<T> {
    pub data: Vec<T>,
    pub nb_col: usize,
    pub nb_row: usize,
}

impl<T: Copy> Matrix<T> {
    /// Creates a Matrix object, data being flattened row wise.
    /// nb_col and nb_row are expected to be > 0 and data length
    /// must be equal to nb_col and nb_row.
    pub fn new(nb_col: usize, nb_row: usize, data: &[T]) -> Self {
        assert_eq!(nb_col * nb_row, data.len());
        assert!(nb_col > 0 && nb_row > 0);

        let data: Vec<T> = data.iter().copied().collect();

        Matrix {
            data,
            nb_col,
            nb_row,
        }
    }

    /// Flatten a Matrix object, meaning the Matrix will have nb_row = 1
    pub fn flatten(&mut self) {
        // self.nb_col *= self.nb_row;
        // self.nb_row = 1;
        self.nb_row *= self.nb_col;
        self.nb_col = 1;
    }

    /// Out of place transposition of a Matrix object
    pub fn transpose(&mut self) {
        let nb_col = self.nb_row;
        let nb_row = self.nb_col;

        let mut new_data = Vec::new();
        for i in 0..nb_row {
            for j in 0..nb_col {
                new_data.push(self.data[j * self.nb_col + i]);
            }
        }

        self.nb_col = nb_col;
        self.nb_row = nb_row;
        self.data = new_data;
    }
}

impl<T: Copy + Add<Output = T> + Mul<Output = T> + Sum> Matrix<T> {
    /// Convolution by a kernel which is represented as
    /// another Matrix object, without strides and padding.
    pub fn convolution(&mut self, kernel: &Matrix<T>) {
        assert!(self.nb_col >= kernel.nb_col && self.nb_row >= kernel.nb_row);
        // we remember the previous number of columns
        // of self, to navigate correctly inside its
        // data field
        let previous_nb_col = self.nb_col;

        // we set the new dimensions
        self.nb_col -= kernel.nb_col - 1;
        self.nb_row -= kernel.nb_row - 1;

        self.data = (0usize..self.nb_col * self.nb_row)
            .map(|k| {
                // we go through each position of the result
                // matrix one by one
                let (i, j) = (k / self.nb_col, k % self.nb_col);
                // we compute the value at this position
                kernel
                    .data
                    .iter()
                    .enumerate()
                    .map(|(index, &x)| {
                        let (k_i, k_j) = (index / kernel.nb_col, index % kernel.nb_col);
                        x * self.data[(i + k_i) * previous_nb_col + (j + k_j)]
                    })
                    .sum()
            })
            .collect();
    }

    /// Linear combination by a weight (Matrix object) and
    /// a bias (flattened Matrix object) of a flattened self.
    pub fn linear_combination(&mut self, weights: &Matrix<T>, bias: &Matrix<T>) {
        // assert self and bias respect the dimension's
        // constraint of the method
        assert!(self.nb_col == 1 && bias.nb_col == 1);
        // assert the weights' dimensions match the ones of
        // self and bias
        assert!(self.nb_row == weights.nb_col && bias.nb_row == weights.nb_row);

        self.data = (0..weights.nb_row)
            .map(|i| {
                // we go through each position of the result
                // matrix one by one and compute its value
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
    /// Apply hyperbolic tangent on Matrix's data
    pub fn tanh(&mut self) {
        self.data.iter_mut().for_each(|x| *x = x.tanh());
    }

    /// Apply RELU on Matrix's data
    pub fn relu(&mut self) {
        self.data.iter_mut().for_each(|x| *x = x.max(0.0));
    }

    /// Apply softmax on Matrix's data
    pub fn softmax(&mut self) {
        let sum: f64 = self.data.iter().map(|&x| x.exp()).sum();
        self.data.iter_mut().for_each(|x| *x = x.exp() / sum);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_convolution() {
        let kernel = Matrix::new(3, 3, &vec![1, 0, 1, 0, 1, 0, 1, 0, 1]);
        let mut matrix = Matrix::new(
            7,
            7,
            &vec![
                0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0,
                0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            ],
        );

        let expected_output = Matrix::new(
            5,
            5,
            &vec![
                1, 4, 3, 4, 1, 1, 2, 4, 3, 3, 1, 2, 3, 4, 1, 1, 3, 3, 1, 1, 3, 3, 1, 1, 0,
            ],
        );

        matrix.convolution(&kernel);

        assert_eq!(matrix, expected_output);
    }

    #[test]
    fn test_linear_combination() {
        let mut one_dim = Matrix::new(1, 3, &vec![1, 1, 1]);
        let weights = Matrix::new(3, 2, &vec![1, 2, 3, 4, 5, 6]);
        let bias = Matrix::new(1, 2, &vec![-1, -2]);

        let expected_output = Matrix::new(1, 2, &vec![5, 13]);

        one_dim.linear_combination(&weights, &bias);
        assert_eq!(one_dim, expected_output);
    }

    #[test]
    fn activation_functions() {
        let mut matrix = Matrix::new(3, 1, &vec![3.0, -4.0, 0.0]);
        matrix.relu();

        let expected_output = Matrix::new(3, 1, &vec![3.0, 0.0, 0.0]);
        assert_eq!(matrix, expected_output);
    }

    #[test]
    fn test_transpose() {
        let mut matrix = Matrix::new(3, 2, &vec![1, 2, 3, 4, 5, 6]);
        matrix.transpose();

        let expected_output = Matrix::new(2, 3, &vec![1, 4, 2, 5, 3, 6]);
        assert_eq!(matrix, expected_output);
    }
}

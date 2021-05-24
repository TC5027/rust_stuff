use std::ops::{Add,Mul};
use std::iter::Sum;

/// Matrix object, defined as a contiguous sequence of data
/// correclty interpreted with the nb_col and nb_row fields.
/// Accessing matrix[i,j] can be done looking at data[i*nb_col+j]
/// (with i and j starting at 0)
#[derive(PartialEq,Debug)]
pub struct Matrix<T> {
    data : Vec<T>,
    nb_col : usize,
    nb_row : usize
}

impl<T> Matrix<T>  {
    /// Creates a Matrix object.
    /// We accept as input for the data anything that implements
    /// the trait IntoIterator.
    /// nb_col and nb_row are expected to be > 0 and data length
    /// must be equal to nb_col and nb_row.
    pub fn new (nb_col : usize, nb_row : usize, data : impl IntoIterator<Item=T>) -> Self {
        let data : Vec<T> = data.into_iter().collect();
        
        assert_eq!(nb_col*nb_row, data.len());
        assert!(nb_col > 0 && nb_row > 0);

        Matrix {
            data : data,
            nb_col : nb_col,
            nb_row : nb_row
        }
    }
}

/// Convolution operator
pub fn convolution<T>(matrix : Matrix<T>, kernel : Matrix<T>) -> Matrix<T> 
    where T : PartialEq + Add<Output=T> + Mul<Output=T> + Sum + Copy
{
    let nb_col = matrix.nb_col - kernel.nb_col + 1;
    let nb_row = matrix.nb_row - kernel.nb_row + 1;

    let data : Vec<T> = (0usize..nb_col*nb_row).map(|k| {
        let (i,j) = (k/nb_col,k%nb_col);
        let value = kernel.data.iter()
            .enumerate()
            .map(|(index,&k_value)| {
                let (k_i,k_j) = (index/kernel.nb_col,index%kernel.nb_col);
                k_value*matrix.data[(i+k_i)*matrix.nb_col + (j+k_j)]
            })
            .sum();
        value
    }).collect();
    
    Matrix::new(nb_col, nb_row, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn matrix_creation() {
        let mut rng = rand::thread_rng();

        let _matrix = Matrix::new(1,4,vec![1u8,2,3,4]);
        let _matrix = Matrix::new(3,4,(0..3*4).map(|_| rng.gen::<i32>()));
        let _matrix = Matrix::new(3,4,(0..3*4).map(|_| rng.gen::<u64>()));
        let _matrix = Matrix::new(3,4,(0..3*4).map(|_| rng.gen::<f64>()));
    }

    #[test]
    fn test_convolution() {
        let kernel = Matrix::new(3,3,vec![1,0,1,0,1,0,1,0,1]);
        let matrix = Matrix::new(7,7,vec![0,1,1,1,0,0,0,0,0,1,1,1,0,0,0,0,0,1,1,1,0,0,0,0,1,1,0,0,0,0,1,1,0,0,0,0,1,1,0,0,0,0,1,1,0,0,0,0,0]);

        let expected_output = Matrix::new(5,5,vec![1,4,3,4,1,1,2,4,3,3,1,2,3,4,1,1,3,3,1,1,3,3,1,1,0]);

        assert_eq!(expected_output,convolution(matrix,kernel));
    }
}

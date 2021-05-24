/// Matrix object, defined as a contiguous sequence of data
/// correclty interpreted with the nb_col and nb_row fields.
pub struct Matrix<T> {
    data : Vec<T>,
    nb_col : usize,
    nb_row : usize
}

impl<T> Matrix<T>  {
    /// Creates a Matrix object.
    /// We accept as input for the data anything that implements
    /// the trait IntoIterator.
    /// nb_col and nb_row are expected to be > 0.
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


#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn matrix_creation() {
        let mut rng = rand::thread_rng();

        let matrix = Matrix::new(1,4,vec![1u8,2,3,4]);
        let matrix = Matrix::new(3,4,(0..3*4).map(|_| rng.gen::<i32>()));
        let matrix = Matrix::new(3,4,(0..3*4).map(|_| rng.gen::<u64>()));
        let matrix = Matrix::new(3,4,(0..3*4).map(|_| rng.gen::<f64>()));
    }
}

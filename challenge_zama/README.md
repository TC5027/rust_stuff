# Challenge - ZAMA - 24/05

## Commands

To run the example : 
```
cargo run --example numbers
```

To run the tests : 
```
cargo test
```

To run the benchmark : 
```
cargo bench
```

The reports created by [Criterion](https://github.com/bheisler/criterion.rs) are located in **target/criterion/report/index.html**

## Example of use

Creating a Matrix is made through the ```new``` method. It takes as input :
* the number of columns of the matrix
* the number of rows of the matrix
* the data, represented flatten by rows

Suppose we want to create the following matrix : 

|1 2 3|
|4 5 6|

```rust
let matrix = Matrix::new(3,2,&vec![1,2,3,4,5,6]);
```
(if the matrix is given by columns we can use ```transpose```)

The library offers several functions used by neural networks like convolution, relu, softmax and more, which can be applied on Matrix instances :

```rust
let mut matrix = Matrix::new(3,3,&vec![1,2,3,4,5,6,7,8,9]);

let kernel = Matrix::new(2,2,&vec![1,1,1,1]);

matrix.convolution(&kernel);

matrix.relu();
```
// tuples can be used as function arguments and as return values

fn reverse(pair: (i32, bool)) -> (bool, i32) {
    let (int_param, bool_param) = pair;
    (bool_param, int_param)
}

// define struct Matrix with debug
#[derive(Debug)]
struct Matrix(f32, f32, f32, f32);

fn main() {
    // define tuple with different types
    let long_tuple = (1u8, 2u16, 3u32, 4u64, -1i8, -2i16, -3i32, -4i64, 0.1f32, 0.2f64, 'a', true);

    // print the first and second of the long_tuple
    println!("long tuple first {}", long_tuple.0);
    println!("long tuple second {}", long_tuple.1);

    // define tuple in tuple
    let tuple_of_tuples = ((1u8, 2u16, 2u32), (4u64, -1i8), -2i16);
    println!("tuple of tuples: {:?}", tuple_of_tuples);

    // Note that a tuple of more than 12 elements cannot be printed

    let pair = (1, true);
    println!("pair is {:?}", pair);

    // call the reverse function above
    println!("the reverse pair is {:?}", reverse(pair));

    // call matrix
    let matrix = Matrix(1.1, 1.2, 2.1, 2.2);
    println!("matrix {:?}", matrix);
}
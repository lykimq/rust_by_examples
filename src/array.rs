// Array and slice
// array knows the length at compile time
// Slice does not know the length at compile time, it is two words objects:
// first words is a pointer to the data
// second word is the length of the slice

use std::mem;

// type signature of slice is `&[T]`
fn analyze_slice(slice: &[i32]) {
    println!("first element of the slice: {}", slice[0]);
    println!("the slice has {} elements", slice.len());
}

fn main() {
    //fixed-size array
    let xs: [i32; 5] = [1, 2, 3, 4, 5];

    println!("first element of array: {}", xs[0]);
    println!("second element of array: {}", xs[1]);
    println!("number of elements in array: {}", xs.len());

    // array are stack allocated
    println!("array occupies {} bytes", mem::size_of_val(&xs));

    // array can be automatically borrowed as slices
    println!("borrow the whole array as a slice");
    analyze_slice(&xs);

    // all elements can be initialized to the same value
    let ys: [i32; 500] = [0; 500];

    // slices can point to a section of an array
    // [starting-index ... ending-index]
    // ending-index is one more than the last position in the slice
    println!("borrow a section of the array as a slice");
    analyze_slice(&ys[1..4]);

    // empty slice `&[]`
    let empty_array: [u32; 0] = [];
    assert_eq!(&empty_array, &[]);
    assert_eq!(&empty_array, &[][..]); // same but more verbose

    // Arrays can be safety accessing using `.get`, which returns
    // an `Option`.
    for i in 0..xs.len() + 1 {
        // + 1 is more than array length
        // will return None case
        match xs.get(i) {
            Some(xval) => println!("iterate {}: {}", i, xval),
            None => println!("Slow down! {} is too far", i),
        }
    }
}
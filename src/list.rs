// import the `fmt` module
use std::fmt;

// Define a structure name `List` containting a `Vec
struct List(Vec<i32>);

// reimplement the Display for List
impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let vec = &self.0;

        // open the [, ? return error
        write!(f, "[")?;

        // iterate over `v` in `vec` while enumerating the
        // iteration count in `count`
        for (count, v) in vec.iter().enumerate() {
            // For every element except the first,
            // add a comma
            // use the ? operator to return on errors.
            if count != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", v)?;
        }

        // Close the open bracket and return a fmt::Result value
        write!(f, "]")
    }
}

fn main() {
    let v = List(vec![1, 2, 3]);
    println!("{}", v);
}
// import fmt
use std::fmt;

// Derive the `fmt::Debug` implementation for Structure.
// `Structure` is a structure which contains a single `i32`

#[derive(Debug)]
struct Structure(i32);

// Put a `Structure` inside of the structure `Deep`. Make
// it printable also
#[derive(Debug)]
struct Deep(Structure);

// Use pretty print
// allow dead_code because struct will complaint that
// it is not used anywhere
#[allow(dead_code)]
#[derive(Debug)]
struct Person<'a> {
    name: &'a str,
    age: u8,
}

// declare another MinMax struct
#[derive(Debug)]
struct MinMax(i64, i64);

// Implement `Display` for MinMax
impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // - Use `self.number` to refer to each positional data point
        // - self is the name of the function, ie: MinMax
        // - write! is similar as println!
        // - Write strictly the first and second element of self
        // into the supplied output stream: `f`
        // - Return `fmt::Result` which indicates whether the operation
        // succeeded or failed.
        write!(f, "({}, {})", self.0, self.1)
    }
}

// Define structure where the fields are nameable for comparison
#[derive(Debug)]
struct Point2D {
    x: f64,
    y: f64,
}

// implement Display for Point2D
impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted
        write!(f, "x : {}, y : {}", self.x, self.y)
    }
}

fn main() {
    // Print debug with {:?}
    println!("{:?} months of the year", 12);

    let name = "Peter";
    let age = 27;
    let peter = Person { name, age };
    // Pretty print with {:#?}
    println!("{:#?}", peter);

    // call MinMax
    let minmax = MinMax(0, 14);
    println!("Compare structures:");
    println!("Display: {}", minmax);
    println!("Display debug: {:?}", minmax);

    // call point2D
    let point = Point2D { x: 3.3, y: 7.2 };
    println!("Compare points:");
    println!("Display: {}", point);
    println!("Display debug: {:?}", point);
}
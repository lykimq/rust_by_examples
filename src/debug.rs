// import fmt, using specified formatting trait
use std::fmt::{ self, Formatter };

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

// Define structure city

struct City {
    name: &'static str,
    // Latitude
    lat: f32,
    // Longitude
    lon: f32,
}

// implement display for City
impl fmt::Display for City {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let lat_c = if self.lat >= 0.0 { 'N' } else { 'S' };
        let lon_c = if self.lon >= 0.0 { 'E' } else { 'W' };

        // write! is like println!, but it will write the formatter
        // string into a buffer (the first argument)
        write!(
            f,
            "{}: {:.3}c {} {:.3}c {}",
            self.name,
            self.lat.abs(),
            lat_c,
            self.lon.abs(),
            lon_c
        )
    }
}

// Define struct Color as debug display
#[allow(dead_code)]
#[derive(Debug)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
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

    // call City
    for city in [
        City { name: "Dublin", lat: 53.347778, lon: -6.259722 },
        City { name: "Oslo", lat: 59.95, lon: 10.75 },
        City { name: "Vancover", lat: 49.25, lon: -123.1 },
    ].iter() {
        println!("{}", *city);
    }

    // call Color
    for color in [
        Color { red: 128, green: 255, blue: 90 },
        Color { red: 0, green: 3, blue: 254 },
        Color { red: 0, green: 0, blue: 0 },
    ].iter() {
        // :? for debug
        println!("{:?}", *color);
    }
}
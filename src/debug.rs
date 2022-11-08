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

fn main() {
    // Print debug with {:?}
    println!("{:?} months of the year", 12);

    let name = "Peter";
    let age = 27;
    let peter = Person { name, age };
    // Pretty print with {:#?}
    println!("{:#?}", peter);
}
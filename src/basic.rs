// Basic about Rust by examples
// 1. Hello world

fn main() {
    // Print out Hello World
    println!("Hello World!");

    /*
    Print variable x
     */
    let x = 5 + 5;
    println!("x = {}", x);

    // Print with positional arguments
    println!("This is position 0 {0}, this is position 1 {1}", "Alice", "Bob");

    // print with argument
    println!(
        "{subject} {verb} {object}",
        subject = "The quick",
        verb = "jumps over",
        object = "the dog"
    );

    // Print with different format character
    println!("Base 10: {}", 1234);
    println!("Base 2 - binary: {:b}", 1234);
    println!("Base 8 - octal:  {:o}", 1234);
    println!("Base 16 - hexadecimal: {:x}", 1234);
    println!("Base 16 - hexadecimal: {:X}", 1234);

    // Print with space width before
    println!("Add space: {number:>5}", number = 1);

    // Print with space and add arbiry number after
    println!("Add 0: {number:0<5}", number = 1);

    // Same as above but use argument
    println!("Argument: {number:0<width$}", number = 1, width = 5);

    // declare dead_code, it will not compile
    #[allow(dead_code)]
    struct Structure(i32);
    // This will not compile because Structure does not implement fmt::Display
    // println!("This struct {} won't print", Struct(3));
}
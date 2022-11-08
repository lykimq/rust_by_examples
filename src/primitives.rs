fn main() {
    let logical: bool = true;

    // regular annotation
    let a_float: f64 = 1.0;

    // suffix annotation
    let an_integer = 5i32;

    // default
    let default_float = 3.0; // f64
    let default_integer = 7; // i32

    // A type can be inferred from context
    let mut inferred_type = 12;
    inferred_type = 4294967296i64;

    // mutable variable
    let mut mutable = 12;
    mutable = 21;

    // variable can be overwritten with shadowing
    let mutable = true;
}
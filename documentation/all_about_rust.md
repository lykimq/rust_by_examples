# RUST knowledge

## Language Fundamentals

### Ownership
Ownership is one of the key concepts in Rust that ensures memory safety without the need for garbage collection. The basic idea is that each piece of memory has a single owner, and the owner is responsible for cleaning up the memory when it is no longer needed. This helps prevent data races and memory leaks.

```rust
fn main(){
    // Ownership example
    let s1 = String::from("Hello"); // s1 is the owner of the String "Hello"
    let s2 = s1; // Ownership transferred from s1 to s2, s1 now is invalidated

    // This would cause a compilation error, as s1 is no longer valid
    // println!("s1: {}", s1);

    // s2 is now the owner of the String "Hello"
    println!("s2; {}", s2);
}

```

### Borrowing
Borrowing allows you to temporarily take a reference to a value without taking ownership. Borrowing can be either mutable or immutable. Mutable borrowing means you can change the value, but only one mutable reference is allowed at a time to prevent data races. Immutable borrowing allows multiple references but does not allow modification.

```rust
fn main(){
    // Borrowing example

    let s1 = String::from("Hello");

    // Immutable borrowing
    let len = calculate_length(&s1);
    println!("Length of '{}' is {}.", s1.len);

    // Mutable borrowing
    let mut s2 = String::from("Word");
    change_string(&mut s2);
    println!("Modified string: {}", s2);

    // Immutable borrowing (reference to String)
    fn calculate_length(s: &String) -> usize {
        s.len()
    }

    // Mutable borrowing (mutable reference to String)
    fn change_string(s: &mut String){
        s.push_str(", Rust!");
    }
}
```

### Lifetimes

Lifetimes in Rust specify how long references are valid. They ensure that references don't outlive the data they point to, preventing dangling references. Lifetime are denoted by apostrophes ('), and theyy are a way to express the scope for which a reference is valid.

```rust

fn main(){
    // Lifetime example
    let string1 = String::from("Hello");
    let result;

    {
        let string2 = String::from("World");
        result = longest_string(&string1, &string2); // Lifetime of result tied to the smaller of string1 and string2
    }

    // This would cause a compilation error, as string2 is no longer valid
    // println!("Longest string: {}", result);

    // Lifetime of result extends only to this point
    println!("Longest string: {}", result);

    // Function with lifetime specifying the scope of references
    fn longest_string<'a>(s1: &'a str, s2: &'a str) -> &'a str {
        if s1.len() > s2.len {
            s1
        } else {
            s2
        }
    }
}
```

In the `longest_string` function, the `'a` is a lifetime parameter indicating that the references `s1` and `s2` must have the same lifetime, and the result reference will also have the same lifetime as the input references.

## Concurrency and Parallelism

## Ownership in Concurrent Programming
Concurrency in Rust is achieved through a combination of ownership, borrowing, and lifetimes, allowing safe parallel execution of code. Rust's ownership system helps prevent data races by enforcing strict rules at compile-time. The ownership model ensures that data cannot be accessed concurrently if it is mutable, preventing race conditions and data corruption.

```rust
use std::thread;

fn main() {
    let mut data = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        data.push(4);
        // Ownership of `data` is transferred to the spawned thread
        // Attempting to access `data` in the main thread would cause a compilation error
    });

    handle.join().unwrap(); // Wait for the spawned thread to finish

    // This would cause a compilation error, as `data` is no longer valid in the main thread
    // data.push(5);
}
```

In this example, ownership of the data vector is moved to the spawned thread, ensuring that the main thread cannot access or modify it concurrently.

### Send and Sync Traits
The Send and Sync traits in Rust are markers that indicate whether a type can be safely transferred between threads (Send) or shared between threads (Sync). Types that implement Send can be moved between threads, while types that implement Sync can be shared between threads without causing data races.

```rust
use std::thread;

// `Send` trait example
fn send_example() {
    let data = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("{:?}", data);
    });
    handle.join().unwrap();
}

// `Sync` trait example
fn sync_example() {
    let counter = std::sync::Mutex::new(0);

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(move || {
                let mut data = counter.lock().unwrap();
                *data += 1;
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
```
In the send_example, the data vector is moved to the spawned thread, leveraging the Send trait. In the sync_example, a Mutex is used to safely share mutable data (counter) between threads, demonstrating the use of the Sync trait.

#### std::thread and std::sync
Rust provides the std::thread module for creating and managing threads. The std::sync module includes synchronization primitives like Mutex and Arc that allow safe sharing of data between threads.

```rust
use std::thread;
use std::sync::{Mutex, Arc};

fn main() {
    // Creating a thread with std::thread
    let handle = thread::spawn(|| {
        println!("Hello from a thread!");
    });

    handle.join().unwrap(); // Wait for the thread to finish

    // Using Mutex for thread-safe data access
    let counter = Arc::new(Mutex::new(0));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                let mut data = counter.lock().unwrap();
                *data += 1;
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
```
In this example, a thread is created using std::thread::spawn. The second part demonstrates using a Mutex wrapped in an Arc (atomic reference counting) to safely share and update a counter value between multiple threads.

### Async/Await
Asynchronous programming in Rust is facilitated by the async/await syntax. Asynchronous code allows for non-blocking execution, enabling concurrent tasks to progress independently without blocking the main thread.

```rust
async fn async_function() {
    // Asynchronous code using async/await
    let result = async {
        // Asynchronous tasks go here
        "Hello from async/await!"
    };

    println!("{}", result.await);
}

#[tokio::main]
async fn main() {
    // Using the Tokio runtime for async execution
    async_function().await;
}
```
In this example, the async_function showcases the use of async/await for asynchronous programming. The main function, marked with #[tokio::main], uses the Tokio runtime to execute asynchronous tasks.

### Popular Asynchronous Libraries
Libraries like Tokio and async-std provide abstractions and utilities for building concurrent and scalable systems in Rust. They offer tools for managing asynchronous tasks, handling I/O efficiently, and building concurrent applications.

```rust
// Using Tokio for asynchronous programming
#[tokio::main]
async fn main() {
    let result = tokio::spawn(async {
        // Asynchronous tasks using Tokio
        "Hello from Tokio!"
    }).await.unwrap();

    println!("{}", result);
}
```

In this example, Tokio's `tokio::spawn` function is used to execute an asynchronous task concurrently. The result is awaited, and the asynchronous code prints a message when complete.


## Error Handling

### Result and Option Types:

- `Result`: The `Resul`` type is used for functions that can return either a successful value (`Ok`) or an error (`Err`). It is a way to handle errors in a structured manner.
- `Option`: Similar to `Result`, `Option` is used for cases where a value might be absent. It can be either `Some` with a value or `None` indicating absence.

```rust
fn main() {
    // Result example
    let result: Result<i32, &str> = divide(10, 2);
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(error) => println!("Error: {}", error),
    }

    // Option example
    let option_value: Option<i32> = find_element(vec![1, 2, 3], 2);
    match option_value {
        Some(value) => println!("Found: {}", value),
        None => println!("Not found"),
    }
}

fn divide(x: i32, y: i32) -> Result<i32, &str> {
    if y == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(x / y)
    }
}

fn find_element(vec: Vec<i32>, target: i32) -> Option<i32> {
    for &value in &vec {
        if value == target {
            return Some(value);
        }
    }
    None
}
```

In this example, the divide function returns a Result indicating whether the division was successful, and the find_element function returns an Option indicating whether the target element was found in the vector.

## Trait System and Generics
### Generic Functions and Structs

- `Generics`: Rust allows you to write generic functions and structs that work with multiple types. Generic code promotes code reuse and flexibility.
- `Traits`: Traits define shared behavior that types can implement. They allow for code abstraction and polymorphism.

```rust
// Generic function
fn print_twice<T>(value: T) where T: std::fmt::Debug {
    println!("{:?}", value);
    println!("{:?}", value);
}

// Generic struct
struct Point<T> {
    x: T,
    y: T,
}

// Trait example
trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

fn main() {
    // Generic function usage
    print_twice("Hello");
    print_twice(42);

    // Generic struct usage
    let point_int = Point { x: 1, y: 2 };
    let point_float = Point { x: 1.5, y: 2.5 };

    // Trait usage
    let circle = Circle { radius: 2.0 };
    println!("Circle area: {}", circle.area());
}
```

In this example, the print_twice function and Point struct are generic, allowing them to work with different types. The Shape trait is implemented for the Circle type, demonstrating trait usage.

## Memory Safety and Performance

## Ownership for Memory Safety

- `Ownership`: Rust's ownership system ensures memory safety without a garbage collector by tracking ownership of data and enforcing strict borrowing rules.
- `Smart Pointers`: Smart pointers like `Rc`, `Arc`, `Cell`, and `RefCell` provide additional flexibility and control over ownership and mutability.

```rust
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    // Ownership example
    let s1 = String::from("Hello");
    let s2 = s1; // Ownership moved to s2, s1 is no longer valid

    // This would cause a compilation error
    // println!("s1: {}", s1);

    // Smart pointer example (Rc)
    let data = Rc::new(vec![1, 2, 3]);
    let clone1 = Rc::clone(&data);
    let clone2 = Rc::clone(&data);
    // Rc allows multiple references to the same data with shared ownership

    // Mutable borrowing with RefCell
    let counter = RefCell::new(0);
    *counter.borrow_mut() += 1;
    // RefCell allows interior mutability with runtime checks
}
```

In this example, ownership is demonstrated with the String type, and smart pointers (Rc) and interior mutability (RefCell) showcase ways to achieve flexibility while maintaining memory safety.

## Testing and Documentation

### Unit and Integration Tests

- `Unit Tests`: Verify individual components or functions in isolation.
- `Integration Tests`: Test the interaction between multiple components or the entire system.

```rust
// Code to be tested

fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }
}

// Integration test (in a separate file)
// File: tests/integration_tests.rs

// Import the code to be tested
use my_project::add;

#[test]
fn test_add_integration() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, 1), 0);
```

In this example, the add function is tested with both unit tests (within the same file) and integration tests (in a separate file).

### Cargo Test

- `cargo test` is a command-line tool in Rust for running tests.
- It automatically discovers and runs unit tests and integration `tests` in the tests directory.

```rust
# Run tests
cargo test
```

## Cargo and Dependency Management

### Cargo commands:

- Cargo is Rust's package manager and build tool.
- Common Cargo commands include `build, run, test, doc, update`, and `publish`.

```rust
# Build the project
cargo build

# Run the project
cargo run

# Run tests
cargo test

# Generate documentation
cargo doc

# Update dependencies
cargo update

# Publish the package to crates.io
cargo publish
```

### Cargo.toml

- `Cargo.toml` is the configuration file for Rust projects.
- It specifies project metadata, dependencies, and build configurations.

```rust
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
rand = "0.8.5"
serde = { version = "1.0", features = ["derive"] }
```

### Semantic Versioning (SemVer)

- Semantic Versioning (SemVer) is a versioning scheme that specifies how versions should be incremented based on changes.
- Versions are in the format MAJOR.MINOR.PATCH.

```rust
[dependencies]
my_library = "0.1.2"
```

In this example, the version "0.1.2" follows the SemVer format.

## Advanced Concepts

### Procedural Macros:

- Proced ural macros are a way to extend the Rust compiler with custom syntax extensions.
- They allow for code generation during compile-time.

```rust
// Procedural macro example
use my_macro::custom_macro;

#[custom_macro]
fn my_function() {
    // Custom code generated by the procedural macro
    println!("Hello from custom_macro!");
}

fn main() {
    my_function();
}
```

In this example, the custom_macro procedural macro generates custom code for the my_function during compile-time.

### Trait Objects and Dynamic Dispatch

- Trait objects allow for working with values of different types that implement the same trait.
- Dynamic dispatch enables calling trait methods on trait objects at runtime.

```rust
// Trait and struct example
trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

fn print_area(shape: &dyn Shape) {
    // Dynamic dispatch using trait objects
    println!("Area: {}", shape.area());
}

fn main() {
    let circle = Circle { radius: 2.0 };
    print_area(&circle);
}
```

In this example, the Shape trait is implemented for the Circle struct, and dynamic dispatch is demonstrated by calling the area method on a trait object.


### Patterns and Anti-Patterns

- Patterns in Rust are idiomatic ways of solving common problems efficiently.
- Anti-patterns are practices that should be avoided.

```rust
// Pattern: Builder pattern
struct PersonBuilder {
    name: Option<String>,
    age: Option<u32>,
}

impl PersonBuilder {
    fn new() -> Self {
        Self {
            name: None,
            age: None,
        }
    }

    fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }

    fn build(self) -> Person {
        Person {
            name: self.name.unwrap_or_else(|| String::from("Unknown")),
            age: self.age.unwrap_or_default(),
        }
    }
}

struct Person {
    name: String,
    age: u32,
}

fn main() {
    // Using the Builder pattern
    let person = PersonBuilder::new()
        .name("Alice")
        .age(30)
        .build();
    println!("{:?}", person);
}
```

In this example, the Builder pattern is used to create a Person struct with optional fields. This pattern improves readability and allows for flexible construction.

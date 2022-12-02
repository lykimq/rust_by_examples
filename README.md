# rust_by_examples

From https://doc.rust-lang.org/stable/rust-by-example/hello.html

## Knowledge

### Types

#### Box
Box is a pointer type for heap allocation.

```rust
// Allocates memory on the heap and then places 'x' into it
let five = Box::new(5)
```

#### Unsized array
- `[u8]`: represents raw unsized array of `u8` somewhere in memory. As an "unsized" type, we can't store it in variables nor pass it to functions, so it is not very useful on its own. Its primarily use is to create slice references and generic typs.
- `&[u8]`: is a reference to an array of `u8`. The reference is represented by a "fat pointer" two machine words wide, consisting of pointer to the data and the length of the data. It is the basic for `&str`.
- `Box<[u8]>`: is like `&[u8]`, except it owns the array, i.e the array is heap-allocated in the constructor and deallocation on `Drop`.
- `Vec<[u8]>`: is like `Box<[u8]>`, except it additionally stores a "capacity" count, making it 3 machine words wide. Separately stored capacity allows for efficient resizing of the underlying array. It is the basic for `String`.
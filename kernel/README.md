# Write a kernel to compile with WASM PVM in SCORU

## Example

### Kernel - wasm program

#### Hello world kernel
- `hello.wat`:
Write a hello word in `.wat` and use `wat2wasm` compiled it to `.wasm`. 

Compile
```
~/wabt/build/wat2wasm hello.wat -o hello.wasm
```

The contents of `hello.wasm` is a valid WASM kernel.

#### Write a dummy kernel in Rust 
- Use `rustup` enabling WASM as a compilation target by using this command:

```
rustup target add wasm32-unknown-unknown
```

- To make sure the use of `target` optional, create a new file in `.cargo/config.toml`

```
   [build]
   target = "wasm32-unknown-unknown"

   [rust]
   lld = true%
```

- Create a rust programm for dummy kernel name it `noop`

```
cargo new noop
```

- Modify `Cargo.toml` to make this library consumed by a kernel WASM crate

```
[lib]
   crate-type = ["cdylib", "rlib"]
```

- Compile `noop` kernel, notice that the `main.rs` needs to be rename as `lib.rs` to make it compiled
```
cargo build --target wasm32-unknown-unknown
```


# Write a kernel to compile with WASM PVM in SCORU

## Examples

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

after successed build, the `noop.wasm` is stored at:
```
../kernel/noop/target/wasm32-unknown-unknown/noop.wasm
```

#### Test the wasm kernel

Now we have a valid kernel `noop.wasm` we can use the `octez-wasm-repl` tool to test it. This tool helps to test the kernels during its development, without replying on starting a rollup on a test network.


```
  octez-wasm-repl ${WASM_FILE} --inputs ${JSON_INPUTS} --rollup ${ROLLUP_ADDRESS}
```
- It takes a `.wasm` file or `.wat`.
- This tool will parses and typechecks the kernel before giving it the PVM.
- It can take a file containing inputs json. A valid `JSON_INPUTS` may look like:
```
[
  { "payload" : { "int" : "0" },
   "sender" : "KT1ThEdxfUcWUwqsdergy3QnbCWGHSUHeHJq",
   "source" : "tz1RjtZUVeLhADFHDL8UwDZA6vjWWhojpu5w",
   "destination" : "scr1HLXM32GacPNDrhHDLAssZG88eWqCUbyLF"
  },
  { "payload" : { "prim" : "False" },
   "sender" : "KT1ThEdxfUcWUwqsdergy3QnbCWGHSUHeHJq",
   "source" : "tz1RjtZUVeLhADFHDL8UwDZA6vjWWhojpu5w"
  }
]
```
noticed that the `payload` can change directly to Micheline instead of its JSON representation.
- and a rollup address. 

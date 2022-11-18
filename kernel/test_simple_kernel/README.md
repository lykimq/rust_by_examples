# Run example of tx-kernel

## Test simple kernel

### Kernel entrypoint
- In this simple kernel `kernel_entry_simpl`, create a `kernel_next` where it creates a panic handler,  and a new `wasm` runtime.

```
cargo new kernel_entry_simpl
```

### Test cases

#### First test
- Create a test call the `kernel_entry_simpl`, in this kernel, it calls the `host.read_input` and will write the output and log. This test will call the `kernel_entry_simpl`

```
cargo new test_simple_kernel
```

#### Second test

- Create a test call the `kernel_entry_simpl`, in this kernel, it calls the `test_tx_kernel` (todo)


### Run

```
 cargo build --target wasm32-unknown-unknown
```

it will create a `test_tx_kernel.wasm` in the folder: `target/wasm32-unknown-unknown/debug/`

- This test is a short version of a reference of the [test_kernel](https://gitlab.com/tezos/kernel/-/tree/main/test_kernel) run this command:

```
cargo build -p test_kernel --release --target wasm32-unknown-unknown
```

## Reference
This document is a short description of how to run [the kernel](https://gitlab.com/tezos/kernel/-/tree/main/): https://hackmd.io/C8lR2snvTr2Mvp2nzIUgMQ
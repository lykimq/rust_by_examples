# Run example of tx-kernel

- Run

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
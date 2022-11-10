(module 
    (import "rollup_safe_core" "write_debug"
        (func $write_debug (param i32 i32) (result i32)))
    (memory 1)
    (export "mem" (memory 0))
    (data (i32.const 100) "hello, world!")
    (func (export "kernel_next")
        (local $hello_address i32)
        (local $hello_length i32)
        (local.set $hello_address (i32.const 100))
        (local.set $hello_length (i32.const 13))
        (drop (call $write_debug (local.get $hello_address)
                                 (local.get $hello_length)))))

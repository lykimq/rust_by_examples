// Set panic hook
#[cfg(feature = "panic-hook")]

/*
  The panic hook is invoked when a thread panics, but before the panic runtime
  is invoked. 
 */
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(panic_handler::panic_handler))
}

extern crate alloc;

/* Define the `kernel_next` to be used in the test_tx_kernel */
#[macro_export]
macro_rules! kernel_entry_simpl {
    ($kernel_next:expr) => {
        /* This `kernel_next` function is called by the wasm host at
           regular interval */
        #[cfg (target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" kernel_next(){
            // set a new panic hook for the kernel
            #[cfg(feature="panic-hook")]
            kernel::set_panic_hook();
            // Create a new reference to the wasm runtime
            let mut host = unsafe{host::wasm_host::WasmHost::new()};
            $kernel_next(&mut host)
        }
    };
}
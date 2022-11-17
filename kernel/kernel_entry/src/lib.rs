//! Contains entrypoint of the SCORU wasm kernel.
//!
//! A kernel must expose a `fn kernel_next();` entrypoint, which is called on a loop
//! by the runtime.  The kernel *yields* to the runtime by returning out of
//! `kernel_next`.
//!
//! There is a limit on how many computation ticks a kernel may perform per entry. It is
//! called a number of times per non-empty level.  The kernel must take care not to perform
//! arbitrarily long computations, to avoid breaching the computation limit.
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![allow(unused_variables)]
#![allow(dead_code)]

#[cfg(feature = "dlmalloc")]
mod allocator {
    use dlmalloc::GlobalDlmalloc;

    #[global_allocator]
    static ALLOCATOR: GlobalDlmalloc = GlobalDlmalloc;
}
#[cfg(feature = "wee_alloc")]
mod allocator {
    #[global_allocator]
    static ALLOCATOR: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
}

/// Set panic hook
#[cfg(feature = "panic-hook")]
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(panic_handler::panic_handler));
}

extern crate alloc;

/// Derive `kernel_next` & `mock_kernel_next` entrypoints.
///
/// ```should_panic
/// # extern crate alloc;
/// #[macro_use] extern crate kernel;
/// #[macro_use] extern crate debug;
///
/// use host::rollup_core::RawRollupCore;
///
/// fn kernel_run<Host: RawRollupCore>(_host: &mut Host) {
///   debug_msg!(Host, "Hello: {}", "Kernel!");
/// }
///
/// kernel_entry!(kernel_run);
///
/// loop {
///   kernel_next();
/// }
/// ```
#[macro_export]
macro_rules! kernel_entry {
    ($kernel_next: expr) => {
        /// The `kernel_next` function is called by the wasm host at regular intervals.
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn kernel_next() {
            #[cfg(feature = "panic-hook")]
            kernel::set_panic_hook();
            let mut host = unsafe { host::wasm_host::WasmHost::new() };
            $kernel_next(&mut host)
        }

        /// The `kernel_next` function is called by the wasm host at regular intervals.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn kernel_next() {
            panic!(
                "kernel_next is only supported on 'target = \"wasm32\"', \
                 use mock_kernel_next instead"
            );
        }

        /// The `mock_kernel_next` is called by the mock host at regular intervals.
        #[cfg(not(target_arch = "wasm32"))]
        pub fn mock_kernel_next(host: &mut mock_runtime::host::MockHost) {
            #[cfg(feature = "panic-hook")]
            kernel::set_panic_hook();

            $kernel_next(host)
        }
    };
}

//! Implementation of [RawRollupCore] that used when compiling to **wasm**.
use crate::rollup_core::{self, Input, RawRollupCore, ValueType, WriteResult};

/// The runtime host when running in `wasm` rollup.
///
/// # Safety
/// The only way to create an instance of `WasmHost` is to call [`WasmHost::new`], which
/// itself is *unsafe* to call. This is done to enforce the invariant that a kernel only
/// ever holds *one* reference of its *runtime*.
///
/// Therefore, `WasmHost` **does not** implement `Debug`, `Copy` or `Clone`. This prevents
/// the `kernel` from attempting to either create a new host, or to store the reference to
/// it within its *cache*.
pub struct WasmHost {}

impl WasmHost {
    /// Create a new reference to the wasm runtime.
    ///
    /// # Safety
    /// **Must** only ever be called once per *kernel entry*. Multiple
    /// instances of `WasmHost` may conflict with each other - breaking invariants
    /// elsewhere in this crate which make assumptions about the behaviour of the runtime.
    pub unsafe fn new() -> Self {
        Self {}
    }
}

unsafe impl RawRollupCore for WasmHost {
    unsafe fn read_input(
        &self,
        r#type: *mut Input,
        level: *mut i32,
        id: *mut i32,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize {
        rollup_core::read_input(r#type, level, id, dst, max_bytes)
    }

    unsafe fn write_output(&self, src: *const u8, num_bytes: usize) -> WriteResult {
        rollup_core::write_output(src, num_bytes)
    }

    unsafe fn write_debug(src: *const u8, num_bytes: usize) {
        rollup_core::write_debug(src, num_bytes)
    }

    unsafe fn store_has(&self, path: *const u8, path_len: usize) -> ValueType {
        rollup_core::store_has(path, path_len)
    }

    unsafe fn store_read(
        &self,
        path: *const u8,
        path_len: usize,
        offset: usize,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize {
        rollup_core::store_read(path, path_len, offset, dst, max_bytes)
    }

    unsafe fn store_write(
        &self,
        path: *const u8,
        path_len: usize,
        offset: usize,
        src: *const u8,
        num_bytes: usize,
    ) -> WriteResult {
        rollup_core::store_write(path, path_len, offset, src, num_bytes)
    }

    unsafe fn store_delete(&self, path: *const u8, len: usize) {
        rollup_core::store_delete(path, len)
    }

    unsafe fn store_list_size(&self, path: *const u8, path_len: usize) -> i64 {
        rollup_core::store_list_size(path, path_len)
    }

    unsafe fn store_list_get(
        &self,
        path: *const u8,
        path_len: usize,
        index: i64,
        dst: *mut u8,
        max_size: usize,
    ) -> usize {
        rollup_core::store_list_get(path, path_len, index, dst, max_size)
    }

    unsafe fn store_move(
        &self,
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    ) {
        rollup_core::store_move(from_path, from_path_len, to_path, to_path_len)
    }

    unsafe fn store_copy(
        &self,
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    ) {
        rollup_core::store_copy(from_path, from_path_len, to_path, to_path_len)
    }

    unsafe fn reveal_preimage(
        &self,
        hash_addr: *const u8,
        destination_addr: *mut u8,
        max_bytes: usize,
    ) -> usize {
        rollup_core::reveal_preimage(hash_addr, destination_addr, max_bytes)
    }
}

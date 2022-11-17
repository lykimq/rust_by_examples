//! Defines the *raw* bindings to the **rollup_safe_core** host module.
//!
//! These can be accessed by a kernel running in safe mode - which prevents the
//! kernel messing up the state tree w.r.t. hardware gas limits, and inputs.

/// The maximum size of input that can be read in one go from a slot message.
pub const MAX_INPUT_SLOT_DATA_CHUNK_SIZE: usize = 4096;

/// The maximum size of input that can be read in one go from a Layer 1 message.
pub const MAX_INPUT_MESSAGE_SIZE: usize = 4096;

/// The maximum size that may be written to `output` in one go.
pub const MAX_OUTPUT_SIZE: usize = 4096;

/// The maximum size that may be written to, or read from, disk in one go.
pub const MAX_FILE_CHUNK_SIZE: usize = 4096;

/// The size of a preimage hash in bytes.
pub const PREIMAGE_HASH_SIZE: usize = 32;

/// Defines whether the given input came from a **Slot**, or a **Layer1 message**.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Input {
    /// An input which came from a **Layer 1** message.
    MessageData,
    /// An input which is part of a **Layer 2** slot.
    SlotDataChunk,
}

/// Returned by 'output' APIs, which have limits on the amount
/// that may be written.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum WriteResult {
    /// The output bytes have been written to output in full.
    Ok,
    /// The output bytes had a size greater than that allowed by the called function.
    TooLarge,
}

/// Returned by [`store_has`] - specifies whether a path has a value, and/or is a prefix.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// The path has no matching value or subtree.
    None,
    /// The path has a value, but is not a prefix to further values.
    Value,
    /// The path is a prefix to further values, but has no value.
    Subtree,
    /// The path has a value, and is a prefix to further values.
    ValueWithSubtree,
}

#[link(wasm_import_module = "rollup_safe_core")]
extern "C" {
    /// If `/input/consumed >= /input/bytes`, return `0`.
    ///
    /// Otherwise:
    /// - Fills the given buffer with up to `max_bytes` and returns actual
    ///   number written.
    /// - Write the current value of `/input/{type,level,id}` to
    ///   `{type,level,id}`.
    pub fn read_input(
        r#type: *mut Input,
        level: *mut i32,
        id: *mut i32,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize;

    /// Write the given number of bytes to output.
    ///
    /// Fails with [WriteResult::TooLarge] if output size is greater than
    /// [MAX_OUTPUT_SIZE].
    pub fn write_output(src: *const u8, num_bytes: usize) -> WriteResult;

    /// Write the given number of bytes to the debug log.
    pub fn write_debug(src: *const u8, num_bytes: usize);

    /// Return whether the given key exists.
    pub fn store_has(path: *const u8, path_len: usize) -> ValueType;

    /// Read up to `num_bytes` bytes from the given key into memory.
    ///
    /// Returns the number of bytes copied to memory.  The bytes read from storage begin
    /// at `offset`.
    pub fn store_read(
        path: *const u8,
        path_len: usize,
        offset: usize,
        dst: *mut u8,
        num_bytes: usize,
    ) -> usize;

    /// Write the given number of bytes from memory to the given key, starting at `offset`.
    ///
    /// Returns [WriteResult::TooLarge] if output size is greater than
    /// [MAX_FILE_CHUNK_SIZE].
    pub fn store_write(
        path: *const u8,
        path_len: usize,
        offset: usize,
        src: *const u8,
        num_bytes: usize,
    ) -> WriteResult;

    /// Delete the given key.
    pub fn store_delete(path: *const u8, path_len: usize);

    /// Get the number of subkeys by prefix.
    pub fn store_list_size(path: *const u8, path_len: usize) -> i64;

    /// Get subkey path at `index` with a given prefix.
    ///
    /// Writes the encoded path of the subkey given by `index` - minus the
    /// *prefix* `path` - to `dst`.  Returns the size of the subkey path in
    /// bytes.
    ///
    /// It can be used together with `store_list_size` to *enumerate* subkeys.
    ///
    /// # Examples
    ///
    /// If the set of keys is `{/a/x, /a/y/z, /b/x}`, then:
    /// ```no_run
    /// # use host::rollup_core::{store_list_size, store_list_get};
    /// # use std::slice::from_raw_parts;
    ///
    /// let prefix = [b'/', b'a'];
    /// let path = prefix.as_ptr();
    /// let len = prefix.len();
    ///
    /// assert_eq!(2, unsafe { store_list_size(path, len) });
    ///
    /// let first_subkey = std::ptr::null_mut();
    /// let second_subkey = std::ptr::null_mut();
    ///
    /// let first_size = unsafe { store_list_get(path, len, 0, first_subkey, 1024) };
    /// let second_size = unsafe { store_list_get(path, len, 1, second_subkey, 1024) };
    ///
    /// let first_slice = unsafe { from_raw_parts(first_subkey, first_size) };
    /// let second_slice = unsafe { from_raw_parts(second_subkey, second_size) };
    ///
    /// assert_eq!([b'/', b'x'], first_slice);
    /// assert_eq!([b'/', b'y', b'/', b'z'], second_slice);
    /// ```
    pub fn store_list_get(
        path: *const u8,
        path_len: usize,
        index: i64,
        dst: *mut u8,
        max_size: usize,
    ) -> usize;

    /// Move a location in the store from one key to another
    /// Reboot if the key doesn't exist.
    /// Overwrites the destination, if it already exists
    ///
    /// e.g. if we have a store with `/x/y/z` containing a value
    /// `store_move /x/y /a/b`
    /// results in a store with `/a/b/z` as a location with a value
    /// (location `/x/y/z` will hold no value after the call to
    /// `store_move`).
    pub fn store_move(
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    );

    /// Copy a location in the store from one key to another
    /// Reboot if the key doesn't exist.
    /// Overwrites the destination, if it already exists.
    ///
    /// e.g. if we have a store with `/x/y/z` containing a value
    /// `store_copy /x/y /a/b`
    /// results in a store with `/x/y/z; /a/b/z` as locations with values.
    pub fn store_copy(
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    );

    /// Loads the preimage of a given 32-byte hash in memory.
    /// If the preimage is larger than `max_bytes`, its contents is trimmed.
    pub fn reveal_preimage(
        hash_addr: *const u8,
        destination_addr: *mut u8,
        max_bytes: usize,
    ) -> usize;
}

/// Wrapper trait for 'rollup_core' host functions.
///
/// Will be mocked out in unit tests.  Parameterised by `&self` - note that while
/// these function may cause side effects, they are unsafe to call.
///
/// # Safety
/// The caller should take care to give correct buffer sizes, pointers, and
/// path-encodings.  See safety notes on each method for more details.
#[cfg_attr(test, mockall::automock)]
pub unsafe trait RawRollupCore {
    /// See [read_input].
    ///
    /// # Safety
    /// - `type`, `level` & `id` must all be valid pointers to their respective types.
    /// - `dst` must point to a mutable slice of bytes with `capacity >= max_bytes`.
    ///
    /// The respective pointers must only be consumed if `return > 0`.
    unsafe fn read_input(
        &self,
        r#type: *mut Input,
        level: *mut i32,
        id: *mut i32,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize;

    /// See [write_output].
    ///
    /// # Safety
    /// - `src` must be a ptr to an initialised slice of bytes.
    /// - `num_bytes` must be the length of that slice.
    unsafe fn write_output(&self, src: *const u8, num_bytes: usize) -> WriteResult;

    /// See [write_debug].
    ///
    /// # Safety
    /// - `src` must be a ptr to an initialised slice of bytes.
    /// - `num_bytes` must be the length of that slice.
    unsafe fn write_debug(src: *const u8, num_bytes: usize);

    /// See [store_has].
    ///
    /// # Safety
    /// - `path` must be a ptr to a correctly path-encoded slice of bytes.
    /// - `len` must be the length of that slice.
    unsafe fn store_has(&self, path: *const u8, path_len: usize) -> ValueType;

    /// See [store_read].
    ///
    /// # Safety
    /// - `path` must be a ptr to a correctly path-encoded slice of bytes.
    /// - `len` must be the length of that slice.
    /// - `dst` must point to a mutable slice of bytes with `capacity >= max_bytes`.
    ///
    /// # Traps
    /// `traps` if `path` does not exist.  You should check with [store_has] first.
    unsafe fn store_read(
        &self,
        path: *const u8,
        path_len: usize,
        offset: usize,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize;

    /// See [store_write].
    ///
    /// # Safety
    /// - `path` must be a ptr to a correctly path-encoded slice of bytes.
    /// - `len` must be the length of that slice.
    /// - `dst` must point to a slice of bytes with `length >= num_bytes`.
    unsafe fn store_write(
        &self,
        path: *const u8,
        path_len: usize,
        offset: usize,
        src: *const u8,
        num_bytes: usize,
    ) -> WriteResult;

    /// See [store_delete].
    ///
    /// # Safety
    /// - `path` must be a ptr to a correctly path-encoded slice of bytes.
    /// - `len` must be the length of that slice.
    ///
    /// # Traps
    /// `traps` if `path` does not exist.  You should check with [store_has] first.
    unsafe fn store_delete(&self, path: *const u8, len: usize);

    /// See [store_list_size].
    ///
    /// # Safety
    /// - `path` must be a ptr to a correctly path-encoded slice of bytes.
    /// - `len` must be the length of that slice.
    ///
    /// # Traps
    /// `traps` if `path` does not exist.  You should check with [store_has] first.
    unsafe fn store_list_size(&self, path: *const u8, path_len: usize) -> i64;

    /// See [store_list_get].
    ///
    /// # Safety
    /// - `path` must be a ptr to a correctly path-encoded slice of bytes.
    /// - `len` must be the length of that slice.
    /// - `dst` must point to a mutable slice of bytes with `capacity >= max_size`.
    ///
    /// # Traps
    /// `traps` if:
    /// - `path` does not exist.  You should check with [store_has] first.
    /// - `index > store_list_size(path) || index < 0`
    unsafe fn store_list_get(
        &self,
        path: *const u8,
        path_len: usize,
        index: i64,
        dst: *mut u8,
        max_size: usize,
    ) -> usize;

    /// See [store_move] above.
    ///
    /// # Safety
    /// - `from_path` and `to_path` must be pointers to correctly path encoded slices
    ///   bytes.
    /// - `from_path_len` and `to_path_len` must be length of those slices respectively.
    ///
    /// # Traps
    /// - if `from_path` doesn't exist
    /// - if `from_path` is a prefix of `to_path`?
    unsafe fn store_move(
        &self,
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    );

    /// See [store_copy] above.
    ///
    /// # Safety
    /// - `from_path` and `to_path` must be pointers to correctly path encoded slices
    ///   bytes.
    /// - `from_path_len` and `to_path_len` must be length of those slices respectively.
    ///
    /// # Traps
    /// - if `from_path` doesn't exist
    /// - if `from_path` is a prefix of `to_path`?
    unsafe fn store_copy(
        &self,
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    );

    /// Loads the preimage of a given hash of size `PREIMAGE_HASH_BYTES` in memory.
    /// If the preimage is larger than `max_bytes`, its contents is trimmed.
    ///
    /// # Safety
    /// - `hash` must be a ptr to a slice of `PREIMAGE_HASH_BYTES` bytes
    /// - `destination_addr `must point to a mutable slice of bytes with
    ///   `capacity >= max_size`.
    unsafe fn reveal_preimage(
        &self,
        hash_addr: *const u8,
        destination_addr: *mut u8,
        max_bytes: usize,
    ) -> usize;
}

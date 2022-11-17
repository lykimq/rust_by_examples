// Copyright (c) 2022 TriliTech <contact@trili.tech>
// SPDX-License-Identifier: MIT
//! Definition of **Runtime** api that is callable from *safe* rust.
//!
//! Includes blanket implementation for all types implementing [RawRollupCore].
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::input::{Input, MessageData, SlotData};
use crate::path::Path;
#[cfg(feature = "alloc")]
use crate::path::{OwnedPath, PATH_MAX_SIZE};
use crate::rollup_core::{RawRollupCore, WriteResult, PREIMAGE_HASH_SIZE};

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
/// List of errors that may be returned when called [Runtime] methods.
///
/// Note that certain error situtations may instead cause the runtime to *trap*, rather
/// than return an explicit error.
pub enum RuntimeError {
    /// Attempted to write too many bytes to output/storage.
    WriteTooLarge,
    /// Attempted to read from/delete a key that does not exist.
    PathNotFound,
    /// Attempted to get a subkey at an out-of-bounds index.
    StoreListIndexOutOfBounds,
}

/// Returned by [`Runtime::store_has`] - specifies whether a path has a value or is a prefix.
#[derive(Debug, Copy, Clone)]
pub enum ValueType {
    /// The path has a value, but is not a prefix to further values.
    Value,
    /// The path is a prefix to further values, but has no value.
    Subtree,
    /// The path has a value, and is a prefix to further values.
    ValueWithSubtree,
}

impl From<WriteResult> for Result<(), RuntimeError> {
    fn from(write_result: WriteResult) -> Self {
        match write_result {
            WriteResult::Ok => Ok(()),
            WriteResult::TooLarge => Err(RuntimeError::WriteTooLarge),
        }
    }
}

/// Safe wrappers for host capabilities.
///
/// **NB**:
/// - methods that take `&self` will not cause changes to the runtime state.
/// - methods taking `&mut self` are expected to cause changes - either to *input*,
///   *output* or *durable storage*.
pub trait Runtime {
    /// Write contents of the given slice to output.
    fn write_output(&mut self, from: &[u8]) -> Result<(), RuntimeError>;

    /// Read up to `max_bytes` from input.
    ///
    /// Returns `None` if no bytes were read.  This happens when the kernel has finished
    /// reading the current message.  The kernel should yield to be given the next message.
    #[cfg(feature = "alloc")]
    fn read_input(&mut self, max_bytes: usize) -> Option<Input>;

    /// Returns whether a given path exists in storage.
    fn store_has<T: Path>(&self, path: &T) -> Option<ValueType>;

    /// Read up to `max_bytes` from the given path in storage, starting `from_offset`.
    #[cfg(feature = "alloc")]
    fn store_read<T: Path>(
        &self,
        path: &T,
        from_offset: usize,
        max_bytes: usize,
    ) -> Result<Vec<u8>, RuntimeError>;

    /// Write the bytes given by `src` to storage at `path`, starting `at_offset`.
    fn store_write<T: Path>(
        &mut self,
        path: &T,
        src: &[u8],
        at_offset: usize,
    ) -> Result<(), RuntimeError>;

    /// Delete `path` from storage.
    fn store_delete<T: Path>(&mut self, path: &T) -> Result<(), RuntimeError>;

    /// Count the number of subkeys under `prefix`.
    ///
    /// See [RawRollupCore::store_list_size].
    fn store_count_subkeys<T: Path>(&self, prefix: &T) -> Result<i64, RuntimeError>;

    /// Get the subkey under `prefix` at `index`.
    ///
    /// # Returns
    /// Returns the subkey as an [OwnedPath], **excluding** the `prefix`.
    #[cfg(feature = "alloc")]
    fn store_get_subkey<T: Path>(
        &self,
        prefix: &T,
        index: i64,
    ) -> Result<OwnedPath, RuntimeError>;

    /// Move one part of durable storage to a different location
    ///
    /// See [RawRollupCore::store_move].
    fn store_move(
        &mut self,
        from_path: &impl Path,
        to_path: &impl Path,
    ) -> Result<(), RuntimeError>;

    /// Copy one part of durable storage to a different location
    ///
    /// See [RawRollupCore::store_copy].
    fn store_copy(
        &mut self,
        from_path: &impl Path,
        to_path: &impl Path,
    ) -> Result<(), RuntimeError>;

    /// Reveal pre-image from a hash of size `PREIMAGE_HASH_SIZE` in bytes
    fn reveal_preimage(
        &self,
        hash: &[u8; PREIMAGE_HASH_SIZE],
        destination: &mut [u8],
    ) -> usize;
}

impl<Host> Runtime for Host
where
    Host: RawRollupCore,
{
    fn write_output(&mut self, output: &[u8]) -> Result<(), RuntimeError> {
        let result =
            unsafe { RawRollupCore::write_output(self, output.as_ptr(), output.len()) };

        match result {
            WriteResult::Ok => Ok(()),
            WriteResult::TooLarge => Err(RuntimeError::WriteTooLarge),
        }
    }

    #[cfg(feature = "alloc")]
    fn read_input(&mut self, max_bytes: usize) -> Option<Input> {
        use crate::rollup_core::Input as InputType;
        use core::mem::MaybeUninit;

        let mut buffer = Vec::with_capacity(max_bytes);

        let mut input_type = MaybeUninit::<InputType>::uninit();
        let mut level = MaybeUninit::<i32>::uninit();
        let mut id = MaybeUninit::<i32>::uninit();

        let bytes_read = unsafe {
            RawRollupCore::read_input(
                self,
                input_type.as_mut_ptr(),
                level.as_mut_ptr(),
                id.as_mut_ptr(),
                buffer.as_mut_ptr(),
                max_bytes,
            )
        };

        if bytes_read == 0 {
            return None;
        }

        unsafe { buffer.set_len(bytes_read) };

        // Match on input type first - in future the input types
        // may only set some of the values here - so assume they are
        // initialized explicitly on each branch.
        let input = match unsafe { input_type.assume_init() } {
            InputType::MessageData => Input::Message(MessageData::new(
                unsafe { level.assume_init() },
                unsafe { id.assume_init() },
                buffer,
            )),
            InputType::SlotDataChunk => Input::Slot(SlotData::new(
                unsafe { level.assume_init() },
                unsafe { id.assume_init() },
                buffer,
            )),
        };

        Some(input)
    }

    fn store_has<T: Path>(&self, path: &T) -> Option<ValueType> {
        use crate::rollup_core::ValueType as VTRaw;

        let value_type =
            unsafe { RawRollupCore::store_has(self, path.as_ptr(), path.size()) };

        match value_type {
            VTRaw::None => None,
            VTRaw::Value => Some(ValueType::Value),
            VTRaw::Subtree => Some(ValueType::Subtree),
            VTRaw::ValueWithSubtree => Some(ValueType::ValueWithSubtree),
        }
    }

    #[cfg(feature = "alloc")]
    fn store_read<T: Path>(
        &self,
        path: &T,
        from_offset: usize,
        max_bytes: usize,
    ) -> Result<Vec<u8>, RuntimeError> {
        let _ = check_path_exists(self, path)?;

        let mut buffer = Vec::with_capacity(max_bytes);

        unsafe {
            let slice = core::slice::from_raw_parts_mut(buffer.as_mut_ptr(), max_bytes);
            let bytes_read = store_read_slice(self, path, from_offset, slice);

            buffer.set_len(bytes_read);
        }

        Ok(buffer)
    }

    fn store_write<T: Path>(
        &mut self,
        path: &T,
        src: &[u8],
        at_offset: usize,
    ) -> Result<(), RuntimeError> {
        unsafe {
            RawRollupCore::store_write(
                self,
                path.as_ptr(),
                path.size(),
                at_offset,
                src.as_ptr(),
                src.len(),
            )
        }
        .into()
    }

    fn store_delete<T: Path>(&mut self, path: &T) -> Result<(), RuntimeError> {
        let _ = check_path_exists(self, path)?;

        unsafe { RawRollupCore::store_delete(self, path.as_ptr(), path.size()) };
        Ok(())
    }

    fn store_count_subkeys<T: Path>(&self, path: &T) -> Result<i64, RuntimeError> {
        Ok(unsafe { RawRollupCore::store_list_size(self, path.as_ptr(), path.size()) })
    }

    #[cfg(feature = "alloc")]
    fn store_get_subkey<T: Path>(
        &self,
        path: &T,
        index: i64,
    ) -> Result<OwnedPath, RuntimeError> {
        let size = self.store_count_subkeys(path)?;

        if index >= 0 && index < size {
            Ok(store_get_subkey_unchecked(self, path, index))
        } else {
            Err(RuntimeError::StoreListIndexOutOfBounds)
        }
    }

    fn store_move(
        &mut self,
        from_path: &impl Path,
        to_path: &impl Path,
    ) -> Result<(), RuntimeError> {
        let _ = check_path_exists(self, from_path)?;

        unsafe {
            RawRollupCore::store_move(
                self,
                from_path.as_ptr(),
                from_path.size(),
                to_path.as_ptr(),
                to_path.size(),
            )
        }
        Ok(())
    }

    fn store_copy(
        &mut self,
        from_path: &impl Path,
        to_path: &impl Path,
    ) -> Result<(), RuntimeError> {
        let _ = check_path_exists(self, from_path)?;

        unsafe {
            RawRollupCore::store_copy(
                self,
                from_path.as_ptr(),
                from_path.size(),
                to_path.as_ptr(),
                to_path.size(),
            )
        }
        Ok(())
    }

    fn reveal_preimage(
        &self,
        hash: &[u8; PREIMAGE_HASH_SIZE],
        buffer: &mut [u8],
    ) -> usize {
        unsafe {
            RawRollupCore::reveal_preimage(
                self,
                hash.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        }
    }
}

/// Loads part of a value from storage, into the buffer.
///
/// # Safety
/// - `path` must exist in the store.
/// - the value at `path` must be of `size > from_offset`.
/// - it will write *up to* `buffer.len()` bytes to the buffer,
///   and return the actual number of bytes written. It is the caller's
///   responsibility to ensure that `buffer` is otherwise initialised.
#[must_use]
#[cfg(feature = "alloc")]
unsafe fn store_read_slice<Host: RawRollupCore, T: Path>(
    host: &Host,
    path: &T,
    from_offset: usize,
    buffer: &mut [u8],
) -> usize {
    host.store_read(
        path.as_ptr(),
        path.size(),
        from_offset,
        buffer.as_mut_ptr(),
        buffer.len(),
    )
}

/// Loads part of a value from storage, extending the buffer.
///
/// It will not read more bytes than the buffer has remaining capacity for.
/// ie calling `store_read_extend` will **not** cause `buffer` to reallocate.
///
/// # Safety
/// - `path` must exist.
/// - the value at `path` must be of `size > from_offset`.
#[cfg(feature = "alloc")]
unsafe fn store_read_extend<Host: RawRollupCore, T: Path>(
    host: &Host,
    path: &T,
    mut from_offset: usize,
    buffer: &mut Vec<u8>,
) {
    while buffer.len() < buffer.capacity() {
        let slice =
            &mut core::slice::from_raw_parts_mut(buffer.as_mut_ptr(), buffer.capacity())
                [buffer.len()..buffer.capacity()];

        match store_read_slice(host, path, from_offset, slice) {
            0 => break,
            l => {
                buffer.set_len(buffer.len() + l);
                from_offset += l;
            }
        }
    }
}

/// Loads a value from the store, where it has been prefixed by the size of the value.
#[cfg(feature = "alloc")]
pub fn load_value_sized<Host: RawRollupCore, T: Path>(
    host: &Host,
    path: &T,
) -> Result<Vec<u8>, RuntimeError> {
    let _ = check_path_exists(host, path)?;

    let size = Runtime::store_read(host, path, 0, core::mem::size_of::<usize>())?;
    let size = usize::from_le_bytes(size.try_into().unwrap());
    let mut buffer = Vec::with_capacity(size);

    unsafe { store_read_extend(host, path, core::mem::size_of::<usize>(), &mut buffer) };

    Ok(buffer)
}

/// Saves a value to the store, prefixing it by its size.
pub fn save_value_sized<T: Path>(host: &mut impl RawRollupCore, path: &T, value: &[u8]) {
    use crate::rollup_core::MAX_FILE_CHUNK_SIZE;

    let size = value.len();
    let _ = Runtime::store_delete(host, path);

    let size = size.to_le_bytes();
    Runtime::store_write(host, path, size.as_ref(), 0)
        .expect("Size prefix should fit in MAX_FILE_CHUNK_SIZE.");

    let mut index = 0;
    while index < value.len() {
        let offset = usize::min(value.len(), index + MAX_FILE_CHUNK_SIZE);
        let _ = Runtime::store_write(
            host,
            path,
            &value[index..offset],
            // Offset for size prefix
            index + size.len(),
        )
        .expect("Unable to persist memory to the store.");

        index += MAX_FILE_CHUNK_SIZE;
    }
}

fn check_path_exists<T: Path>(
    runtime: &impl Runtime,
    path: &T,
) -> Result<(), RuntimeError> {
    if let Some(ValueType::Value | ValueType::ValueWithSubtree) = runtime.store_has(path)
    {
        Ok(())
    } else {
        Err(RuntimeError::PathNotFound)
    }
}

#[cfg(feature = "alloc")]
fn store_get_subkey_unchecked(
    host: &impl RawRollupCore,
    path: &impl Path,
    index: i64,
) -> OwnedPath {
    let max_size = PATH_MAX_SIZE - path.size();
    let mut buffer = Vec::with_capacity(max_size);

    unsafe {
        let bytes_written = host.store_list_get(
            path.as_ptr(),
            path.size(),
            index,
            buffer.as_mut_ptr(),
            max_size,
        );

        buffer.set_len(bytes_written);

        OwnedPath::from_bytes_unchecked(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::{Runtime, PREIMAGE_HASH_SIZE};
    use crate::{
        input::{Input, MessageData, SlotData},
        path::{OwnedPath, Path, RefPath, PATH_MAX_SIZE},
        rollup_core::{
            Input as RawInput, MockRawRollupCore, ValueType as RawValueType, WriteResult,
            MAX_OUTPUT_SIZE,
        },
        runtime::RuntimeError,
    };
    use std::slice::{from_raw_parts, from_raw_parts_mut};
    use test_helpers::*;

    // The amount to read from input in tests
    const INPUT_MESSAGE_SIZE: usize = 100;
    const READ_SIZE: usize = 80;

    #[test]
    fn given_output_written_then_ok() {
        // Arrange
        let mut mock = MockRawRollupCore::new();
        let output = "just a bit of output we want to write";

        mock.expect_write_output()
            .withf(|ptr, len| {
                let slice = unsafe { from_raw_parts(*ptr, *len) };

                output.as_bytes() == slice
            })
            .return_const(WriteResult::Ok);

        // Act
        let result = mock.write_output(output.as_bytes());

        // Assert
        assert_eq!(Ok(()), result);
    }

    #[test]
    fn given_output_too_large_then_err() {
        // Arrange
        let mut mock = MockRawRollupCore::new();

        let output = [b'a'; MAX_OUTPUT_SIZE + 1];

        mock.expect_write_output().return_once(|ptr, len| {
            let slice = unsafe { from_raw_parts(ptr, len) };

            assert!(slice.iter().all(|b| b == &b'a'));
            assert_eq!(MAX_OUTPUT_SIZE + 1, slice.len());

            WriteResult::TooLarge
        });

        // Act
        let result = mock.write_output(output.as_slice());

        // Assert
        assert_eq!(Err(RuntimeError::WriteTooLarge), result);
    }

    #[test]
    fn read_input_returns_none_when_nothing_read() {
        // Arrange
        let mut mock = MockRawRollupCore::new();
        mock.expect_read_input().return_const(0 as usize);

        // Act
        let outcome = mock.read_input(INPUT_MESSAGE_SIZE);

        // Assert
        assert_eq!(None, outcome);
    }

    #[test]
    fn read_slot_input_with_size_max_bytes() {
        // Arrange
        let level = 2;
        let id = 3;
        let byte = b'!';
        const FRACTION: usize = 1;

        let mut mock =
            read_input_with(RawInput::SlotDataChunk, level, id, byte, FRACTION);

        // Act
        let outcome = mock.read_input(INPUT_MESSAGE_SIZE);

        // Assert
        let expected = SlotData::new(
            level,
            id,
            Box::new([byte; INPUT_MESSAGE_SIZE / FRACTION]).to_vec(),
        );

        assert_eq!(Some(Input::Slot(expected)), outcome);
    }

    #[test]
    fn read_slot_input_with_size_lt_max_bytes() {
        // Arrange
        let level = 5;
        let id = 108;
        let byte = b'\\';
        const FRACTION: usize = 2;

        let mut mock =
            read_input_with(RawInput::SlotDataChunk, level, id, byte, FRACTION);

        // Act
        let outcome = mock.read_input(INPUT_MESSAGE_SIZE);

        // Assert
        let expected = SlotData::new(
            level,
            id,
            Box::new([byte; INPUT_MESSAGE_SIZE / FRACTION]).to_vec(),
        );

        assert_eq!(Some(Input::Slot(expected)), outcome);
    }

    #[test]
    fn read_message_input_with_size_max_bytes() {
        // Arrange
        let level = 5;
        let id = 12908;
        let byte = b'?';
        const FRACTION: usize = 1;

        let mut mock = read_input_with(RawInput::MessageData, level, id, byte, FRACTION);

        // Act
        let outcome = mock.read_input(INPUT_MESSAGE_SIZE);

        // Assert
        let expected = MessageData::new(
            level,
            id,
            Box::new([byte; INPUT_MESSAGE_SIZE / FRACTION]).to_vec(),
        );

        assert_eq!(Some(Input::Message(expected)), outcome);
    }

    #[test]
    fn read_message_input_with_size_lt_max_bytes() {
        // Arrange
        let level = 5;
        let id = 99128;
        let byte = b'A';
        const FRACTION: usize = 4;

        let mut mock = read_input_with(RawInput::MessageData, level, id, byte, FRACTION);

        // Act
        let outcome = mock.read_input(INPUT_MESSAGE_SIZE);

        // Assert
        let expected = MessageData::new(
            level,
            id,
            Box::new([byte; INPUT_MESSAGE_SIZE / FRACTION]).to_vec(),
        );

        assert_eq!(Some(Input::Message(expected)), outcome);
    }

    #[test]
    fn store_has_existing_return_true() {
        // Arrange
        let mut mock = MockRawRollupCore::new();
        let existing_path = RefPath::assert_from("/an/Existing/path".as_bytes());

        mock.expect_store_has()
            .withf(move |ptr, size| {
                let bytes = unsafe { from_raw_parts(*ptr, *size) };
                existing_path.as_bytes() == bytes
            })
            .return_const(RawValueType::Value);

        // Act
        let result = mock.store_has(&existing_path);

        // Assert
        assert!(result.is_some());
    }

    fn mock_path_not_existing(path_bytes: Vec<u8>) -> MockRawRollupCore {
        let mut mock = MockRawRollupCore::new();

        mock.expect_store_has()
            .withf(move |ptr, size| {
                let bytes = unsafe { from_raw_parts(*ptr, *size) };
                path_bytes == bytes
            })
            .return_const(RawValueType::None);

        mock
    }

    #[test]
    fn store_has_not_existing_returns_false() {
        // Arrange
        let path_bytes = String::from("/does/not.exist").into_bytes();
        let non_existent_path: OwnedPath = RefPath::assert_from(&path_bytes).into();

        let mock = mock_path_not_existing(path_bytes);

        // Act
        let result = mock.store_has(&non_existent_path);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn store_read_max_bytes() {
        // Arrange
        const FRACTION: usize = 1;
        const PATH: RefPath<'static> = RefPath::assert_from("/a/simple/path".as_bytes());
        const OFFSET: usize = 5;

        let mut mock = mock_path_exists(PATH.as_bytes());
        mock.expect_store_read()
            .withf(|path_ptr, path_size, from_offset, _, max_bytes| {
                let slice = unsafe { from_raw_parts(*path_ptr, *path_size) };

                READ_SIZE == *max_bytes
                    && PATH.as_bytes() == slice
                    && OFFSET == *from_offset
            })
            .return_once(|_, _, _, buf_ptr, _| {
                let stored_bytes = [b'2'; READ_SIZE / FRACTION];
                let buffer = unsafe { from_raw_parts_mut(buf_ptr, READ_SIZE / FRACTION) };
                buffer.copy_from_slice(&stored_bytes);
                READ_SIZE / FRACTION
            });

        // Act
        let result = mock.store_read(&PATH, OFFSET, READ_SIZE);

        // Assert
        let expected = std::iter::repeat(b'2').take(READ_SIZE / FRACTION).collect();

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn store_read_lt_max_bytes() {
        // Arrange
        const FRACTION: usize = 5;
        const PATH: RefPath<'static> = RefPath::assert_from("/a/simple/path".as_bytes());
        const OFFSET: usize = 10;

        let mut mock = mock_path_exists(PATH.as_bytes());
        mock.expect_store_read()
            .withf(|path_ptr, path_size, from_offset, _, max_bytes| {
                let slice = unsafe { from_raw_parts(*path_ptr, *path_size) };

                READ_SIZE == *max_bytes
                    && PATH.as_bytes() == slice
                    && OFFSET == *from_offset
            })
            .return_once(|_, _, _, buf_ptr, _| {
                let stored_bytes = [b'Z'; READ_SIZE / FRACTION];
                let buffer = unsafe { from_raw_parts_mut(buf_ptr, READ_SIZE / FRACTION) };
                buffer.copy_from_slice(&stored_bytes);
                READ_SIZE / FRACTION
            });

        // Act
        let result = mock.store_read(&PATH, OFFSET, READ_SIZE);

        // Assert
        let expected = std::iter::repeat(b'Z').take(READ_SIZE / FRACTION).collect();

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn store_read_path_not_found() {
        // Arrange
        let bytes = "/a/2nd/PATH.which/doesnt/exist".as_bytes().to_vec();
        let path: OwnedPath = RefPath::assert_from(&bytes).into();
        let offset = 25;

        let mock = mock_path_not_existing(bytes);

        // Act
        let result = mock.store_read(&path, offset, READ_SIZE);

        // Assert
        assert_eq!(Err(RuntimeError::PathNotFound), result);
    }

    #[test]
    fn store_write_ok() {
        // Arrange
        const PATH: RefPath<'static> = RefPath::assert_from("/a/simple/path".as_bytes());
        const OUTPUT: &'static [u8] = "One two three four five".as_bytes();
        const OFFSET: usize = 12398;

        let mut mock = MockRawRollupCore::new();
        mock.expect_store_write()
            .withf(|path_ptr, path_size, at_offset, src_ptr, src_size| {
                let path_slice = unsafe { from_raw_parts(*path_ptr, *path_size) };
                let output_slice = unsafe { from_raw_parts(*src_ptr, *src_size) };

                OUTPUT == output_slice
                    && PATH.as_bytes() == path_slice
                    && OFFSET == *at_offset
            })
            .return_const(WriteResult::Ok);

        // Act
        let result = mock.store_write(&PATH, OUTPUT, OFFSET);

        // Assert
        assert_eq!(Ok(()), result);
    }

    #[test]
    fn store_write_too_large() {
        // Arrange
        const PATH: RefPath<'static> = RefPath::assert_from("/a/simple/path".as_bytes());
        const OUTPUT: &'static [u8] = "once I saw a fish alive".as_bytes();
        const OFFSET: usize = 0;

        let mut mock = MockRawRollupCore::new();
        mock.expect_store_write()
            .withf(|path_ptr, path_size, at_offset, src_ptr, src_size| {
                let path_slice = unsafe { from_raw_parts(*path_ptr, *path_size) };
                let output_slice = unsafe { from_raw_parts(*src_ptr, *src_size) };

                OUTPUT == output_slice
                    && PATH.as_bytes() == path_slice
                    && OFFSET == *at_offset
            })
            .return_const(WriteResult::TooLarge);

        // Act
        let result = mock.store_write(&PATH, OUTPUT, OFFSET);

        // Assert
        assert_eq!(Err(RuntimeError::WriteTooLarge), result);
    }

    #[test]
    fn store_delete() {
        // Arrange
        const PATH: RefPath<'static> =
            RefPath::assert_from("/a/2nd/PATH.which/does/exist".as_bytes());

        let mut mock = mock_path_exists(PATH.as_bytes());
        mock.expect_store_delete()
            .withf(|ptr, size| {
                let slice = unsafe { from_raw_parts(*ptr, *size) };

                PATH.as_bytes() == slice
            })
            .return_const(());

        // Act
        let result = mock.store_delete(&PATH);

        // Assert
        assert_eq!(Ok(()), result);
    }

    #[test]
    fn store_delete_path_not_found() {
        // Arrange
        let bytes = String::from("/a/2nd/PATH.which/doesnt/exist").into_bytes();
        let path: OwnedPath = RefPath::assert_from(&bytes).into();

        let mut mock = mock_path_not_existing(bytes);

        // Act
        let result = mock.store_delete(&path);

        // Assert
        assert_eq!(Err(RuntimeError::PathNotFound), result);
    }

    #[test]
    fn store_count_subkeys() {
        // Arrange
        const PATH: RefPath<'static> =
            RefPath::assert_from("/prefix/of/other/keys".as_bytes());

        let subkey_count = 14;

        let mut mock = MockRawRollupCore::new();

        mock.expect_store_list_size()
            .withf(|ptr, size| {
                let slice = unsafe { from_raw_parts(*ptr, *size) };

                PATH.as_bytes() == slice
            })
            .return_const(subkey_count);

        // Act
        let result = mock.store_count_subkeys(&PATH);

        // Assert
        assert_eq!(Ok(subkey_count), result);
    }

    #[test]
    fn store_get_subkey() {
        // Arrange
        const PATH: RefPath<'static> =
            RefPath::assert_from("/prefix/of/other/paths".as_bytes());

        let subkey_index = 14;
        let subkey_count = 20;
        let buffer_size = PATH_MAX_SIZE - PATH.size();

        let mut mock = MockRawRollupCore::new();
        mock.expect_store_list_size()
            .withf(|ptr, size| {
                let slice = unsafe { from_raw_parts(*ptr, *size) };

                PATH.as_bytes() == slice
            })
            .return_const(subkey_count);

        mock.expect_store_list_get()
            .withf(move |path_ptr, path_size, index, _, max_bytes| {
                let slice = unsafe { from_raw_parts(*path_ptr, *path_size) };

                PATH.as_bytes() == slice
                    && subkey_index == *index
                    && buffer_size == *max_bytes
            })
            .return_once(|_, _, _, buf_ptr, _| {
                let path_bytes = "/short/suffix".as_bytes();
                let buffer = unsafe { from_raw_parts_mut(buf_ptr, path_bytes.len()) };
                buffer.copy_from_slice(path_bytes);

                path_bytes.len()
            });

        // Act
        let result = mock.store_get_subkey(&PATH, subkey_index);

        // Assert
        let expected = RefPath::assert_from("/short/suffix".as_bytes()).into();

        assert_eq!(Ok(expected), result);
    }

    #[test]
    fn store_get_subkey_index_out_of_range_upper() {
        // Arrange
        const PATH: RefPath<'static> =
            RefPath::assert_from("/prefix/of/other/paths".as_bytes());

        let subkey_index = 0;
        let subkey_count = 0;

        let mut mock = MockRawRollupCore::new();
        mock.expect_store_list_size()
            .withf(|ptr, size| {
                let slice = unsafe { from_raw_parts(*ptr, *size) };

                PATH.as_bytes() == slice
            })
            .return_const(subkey_count);

        // Act
        let result = mock.store_get_subkey(&PATH, subkey_index);

        // Assert
        assert_eq!(Err(RuntimeError::StoreListIndexOutOfBounds), result);
    }

    #[test]
    fn store_get_subkey_index_out_of_range_lower() {
        // Arrange
        const PATH: RefPath<'static> =
            RefPath::assert_from("/prefix/of/other/paths".as_bytes());

        let subkey_index = -1;
        let subkey_count = 5;

        let mut mock = MockRawRollupCore::new();
        mock.expect_store_list_size()
            .withf(|ptr, size| {
                let slice = unsafe { from_raw_parts(*ptr, *size) };

                PATH.as_bytes() == slice
            })
            .return_const(subkey_count);

        // Act
        let result = mock.store_get_subkey(&PATH, subkey_index);

        // Assert
        assert_eq!(Err(RuntimeError::StoreListIndexOutOfBounds), result);
    }

    #[test]
    fn reveal_preimage_ok() {
        let mut mock = MockRawRollupCore::new();

        mock.expect_reveal_preimage()
            .withf(|hash_addr, _dest_addr, max_bytes| {
                let hash = unsafe { from_raw_parts(*hash_addr, PREIMAGE_HASH_SIZE) };
                hash == &[5; PREIMAGE_HASH_SIZE] && *max_bytes == 55
            })
            .return_once(|_, destination_address, _| {
                let revealed_bytes = [b'!'; 50];
                let buffer = unsafe { from_raw_parts_mut(destination_address, 50) };
                buffer.copy_from_slice(&revealed_bytes);
                50
            });
        let mut buffer = [0; 55];
        // Act
        let result =
            mock.reveal_preimage(&[5; PREIMAGE_HASH_SIZE], buffer.as_mut_slice());

        // Assert
        assert_eq!(50, result);
    }

    mod test_helpers {
        use super::{MockRawRollupCore, RawInput, RawValueType, INPUT_MESSAGE_SIZE};
        use std::slice::{from_raw_parts, from_raw_parts_mut};

        pub fn mock_path_exists(path_bytes: &'static [u8]) -> MockRawRollupCore {
            let mut mock = MockRawRollupCore::new();

            mock.expect_store_has()
                .withf(move |ptr, size| {
                    let bytes = unsafe { from_raw_parts(*ptr, *size) };
                    path_bytes == bytes
                })
                .return_const(RawValueType::Value);

            mock
        }

        pub fn read_input_with(
            input: RawInput,
            level: i32,
            id: i32,
            fill_with: u8,
            fill_fraction: usize,
        ) -> MockRawRollupCore {
            let mut mock = MockRawRollupCore::new();

            let write_bytes = INPUT_MESSAGE_SIZE / fill_fraction;

            let input_bytes = std::iter::repeat(fill_with)
                .take(write_bytes)
                .collect::<Box<_>>();

            mock.expect_read_input().return_once(
                move |input_type_arg, level_arg, id_arg, buffer_arg, max_bytes_arg| {
                    assert_eq!(max_bytes_arg, INPUT_MESSAGE_SIZE);

                    unsafe {
                        *input_type_arg = input;
                        *level_arg = level;
                        *id_arg = id;
                        let buffer = from_raw_parts_mut(buffer_arg, write_bytes);
                        buffer.copy_from_slice(input_bytes.as_ref());
                    }
                    write_bytes
                },
            );

            mock
        }
    }
}

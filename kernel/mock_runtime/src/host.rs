//! Contains an implementation of [RawRollupCore] suitable for running the
//! kernel standalone for experiements and testing purposes. Used when
//! _not_ compiling to **wasm**.

use crate::state::{HostState, NextInput};
use core::{
    cell::RefCell,
    ptr,
    slice::{from_raw_parts, from_raw_parts_mut},
};
use host::rollup_core::{
    Input, RawRollupCore, ValueType, WriteResult, PREIMAGE_HASH_SIZE,
};

struct WrappedLog(RefCell<Vec<String>>);
unsafe impl Sync for WrappedLog {}

impl WrappedLog {
    fn add_debug_log(&self, msg: String) {
        eprintln!("DEBUG: {}", msg);
        self.0.borrow_mut().push(msg)
    }

    fn read_log<T>(&self, f: impl FnOnce(&[String]) -> T) -> T {
        let log = self.0.borrow();

        f(log.as_slice())
    }
}

thread_local! {
    static DEBUG_LOG: WrappedLog = WrappedLog(RefCell::new(Vec::new()));
}

/// Run assertion on the current `DebugLog` of `MockHost`.
pub fn check_debug_log(f: impl Fn(&[String])) {
    DEBUG_LOG.with(|log| log.read_log(f))
}

/// Reset `MockHost` runtime state to `HostState::default()`.
pub fn reset_debug_log() {
    DEBUG_LOG.with(|log| log.0.borrow_mut().clear());
}

/// The runtime host when _not_ running in **wasm**.
#[derive(Debug, Default)]
pub struct MockHost {
    state: RefCell<HostState>,
}

impl MockHost {
    /// Consumes the `MockHost` and returns its inner [`HostState`].
    pub fn into_inner(self) -> HostState {
        self.state.into_inner()
    }
}

impl From<HostState> for MockHost {
    fn from(state: HostState) -> Self {
        Self {
            state: RefCell::new(state),
        }
    }
}

impl AsMut<HostState> for MockHost {
    fn as_mut(&mut self) -> &mut HostState {
        self.state.get_mut()
    }
}

#[allow(unused_variables)]
unsafe impl RawRollupCore for MockHost {
    unsafe fn read_input(
        &self,
        r#type: *mut Input,
        level: *mut i32,
        id: *mut i32,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize {
        if let Some(NextInput {
            input_type,
            level: input_level,
            id: input_id,
            payload,
        }) = self.state.borrow_mut().handle_read_input(max_bytes)
        {
            ptr::write(r#type, input_type);
            ptr::write(level, input_level);
            ptr::write(id, input_id);

            // safe as payload.len() <= max_bytes
            let slice = from_raw_parts_mut(dst, payload.len());
            slice.copy_from_slice(payload.as_slice());

            payload.len()
        } else {
            0_usize
        }
    }

    unsafe fn write_debug(src: *const u8, num_bytes: usize) {
        let debug_out = from_raw_parts(src, num_bytes).to_vec();

        let debug = String::from_utf8(debug_out).expect("unexpected non-utf8 debug log");

        #[cfg(not(any(target_arch = "wasm32", test)))]
        eprintln!("DEBUG: {}", &debug);

        DEBUG_LOG.with(|log| log.add_debug_log(debug));
    }

    unsafe fn write_output(&self, src: *const u8, num_bytes: usize) -> WriteResult {
        let output = from_raw_parts(src, num_bytes).to_vec();

        self.state.borrow_mut().handle_write_output(output)
    }

    unsafe fn store_has(&self, path: *const u8, len: usize) -> ValueType {
        let path = from_raw_parts(path, len);
        self.state.borrow().handle_store_has(path)
    }

    unsafe fn store_read(
        &self,
        path: *const u8,
        len: usize,
        offset: usize,
        dst: *mut u8,
        max_bytes: usize,
    ) -> usize {
        let path = from_raw_parts(path, len);

        let bytes = self
            .state
            .borrow()
            .handle_store_read(path, offset, max_bytes);

        assert!(bytes.len() <= max_bytes);

        let slice = from_raw_parts_mut(dst, bytes.len());
        slice.copy_from_slice(bytes.as_slice());

        bytes.len()
    }

    unsafe fn store_write(
        &self,
        path: *const u8,
        len: usize,
        offset: usize,
        src: *const u8,
        num_bytes: usize,
    ) -> WriteResult {
        let path = from_raw_parts(path, len);
        let bytes = from_raw_parts(src, num_bytes);

        self.state
            .borrow_mut()
            .handle_store_write(path, offset, bytes)
    }

    unsafe fn store_delete(&self, path: *const u8, len: usize) {
        let path = from_raw_parts(path, len);

        self.state.borrow_mut().handle_store_delete(path);
    }

    unsafe fn store_list_size(&self, path: *const u8, len: usize) -> i64 {
        let path = from_raw_parts(path, len);

        self.state.borrow().handle_store_list_size(path)
    }

    unsafe fn store_list_get(
        &self,
        path: *const u8,
        len: usize,
        index: i64,
        dst: *mut u8,
        max_size: usize,
    ) -> usize {
        let path = from_raw_parts(path, len);

        let subkey = self
            .state
            .borrow()
            .handle_store_list_get(path, index)
            .as_bytes()
            .to_vec();

        let copy_len = usize::max(max_size, subkey.len());

        let slice = from_raw_parts_mut(dst, copy_len);
        slice.copy_from_slice(&subkey[..copy_len]);

        copy_len
    }

    unsafe fn store_move(
        &self,
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    ) {
        let from_path = from_raw_parts(from_path, from_path_len);
        let to_path = from_raw_parts(to_path, to_path_len);

        self.state
            .borrow_mut()
            .handle_store_move(from_path, to_path);
    }

    unsafe fn store_copy(
        &self,
        from_path: *const u8,
        from_path_len: usize,
        to_path: *const u8,
        to_path_len: usize,
    ) {
        let from_path = from_raw_parts(from_path, from_path_len);
        let to_path = from_raw_parts(to_path, to_path_len);

        self.state
            .borrow_mut()
            .handle_store_copy(from_path, to_path);
    }

    unsafe fn reveal_preimage(
        &self,
        hash_addr: *const u8,
        destination_addr: *mut u8,
        max_bytes: usize,
    ) -> usize {
        let hash = from_raw_parts(hash_addr, 32)
            .try_into()
            .unwrap_or_else(|_| panic!("Hash is not {} bytes", PREIMAGE_HASH_SIZE));

        let bytes = self
            .state
            .borrow()
            .handle_reveal_preimage(&hash, max_bytes)
            .to_vec();

        assert!(bytes.len() <= max_bytes);

        let slice = from_raw_parts_mut(destination_addr, bytes.len());
        slice.copy_from_slice(bytes.as_slice());

        bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::{reset_debug_log, MockHost};

    use crate::state::{self, HostState};
    use host::{
        input::{Input as KernelInput, MessageData},
        path::RefPath,
        rollup_core::{Input, MAX_INPUT_MESSAGE_SIZE},
        runtime::{load_value_sized, save_value_sized, Runtime},
    };

    #[test]
    fn test_read_input_slot() {
        // Arrange
        let state = new_host_state();

        state.borrow_mut().mark_level_for_input(0);
        state.borrow_mut().add_next_inputs(
            0,
            vec![(Input::MessageData, vec![5; MAX_INPUT_MESSAGE_SIZE / 2])].iter(),
        );

        let mut mock_host = MockHost { state };

        // Act
        let result = mock_host.read_input(MAX_INPUT_MESSAGE_SIZE);

        // Assert
        let expected = Some(KernelInput::Message(MessageData::new(
            0,
            0,
            vec![5; MAX_INPUT_MESSAGE_SIZE / 2],
        )));

        assert_eq!(expected, result);
    }

    #[test]
    fn test_reveal_preimage() {
        // Arrange
        let state = new_host_state();

        let data = vec![b'a'; 3 * 1024];

        let hash = state.borrow_mut().set_preimage(data);

        let mock_host = MockHost { state };

        let mut buffer = [0; 300];
        // Act
        let _result = mock_host.reveal_preimage(&hash, &mut buffer);

        // Assert

        assert_eq!(buffer, [b'a'; 300]);
    }

    fn new_host_state() -> RefCell<HostState> {
        reset_debug_log();

        let mut state = HostState::default();
        state.store.set_value::<state::Checkpoints>(
            state::CHECKPOINTS,
            state::CHECKPOINTS_PER_LEVEL,
        );
        state
            .store
            .set_value::<state::InputConsuming>(state::INPUT_CONSUMING, true);

        RefCell::new(state)
    }

    #[test]
    fn save_value_sized_load_value_sized_roundtrip() {
        // Arrange
        const PATH: RefPath = RefPath::assert_from(b"/testing/path");
        let value = (0..79).cycle().take(8000).collect::<Vec<u8>>();

        let mut host = MockHost {
            state: new_host_state(),
        };

        // Act
        save_value_sized(&mut host, &PATH, value.as_slice());
        let result = load_value_sized(&host, &PATH);

        // Assert
        assert_eq!(result, Ok(value));
    }
}

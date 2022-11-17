//! Mock runtime state & state transitions
use std::collections::VecDeque;

use host::{
    path::{Path, RefPath, DURABLE_STORAGE_PREFIX},
    rollup_core::{
        Input, ValueType, WriteResult, MAX_INPUT_MESSAGE_SIZE,
        MAX_INPUT_SLOT_DATA_CHUNK_SIZE, MAX_OUTPUT_SIZE, PREIMAGE_HASH_SIZE,
    },
};

use crate::trap::{
    trap,
    HostError::*,
    KernelError::{self, *},
    TrapCondition::*,
};

pub(crate) mod store;
use self::store::Store;

pub(crate) type InputLevel = i32;
pub(crate) const INPUT_LEVEL: &str = "/input/level";

pub(crate) type OutputId = u32;
pub(crate) const OUTPUT_ID: &str = "/output/id";

pub(crate) type InputId = i32;
pub(crate) const INPUT_ID: &str = "/input/id";

pub(crate) type InputConsuming = bool;
pub(crate) const INPUT_CONSUMING: &str = "/input/consuming";

pub(crate) type Reboot = u8;
pub(crate) const REBOOT: &str = "/reboot";

pub(crate) type Checkpoints = usize;
/// A checkpoint occurs each time the kernel yields.
pub(crate) const CHECKPOINTS: &str = "/checkpoints";
/// There is a fixed number of checkpoints per non-empty level.
pub(crate) const CHECKPOINTS_PER_LEVEL: Checkpoints = 1000;

// 4KB
const MAX_HEADER_SIZE: usize = 4 * 1024;
// 1MB
const MAX_SLOT_SIZE: usize = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NextInput {
    pub input_type: Input,
    pub level: InputLevel,
    pub id: i32,
    pub payload: Vec<u8>,
}

/// When handling a yield, the next step that the mock host should take.
#[derive(Debug, PartialEq, Eq)]
pub enum YieldStep {
    /// The Host should call [`HostState::handle_yield`] again.
    HandleYield,
    /// The kernel should be rebooted.
    Reboot,
    /// The kernel's `kernel_next` entrypoint should be called.
    Trampoline,
    /// The host should simulate input ticks `at_level` given or greater.
    ///
    /// see [`HostState::add_next_inputs`]
    InputTicks(InputLevel),
    /// The host should mark a new level for input, at at least `level`.
    ///
    /// see [`HostState::mark_level_for_input`]
    MarkLevelForInput(InputLevel),
}

/// The mock `HostState` used by the *mock runtime*, contains the *store* and *debug_log*.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HostState {
    /// Key-value store of runtime state.
    pub store: Store,
    input_levels: VecDeque<InputLevel>,
}

impl Default for HostState {
    fn default() -> Self {
        let mut store = Store::default();
        store.set_value::<InputLevel>(INPUT_LEVEL, 0);
        store.set_value::<InputId>(INPUT_ID, 0);
        store.set_value::<Checkpoints>(CHECKPOINTS, 0);
        store.set_value::<OutputId>(OUTPUT_ID, 0);

        Self {
            store,
            input_levels: VecDeque::new(),
        }
    }
}

impl HostState {
    /// Set the host ready to consume input at level.
    pub fn set_ready_for_input(&mut self, level: InputLevel) {
        self.store.set_value(INPUT_LEVEL, level);
        self.store.set_value(INPUT_CONSUMING, true);
        self.store
            .set_value::<Checkpoints>(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);

        self.mark_level_for_input(level);
    }

    /// Mark a level for input - causing runtime to ask for messages when reaching level.
    ///
    /// see [`HostState::handle_yield`]
    pub fn mark_level_for_input(&mut self, level: InputLevel) {
        let last = self.input_levels.back();
        let input_level: InputLevel = self.store.get_value(INPUT_LEVEL);

        if last.is_none() || *last.unwrap() < level && input_level <= level {
            self.input_levels.push_back(level);
        } else {
            panic!(
                "Attempted to mark level {} for input,\
                 but was smaller than a previously marked level {:?}",
                level, last
            );
        }
    }

    /// Write output bytes into store, if `output.len() < MAX_OUTPUT_SIZE`.
    ///
    /// ```markdown
    /// /output/<level>/<n> >= u32::MAX
    /// -----------
    /// (trigger Boot Sequence)
    ///
    ///
    /// /output/<level>/<n> < u32::MAX
    /// ------------------------------
    /// /output/id           += 1
    /// /output/<level>/<n>  := <written bytes>
    ///    level = /input/level
    ///    n     = /output/id
    /// ```
    ///
    /// `n` is the *id* of the first chunk of the input which this chunk is part of.
    /// It is an *identifier* for a particular input, which is constant across all
    /// parts *of that input*, and is the `id` that is given to the kernel when it calls
    /// [`read_input`].
    ///
    /// [`read_input`]: host::rollup_core::RawRollupCore::read_input
    pub(crate) fn handle_write_output(&mut self, output: Vec<u8>) -> WriteResult {
        if output.len() > MAX_OUTPUT_SIZE {
            return WriteResult::TooLarge;
        }

        let output_id: OutputId = self.store.get_value(OUTPUT_ID);

        if output_id == u32::MAX {
            trap(KernelFailure(TooManyOutputsWritten))
        }

        self.store.set_value::<OutputId>(OUTPUT_ID, output_id + 1);

        let level: InputLevel = self.store.get_value(INPUT_LEVEL);

        let n = output_id;

        #[cfg(not(test))]
        println!(
            "DEBUG: output at level:{} id:{} - \n{}",
            level,
            output_id,
            String::from_utf8_lossy(output.as_slice())
        );

        let output_path = format!("/output/{}/{}", level, n);
        self.store.set_value(&output_path, output);

        WriteResult::Ok
    }

    /// Simulates input ticks for adding inputs at chosen level.
    ///
    /// # Panics
    /// Panics if
    /// - `self.ready_for_input(at_level) == false`
    /// - `input_type == MessageData && payload.len() > 4KB`
    /// - `input_type == Slot && payload.len() > 1MB`
    ///
    /// For each input, sets the following:
    /// ```markdown
    /// /input/<level>/<id>/payload = <payload of the next input>
    /// /input/<level>/<id>/type := <type of the next input>
    /// /input/<level>/<id>/n = <index of the first chunk of this payload>
    /// /input/<level>/size :+ 1
    /// ```
    pub fn add_next_inputs<'a>(
        &mut self,
        at_level: InputLevel,
        inputs: impl Iterator<Item = &'a (Input, Vec<u8>)>,
    ) {
        if !self.ready_for_input(at_level) {
            panic!(
                "host not ready for input at level:{} | current level:{}",
                at_level,
                self.store.get_value::<InputLevel>(INPUT_LEVEL)
            )
        }

        let mut id = 0;
        let mut level_size = 0;

        for (input_type, payload) in inputs {
            let mut payload = payload.as_slice();

            let chunk_size = match (&input_type, payload.len()) {
                (Input::MessageData, l) if l > MAX_HEADER_SIZE => {
                    panic!("input header too big -size:{} -max:{}", l, MAX_HEADER_SIZE)
                }
                (Input::SlotDataChunk, l) if l > MAX_SLOT_SIZE => {
                    panic!("input slot too big -size:{} -max:{}", l, MAX_SLOT_SIZE)
                }
                (_, 0) => panic!("input empty"),
                (Input::MessageData, _) => MAX_INPUT_MESSAGE_SIZE,
                (Input::SlotDataChunk, _) => MAX_INPUT_SLOT_DATA_CHUNK_SIZE,
            };

            let n = id;
            while !payload.is_empty() {
                let next_payload = if payload.len() > chunk_size {
                    let (left, right) = payload.split_at(chunk_size);
                    payload = right;

                    left
                } else {
                    let temp = payload;
                    payload = &[];
                    temp
                };

                self.store.set_value(
                    &format!("/input/{}/{}/payload", at_level, id),
                    next_payload.to_vec(),
                );
                self.store
                    .set_value(&format!("/input/{}/{}/type", at_level, id), *input_type);
                self.store
                    .set_value(&format!("/input/{}/{}/n", at_level, id), n);

                id += 1;
                level_size += 1;
            }
        }

        self.store
            .set_value(&format!("/input/{}/size", at_level), level_size);
    }

    /// Returns whether it is safe to call `add_next_inputs` without triggering a panic.
    ///
    /// Returns *true* if all the following are *true*:
    /// - no previous inputs have been added at `/input/level`
    /// - `/input/level >= level`
    /// - `/checkpoints == <CHECKPOINTS_PER_LEVEL>`
    /// - `/input/consuming == true`
    ///
    /// Returns `false` otherwise.
    pub fn ready_for_input(&self, at_level: InputLevel) -> bool {
        let next_input_level = self.input_levels.front();

        let curr_level: InputLevel = self.store.get_value(INPUT_LEVEL);
        let is_consuming: InputConsuming =
            self.store.maybe_get_value(INPUT_CONSUMING).unwrap_or(false);

        Some(&at_level) == next_input_level
            && curr_level >= at_level
            && self.checkpoints() == CHECKPOINTS_PER_LEVEL
            && is_consuming
    }

    fn checkpoints(&self) -> Checkpoints {
        let checkpoints: Checkpoints = self.store.get_value(CHECKPOINTS);

        if checkpoints > CHECKPOINTS_PER_LEVEL {
            trap(HostFailure(TooManyCheckpointsInCommitment));
        }

        checkpoints
    }

    /// Handles a kernel yield - returning the next action to be taken by the runtime.
    ///
    /// ```markdown
    /// /reboot is set
    /// -----------------------
    /// (trigger Boot Sequence)
    ///
    /// /reboot is not set
    /// -----------------------
    ///
    ///   /checkpoints != <CHECKPOINTS_PER_LEVEL>
    ///   -------------------------------------------------
    ///   /input/checkpoints  += 1
    ///
    ///   /checkpoints = <CHECKPOINTS_PER_LEVEL>
    ///   -------------------------------------------
    ///
    ///     /input/consuming is not set
    ///     ---------------------------
    ///       /input/level     += 1
    ///       /input/consuming := True
    ///
    ///     /input/consuming is set
    ///     -------------------------
    ///       /input/level >= <level of the next message>
    ///       -------------------------------------------
    ///       (handle add_next_inputs)
    ///
    ///       /input/level < <level of the next message>
    ///       -------------------------------------------------
    ///       /gas/remaining      =  <CHECKPOINT_SIZE_IN_TICKS>
    ///       /checkpoints        := 1
    ///       /input/id           := 0
    ///       Remove /input/consuming
    /// ```
    ///
    /// See [`HostState::add_next_inputs`].
    pub fn handle_yield(&mut self) -> YieldStep {
        if let Some(1) = self.store.maybe_get_value::<Reboot>(REBOOT) {
            return YieldStep::Reboot;
        }

        if self.checkpoints() != CHECKPOINTS_PER_LEVEL {
            self.store
                .update_value(CHECKPOINTS, |checkpoints: Checkpoints| checkpoints + 1);

            return YieldStep::Trampoline;
        }

        let consuming = self
            .store
            .maybe_get_value::<InputConsuming>(INPUT_CONSUMING);

        if consuming.is_none() {
            self.store
                .update_value(INPUT_LEVEL, |level: InputLevel| level + 1);
            self.store.set_value(INPUT_CONSUMING, true);

            return YieldStep::HandleYield;
        }

        let level: InputLevel = self.store.get_value(INPUT_LEVEL);
        let next_input_level = self.input_levels.front();

        if next_input_level.is_none() {
            return YieldStep::MarkLevelForInput(level);
        }

        let next_level = next_input_level.unwrap();
        let next_level_size = self
            .store
            .maybe_get_value::<i32>(&format!("/input/{}/size", next_level));

        if level == *next_level && next_level_size.is_none() {
            return YieldStep::InputTicks(level);
        }

        if level <= *next_level {
            self.store.set_value::<Checkpoints>(CHECKPOINTS, 1);
            self.store.set_value::<InputId>(INPUT_ID, 0);
            self.store.delete_value(INPUT_CONSUMING);
            return YieldStep::Trampoline;
        }

        panic!("Handle yield called on inconsistent state {:?}", &self);
    }

    /// Return the next input
    ///
    /// ```markdown
    /// /input/id < /input/<level>/size
    /// -------------------------------
    /// /input/id += 1
    ///
    /// /input/id >= /input/<level>/size
    /// --------------------------------
    /// (return 0, no changes)
    /// ```
    pub(crate) fn handle_read_input(&mut self, max_bytes: usize) -> Option<NextInput> {
        let input_id: InputId = self.store.get_value(INPUT_ID);
        let level: InputLevel = self.store.get_value(INPUT_LEVEL);
        let level_size: Option<i32> = self
            .store
            .maybe_get_value(&format!("/input/{}/size", level));

        match level_size {
            Some(level_size) if input_id < level_size => {
                let input_type = self
                    .store
                    .get_value(&format!("/input/{}/{}/type", level, input_id));

                let id = self
                    .store
                    .get_value(&format!("/input/{}/{}/n", level, input_id));

                let mut payload: Vec<u8> = self
                    .store
                    .get_value(&format!("/input/{}/{}/payload", level, input_id));
                payload.truncate(max_bytes);

                self.store.set_value(INPUT_ID, input_id + 1);

                Some(NextInput {
                    input_type,
                    level,
                    id,
                    payload,
                })
            }
            Some(level_size) if input_id == level_size => {
                // Inputs have been consumed from the current input level
                assert_eq!(Some(level), self.input_levels.pop_front());
                // Technically this breaks with how the doc, but we use this to
                // specify the behaviour where we clean up the current level from
                // input_levels
                self.store.set_value(INPUT_ID, input_id + 1);
                None
            }
            _ => None,
        }
    }

    /// Retrieves and returns a previously stored preimage from its cryptographic hash
    ///
    /// If the `preimage.len() > MAX_BYTES`, then only its first `MAX_BYTES`
    /// are returned.
    ///
    /// # Panics
    /// Panics if
    /// -  There is no stored preimage bound to `hash`
    pub(crate) fn handle_reveal_preimage(
        &self,
        hash: &[u8; PREIMAGE_HASH_SIZE],
        max_bytes: usize,
    ) -> &[u8] {
        let preimage = self.store.retrieve_preimage(hash);
        if preimage.len() < max_bytes {
            preimage
        } else {
            &preimage[0..max_bytes]
        }
    }

    /// Stores a preimage, returning its cryptographic hash.
    ///
    /// The preimage is stored in a hash-indexed map.
    /// The key used to store the preimage is the
    /// cryptographic hash of the latter. The hash is computed
    /// using `crypto::blake2b::digest_256`
    ///
    /// # Panics
    /// Panics if
    /// - `preimage.len() > 4096`
    /// - the hash of the preimage cannot be computed
    /// - the size of the hash of the preimage is not PREIMAGE_HASH_SIZE bytes
    pub fn set_preimage(&mut self, preimage: Vec<u8>) -> [u8; PREIMAGE_HASH_SIZE] {
        self.store.add_preimage(preimage)
    }

    /// Returns whether the given key exists in storage, under the `durable` prefix.
    ///
    /// ```markdown
    /// ------------
    /// (no changes)
    /// ```
    ///
    /// # Traps
    /// Traps if the given bytes are not a valid [path].
    ///
    /// [path]: host::path
    pub fn handle_store_has(&self, raw_path: &[u8]) -> ValueType {
        let path = with_durable(raw_path);

        let has_value = self.store.has_entry(&path);
        let has_subvalue =
            self.handle_store_list_size(raw_path) > (if has_value { 1 } else { 0 });

        match (has_value, has_subvalue) {
            (false, false) => ValueType::None,
            (true, false) => ValueType::Value,
            (false, true) => ValueType::Subtree,
            (true, true) => ValueType::ValueWithSubtree,
        }
    }

    /// Read up to `num_bytes` starting at `offset` from the given key into memory.
    ///
    /// The key is prefixed with the `durable` prefix.
    /// If `num_bytes > 4096` then only a maximum of `4096` bytes will be returned.
    ///
    /// ```markdown
    /// ------------
    /// (no changes)
    /// ```
    ///
    /// # Traps
    /// Traps if the key is an invalid path, doesn't exist, or if the offset is out
    /// of bounds of the value.
    pub fn handle_store_read(
        &self,
        path: &[u8],
        offset: usize,
        max_bytes: usize,
    ) -> Vec<u8> {
        const MAX_READ_SIZE: usize = 4096;

        let path = with_durable(path);

        if !self.store.has_entry(&path) {
            trap(KernelFailure(KernelError::PathNotFound(path)));
        }

        let bytes: Vec<u8> = self.store.get_value(&path);
        if offset > bytes.len() {
            trap(KernelFailure(KernelError::OffsetOutOfBounds(
                offset,
                bytes.len(),
            )));
        }

        let num_bytes =
            usize::min(MAX_READ_SIZE, usize::min(max_bytes, bytes.len() - offset));
        let mut value = Vec::with_capacity(num_bytes);

        value.extend_from_slice(&bytes[offset..(offset + num_bytes)]);
        value
    }

    /// Write `bytes` into the value at `path`, starting from `offset` in the value.
    ///
    /// The key is prefixed with the `durable` prefix.
    ///
    /// If `bytes.len() > 4096` then a [`WriteResult::TooLarge`] will be returned.
    ///
    /// ```markdown
    /// ------------
    /// (bytes written to `path` at `offset`)
    /// ```
    ///
    /// # Traps
    /// Traps if the key is an invalid path, doesn't exist, or if the offset is out
    /// of bounds of the value.
    pub fn handle_store_write(
        &mut self,
        path: &[u8],
        offset: usize,
        bytes: &[u8],
    ) -> WriteResult {
        const MAX_WRITE_SIZE: usize = 4096;

        if bytes.len() > MAX_WRITE_SIZE {
            return WriteResult::TooLarge;
        }

        let path = with_durable(path);

        let mut value: Vec<u8> = match self.store.maybe_get_value(&path) {
            Some(value) => value,
            // No value, so only valid offset is zero (ie writing a new value).
            None => Vec::with_capacity(bytes.len()),
        };

        if offset > value.len() {
            trap(KernelFailure(KernelError::OffsetOutOfBounds(
                offset,
                value.len(),
            )));
        } else if offset < value.len() && (offset + bytes.len()) <= value.len() {
            let _ = value
                .splice(offset..(offset + bytes.len()), bytes.iter().copied())
                .collect::<Vec<_>>();
        } else {
            value.truncate(offset);
            value.extend_from_slice(bytes);
        };

        self.store.set_value(&path, value);

        WriteResult::Ok
    }

    /// Delete all subkeys of `prefix`, and `prefix` itself, it it exists.
    ///
    /// The key is prefixed with the `durable` prefix.
    ///
    /// ```markdown
    /// ------------
    /// (all subkeys of prefix removed)
    /// ```
    ///
    /// # Traps
    /// Traps if the key is an invalid path.
    pub fn handle_store_delete(&mut self, prefix: &[u8]) {
        let durable_prefix = with_durable(prefix);

        let keys = self
            .subkeys_of(prefix)
            .map(|subkey| format!("{}{}", durable_prefix, subkey))
            .collect::<Vec<_>>();

        for key in keys.iter() {
            self.store.delete_value(key);
        }
    }

    /// Get the number of subkeys by prefix.
    ///
    /// The key is prefixed with the `durable` prefix.
    ///
    /// ```markdown
    /// ------------
    /// (no changes)
    /// ```
    ///
    /// # Traps
    /// Traps if the key is an invalid path.
    pub fn handle_store_list_size(&self, prefix: &[u8]) -> i64 {
        self.subkeys_of(prefix)
            .count()
            .try_into()
            .expect("Host contains more than i64::MAX subkeys")
    }

    /// Get the subkey of `prefix` at the given `index`.
    ///
    /// The key is prefixed with the `durable` prefix.
    /// Subkeys are ordered alphabetically.
    ///
    /// ```markdown
    /// ------------
    /// (no changes)
    /// ```
    ///
    /// # Traps
    /// Traps if the key is an invalid path, or if the index is out of bounds of
    /// the subkeys.
    pub fn handle_store_list_get(&self, prefix: &[u8], index: i64) -> &str {
        let mut keys = self.subkeys_of(prefix).collect::<Vec<_>>();
        let num_keys = keys.len();

        let trap_index = |index: i64| {
            trap(KernelFailure(KernelError::PrefixSubkeyIndexOutOfBounds {
                prefix: std::str::from_utf8(prefix).unwrap().to_string(),
                subkey_count: num_keys,
                given_index: index,
            }));
        };
        let idx: usize = index.try_into().unwrap_or_else(|_| {
            trap_index(index);
            0
        });

        keys.sort_unstable();

        keys.get(idx).unwrap_or_else(|| {
            trap_index(index);
            &""
        })
    }

    /// Move one part of durable storage to another location.
    ///
    /// Everything is moved at once - not iteratively, so it is possible to
    /// move one part of storage to a location path that has the source as
    /// prefix.
    pub fn handle_store_move(&mut self, from_path: &[u8], to_path: &[u8]) {
        let from_durable_prefix = with_durable(from_path);
        let to_durable_prefix = with_durable(to_path);

        let keys = self
            .subkeys_of(from_path)
            .map(|subkey| {
                (
                    format!("{}{}", from_durable_prefix, subkey),
                    format!("{}{}", to_durable_prefix, subkey),
                )
            })
            .collect::<Vec<_>>();

        let mut temp_store: Vec<(String, Vec<u8>)> = Vec::new();

        for key in keys.iter() {
            let value = self.store.get_value(&key.0);
            temp_store.push((key.1.to_string(), value));
        }

        for key in keys.iter() {
            self.store.delete_value(&key.0);
        }

        self.handle_store_delete(to_path);

        for (key, value) in temp_store {
            self.store.set_value(&key, value);
        }
    }

    /// Copy one part of durable storage to another location.
    ///
    /// Similarly to `store_move`, everything is moved at once - not iteratively,
    /// so it is possible to copy one part of storage to another location where
    /// source and destination has overlap.
    pub fn handle_store_copy(&mut self, from_path: &[u8], to_path: &[u8]) {
        let from_durable_prefix = with_durable(from_path);
        let to_durable_prefix = with_durable(to_path);

        let keys = self
            .subkeys_of(from_path)
            .map(|subkey| {
                (
                    format!("{}{}", from_durable_prefix, subkey),
                    format!("{}{}", to_durable_prefix, subkey),
                )
            })
            .collect::<Vec<_>>();

        let mut temp_store: Vec<(String, Vec<u8>)> = Vec::with_capacity(keys.len());

        for key in keys.iter() {
            let value = self.store.get_value(&key.0);
            temp_store.push((key.1.to_string(), value));
        }

        self.handle_store_delete(to_path);

        for (key, value) in temp_store {
            self.store.set_value(&key, value);
        }
    }

    // Return an iterator over the subkeys of the given prefix.
    fn subkeys_of(&self, prefix: &[u8]) -> impl Iterator<Item = &str> {
        use host::path::PATH_SEPARATOR;

        let prefix = with_durable(prefix);
        let prefix_len = prefix.len();
        let separator =
            std::str::from_utf8(&[PATH_SEPARATOR]).expect("PATH_SEPARATOR is valid utf8");

        self.store
            .list_paths()
            .filter(move |p| p.starts_with(&prefix))
            .map(move |p| &p[prefix_len..])
            .filter(move |p| (p.starts_with(&separator) || p.is_empty()))
    }
}

fn with_durable(s: &[u8]) -> String {
    let mut p = Vec::with_capacity(s.len() + DURABLE_STORAGE_PREFIX.size());
    p.extend_from_slice(DURABLE_STORAGE_PREFIX.as_bytes());
    p.extend_from_slice(s);
    if let Err(err) = RefPath::try_from(p.as_slice()) {
        trap(KernelFailure(KernelError::InvalidPath(err)));
    }

    String::from_utf8(p).expect("A valid path *must* be valid utf8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Debug;
    use store::{Store, StoreValue};

    #[test]
    fn add_inputs_when_ready() {
        let level: InputLevel = 10;

        // Arrange
        let mut state = HostState::default();

        // make ready for input
        state.store.set_value(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);
        state.store.set_value(INPUT_CONSUMING, true);
        state.store.set_value(INPUT_LEVEL, level);

        let mut pristine = state.clone();

        let inputs = vec![
            (Input::MessageData, vec![4; MAX_INPUT_MESSAGE_SIZE / 2]),
            (
                Input::SlotDataChunk,
                vec![3; MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 4],
            ),
            (Input::MessageData, vec![2; MAX_HEADER_SIZE]),
            (
                Input::SlotDataChunk,
                vec![
                    1;
                    MAX_INPUT_SLOT_DATA_CHUNK_SIZE * 2
                        + MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 2
                ],
            ),
            (Input::MessageData, vec![0; MAX_INPUT_MESSAGE_SIZE / 8]),
        ];

        // Act
        state.mark_level_for_input(level);
        state.add_next_inputs(level, inputs.iter());

        // Assert

        // total number of chunks added
        assert_value_and_delete(
            &mut state.store,
            "/input/10/size",
            // sum of 'ceil(chunks)' in inputs.
            1 + 1 + 1 + 3 + 1,
        );

        // first chunk was a header
        assert_input_and_delete(
            &mut state.store,
            level,
            0,
            vec![4; MAX_INPUT_MESSAGE_SIZE / 2],
            Input::MessageData,
            0,
        );
        // second chunk was a complete slot
        assert_input_and_delete(
            &mut state.store,
            level,
            1,
            vec![3; MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 4],
            Input::SlotDataChunk,
            1,
        );
        // third chunk was a header
        assert_input_and_delete(
            &mut state.store,
            level,
            2,
            vec![2; MAX_INPUT_MESSAGE_SIZE],
            Input::MessageData,
            2,
        );
        // fourth chunk is the first chunk of a slot
        assert_input_and_delete(
            &mut state.store,
            level,
            3,
            vec![1; MAX_INPUT_SLOT_DATA_CHUNK_SIZE],
            Input::SlotDataChunk,
            3,
        );
        // fifth chunk is the second chunk of a slot
        assert_input_and_delete(
            &mut state.store,
            level,
            4,
            vec![1; MAX_INPUT_SLOT_DATA_CHUNK_SIZE],
            Input::SlotDataChunk,
            3,
        );
        // sixth chunk is the third chunk of a slot
        assert_input_and_delete(
            &mut state.store,
            level,
            5,
            vec![1; MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 2],
            Input::SlotDataChunk,
            3,
        );
        // seventh chunk is a header
        assert_input_and_delete(
            &mut state.store,
            level,
            6,
            vec![0; MAX_INPUT_MESSAGE_SIZE / 8],
            Input::MessageData,
            6,
        );

        // all paths due to adding inputs now removed, but we have marked a level
        // for input
        pristine.mark_level_for_input(10);
        assert_eq!(pristine, state);
    }

    #[test]
    fn read_next_inputs() {
        let level: InputLevel = 10;

        // Arrange
        let mut state = HostState::default();

        // make ready for input
        state.store.set_value(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);
        state.store.set_value(INPUT_CONSUMING, true);
        state.store.set_value(INPUT_LEVEL, level);
        state.store.set_value(INPUT_ID, 0);

        let inputs = vec![
            (Input::MessageData, vec![2; MAX_INPUT_MESSAGE_SIZE / 2]),
            (
                Input::SlotDataChunk,
                vec![
                    1;
                    MAX_INPUT_SLOT_DATA_CHUNK_SIZE * 2
                        + MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 2
                ],
            ),
            (Input::MessageData, vec![0; MAX_INPUT_MESSAGE_SIZE / 8]),
        ];

        // Act
        state.mark_level_for_input(level);
        state.add_next_inputs(level, inputs.iter());

        // Assert

        // first chunk was a header
        assert_eq!(
            Some(NextInput {
                level,
                id: 0,
                input_type: Input::MessageData,
                payload: vec![2; MAX_INPUT_MESSAGE_SIZE / 2]
            }),
            state.handle_read_input(MAX_INPUT_MESSAGE_SIZE)
        );

        // read three chunks from a slot - the last of which is incomplete
        assert_eq!(
            Some(NextInput {
                level,
                id: 1,
                input_type: Input::SlotDataChunk,
                payload: vec![1; MAX_INPUT_SLOT_DATA_CHUNK_SIZE]
            }),
            state.handle_read_input(MAX_INPUT_SLOT_DATA_CHUNK_SIZE)
        );
        assert_eq!(
            Some(NextInput {
                level,
                id: 1,
                input_type: Input::SlotDataChunk,
                payload: vec![1; MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 4]
            }),
            // We read less than the full chunk here
            state.handle_read_input(MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 4)
        );
        // But read still moves to the next chunk
        assert_eq!(
            Some(NextInput {
                level,
                id: 1,
                input_type: Input::SlotDataChunk,
                payload: vec![1; MAX_INPUT_SLOT_DATA_CHUNK_SIZE / 2]
            }),
            state.handle_read_input(MAX_INPUT_SLOT_DATA_CHUNK_SIZE)
        );

        assert_eq!(
            Some(NextInput {
                level,
                id: 4,
                input_type: Input::MessageData,
                payload: vec![0; MAX_INPUT_MESSAGE_SIZE / 8]
            }),
            state.handle_read_input(MAX_INPUT_MESSAGE_SIZE)
        );

        // We've run out of input for this level
        assert_eq!(None, state.handle_read_input(MAX_INPUT_MESSAGE_SIZE));
    }

    #[test]
    fn yield_trampoline_on_checkpoint() {
        // Arrange
        let starting_checkpoints: Checkpoints = 5;
        let mut state = HostState::default();

        state.store.set_value(CHECKPOINTS, starting_checkpoints);

        // Act
        let step = state.handle_yield();

        // Assert
        assert_eq!(YieldStep::Trampoline, step);

        assert_eq!(
            starting_checkpoints + 1,
            state.store.get_value::<Checkpoints>(CHECKPOINTS)
        );
    }

    #[test]
    fn yield_handle_yield_on_not_consuming() {
        // Arrange
        let level: InputLevel = 8;

        let mut state = HostState::default();

        state.store.set_value(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);
        state.store.set_value(INPUT_LEVEL, level);

        // Act
        let step = state.handle_yield();

        // Assert
        assert_eq!(YieldStep::HandleYield, step);

        assert_eq!(
            true,
            state.store.get_value::<InputConsuming>(INPUT_CONSUMING)
        );

        assert_eq!(level + 1, state.store.get_value::<InputLevel>(INPUT_LEVEL));
    }

    #[test]
    fn yield_mark_for_input_on_consuming() {
        // Arrange
        let level: InputLevel = 8;

        let mut state = HostState::default();

        state.store.set_value(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);
        state.store.set_value(INPUT_LEVEL, level);
        state.store.set_value(INPUT_CONSUMING, true);

        // Act
        let step = state.handle_yield();

        // Assert
        assert_eq!(YieldStep::MarkLevelForInput(level), step);
    }

    #[test]
    fn yield_input_ticks_at_level() {
        // Arrange
        let level: InputLevel = 8;

        let mut state = HostState::default();

        state.store.set_value(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);
        state.store.set_value(INPUT_LEVEL, level);
        state.store.set_value(INPUT_CONSUMING, true);

        state.mark_level_for_input(level);

        // Act
        let step = state.handle_yield();

        // Assert
        assert_eq!(YieldStep::InputTicks(level), step);
    }

    #[test]
    fn yield_trampoline_once_input_consumed() {
        // Arrange
        let level: InputLevel = 8;

        let mut state = HostState::default();

        state.store.set_value(CHECKPOINTS, CHECKPOINTS_PER_LEVEL);
        state.store.set_value(INPUT_LEVEL, level);
        state.store.set_value(INPUT_CONSUMING, true);

        state.mark_level_for_input(level + 1);

        // Act
        let step = state.handle_yield();

        // Assert
        assert_eq!(YieldStep::Trampoline, step);

        assert_eq!(1, state.store.get_value::<Checkpoints>(CHECKPOINTS));
        assert_eq!(0, state.store.get_value::<InputId>(INPUT_ID));
        assert!(state
            .store
            .maybe_get_value::<InputConsuming>(INPUT_CONSUMING)
            .is_none());
    }

    #[test]
    fn store_write_new_path() {
        // Arrange
        let mut state = HostState::default();
        let path: &[u8] = b"/test/path";

        let written = vec![1, 2, 3, 4];

        // Act
        state.handle_store_write(path, 0, &written);

        // Assert
        assert_eq!(
            ValueType::Value,
            state.handle_store_has(&path),
            "Path previously written to"
        );
        assert_eq!(written, state.handle_store_read(&path, 0, 4096));
    }

    #[test]
    fn store_write_extend_path() {
        // Arrange
        let mut state = HostState::default();
        let path: &[u8] = b"/test/path";

        let written = vec![1, 2, 3, 4];
        state.handle_store_write(path, 0, &written);

        // Act
        state.handle_store_write(path, 4, &written);

        // Assert
        let expected = vec![1, 2, 3, 4, 1, 2, 3, 4];
        assert_eq!(
            ValueType::Value,
            state.handle_store_has(&path),
            "Path previously written to"
        );
        assert_eq!(expected, state.handle_store_read(&path, 0, 4096));
    }

    #[test]
    fn store_write_extend_from_within() {
        // Arrange
        let mut state = HostState::default();
        let path: &[u8] = b"/test/path";

        let written = vec![1, 2, 3, 4];
        state.handle_store_write(path, 0, &written);

        // Act
        state.handle_store_write(path, 2, &written);

        // Assert
        let expected = vec![1, 2, 1, 2, 3, 4];
        assert_eq!(
            ValueType::Value,
            state.handle_store_has(&path),
            "Path previously written to"
        );
        assert_eq!(expected, state.handle_store_read(&path, 0, 4096));
    }

    #[test]
    fn store_write_fully_within() {
        // Arrange
        let mut state = HostState::default();
        let path: &[u8] = b"/test/path";

        state.handle_store_write(path, 0, &[1, 2, 3, 4]);

        // Act
        state.handle_store_write(path, 1, &[9, 10]);

        // Assert
        let expected = vec![1, 9, 10, 4];
        assert_eq!(
            ValueType::Value,
            state.handle_store_has(&path),
            "Path previously written to"
        );
        assert_eq!(expected, state.handle_store_read(&path, 0, 4096));
    }

    #[test]
    fn store_delete() {
        // Arrange
        let mut state = HostState::default();
        let prefix = "/a/long/prefix";

        state.handle_store_write(prefix.as_bytes(), 0, &[]);

        for i in 0..10 {
            // subkey of prefix
            let subkey = format!("{}/{}", prefix, i);
            // not subkey as not a sub-path
            let almost_subkey = format!("{}{}", prefix, i);
            // completely different prefix
            let not_subkey = format!("/different/prefix/{}", i);

            state.handle_store_write(subkey.as_bytes(), 0, &[]);
            state.handle_store_write(almost_subkey.as_bytes(), 0, &[]);
            state.handle_store_write(not_subkey.as_bytes(), 0, &[]);
        }

        // Act
        state.handle_store_delete(prefix.as_bytes());

        // Assert
        assert_eq!(ValueType::None, state.handle_store_has(prefix.as_bytes()));
        assert_eq!(0, state.handle_store_list_size(prefix.as_bytes()));
    }

    #[test]
    fn store_list_size() {
        // Arrange
        let mut state = HostState::default();
        let prefix = "/a/long/prefix";

        for i in 0..10 {
            // subkey of prefix
            let subkey = format!("{}/{}", prefix, i);
            // not subkey as not a sub-path
            let almost_subkey = format!("{}{}", prefix, i);
            // completely different prefix
            let not_subkey = format!("/different/prefix/{}", i);

            state.handle_store_write(subkey.as_bytes(), 0, &[]);
            state.handle_store_write(almost_subkey.as_bytes(), 0, &[]);
            state.handle_store_write(not_subkey.as_bytes(), 0, &[]);
        }

        // Act
        let result = state.handle_store_list_size(prefix.as_bytes());

        // Assert
        assert_eq!(10, result, "Expected 10 subkeys of prefix");
    }

    #[test]
    fn store_list_get() {
        // Arrange
        let mut state = HostState::default();
        let prefix = "/a/long/prefix";

        for i in 0..10 {
            // subkey of prefix
            let subkey = format!("{}/{}", prefix, i);
            // not subkey as not a sub-path
            let almost_subkey = format!("{}{}", prefix, i);
            // completely different prefix
            let not_subkey = format!("/different/prefix/{}", i);

            state.handle_store_write(subkey.as_bytes(), 0, &[]);
            state.handle_store_write(almost_subkey.as_bytes(), 0, &[]);
            state.handle_store_write(not_subkey.as_bytes(), 0, &[]);
        }

        // Act
        let result = state.handle_store_list_get(prefix.as_bytes(), 6);

        // Assert
        assert_eq!("/6", result, "Expected different subkey");
    }

    #[test]
    fn store_move() {
        // Arrange
        let mut state = HostState::default();
        state.handle_store_write(b"/a/b", 0, b"ab");
        state.handle_store_write(b"/a/b/c", 0, b"abc");
        state.handle_store_write(b"/a/b/c/z", 0, b"abcz");
        state.handle_store_write(b"/a/b/d", 0, b"abd");
        state.handle_store_write(b"/a/bc", 0, b"abc");

        // Act
        state.handle_store_move(b"/a/b", b"/a/b/c");

        // Assert
        assert_eq!(b"ab".to_vec(), state.handle_store_read(b"/a/b/c", 0, 4096));
        assert_eq!(
            b"abc".to_vec(),
            state.handle_store_read(b"/a/b/c/c", 0, 4096)
        );
        assert_eq!(
            b"abd".to_vec(),
            state.handle_store_read(b"/a/b/c/d", 0, 4096)
        );
        assert_eq!(b"abc".to_vec(), state.handle_store_read(b"/a/bc", 0, 4096));
        assert_eq!(ValueType::Subtree, state.handle_store_has(b"/a/b"));
        assert_eq!(ValueType::None, state.handle_store_has(b"/a/b/c/z"));
    }

    #[test]
    fn store_copy() {
        // Arrange
        let mut state = HostState::default();
        state.handle_store_write(b"/a/b", 0, b"ab");
        state.handle_store_write(b"/a/b/c", 0, b"abc");
        state.handle_store_write(b"/a/b/c/z", 0, b"abcz");
        state.handle_store_write(b"/a/b/d", 0, b"abd");
        state.handle_store_write(b"/a/bc", 0, b"abc");

        // Act
        state.handle_store_copy(b"/a/b", b"/a/b/c");

        // Assert
        assert_eq!(b"ab".to_vec(), state.handle_store_read(b"/a/b/c", 0, 4096));
        assert_eq!(
            b"abc".to_vec(),
            state.handle_store_read(b"/a/b/c/c", 0, 4096)
        );
        assert_eq!(
            b"abd".to_vec(),
            state.handle_store_read(b"/a/b/c/d", 0, 4096)
        );
        assert_eq!(b"abc".to_vec(), state.handle_store_read(b"/a/bc", 0, 4096));
        assert_eq!(b"ab".to_vec(), state.handle_store_read(b"/a/b", 0, 4096));
        assert_eq!(ValueType::None, state.handle_store_has(b"/a/b/c/z"));
    }

    fn assert_input_and_delete(
        store: &mut Store,
        level: InputLevel,
        idx: i32,
        payload: Vec<u8>,
        input_type: Input,
        id: i32,
    ) {
        assert_value_and_delete(
            store,
            &format!("/input/{}/{}/payload", level, idx),
            payload,
        );
        assert_value_and_delete(store, &format!("/input/{}/{}/n", level, idx), id);
        assert_value_and_delete(
            store,
            &format!("/input/{}/{}/type", level, idx),
            input_type,
        );
    }

    fn assert_value_and_delete<T: StoreValue + Eq + Debug>(
        store: &mut Store,
        path: &str,
        expected: T,
    ) {
        let v = store.get_value::<T>(path);

        assert_eq!(expected, v, "comparison for path {:?} failed", path);

        store.delete_value(path);
    }
}

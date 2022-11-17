//! Simulation of runtime trap conditions.
//!
//! These are runtime 'aborts' that result in the kernel being rebooted.

use host::path::PathError;

/// Trap conditions are either caused by errors in the **Host** or **Kernel**.
#[derive(Debug)]
pub enum TrapCondition {
    /// Failure condition due to invalid assumption by the mock runtime.
    HostFailure(HostError),
    /// Failure condition due to incorrect kernel behaviour.
    KernelFailure(KernelError),
}

/// The mock runtime (the *host*) makes certain assumptions about its own behaviour.
///
/// These errors occur when those assumptions breakdown.
#[derive(Debug)]
pub enum HostError {
    /// Typechecking error when reading to/from storage.
    ///
    /// The [host::rollup_core] apis are *untyped* - namely, they operate on byte slices.
    /// The mock runtime [`store`] also only stores these byte slices.
    ///
    /// The mock runtime assumes certain keys in storage have a specific type - which is
    /// converted to/from the byte slice when reading/writing to the store.
    ///
    /// [`store`]: mock_runtime::state::HostState::store.
    InvalidEncoding(&'static str),
    /// A host path was assumed to exist, but not found.
    ExistingPathNotFound(String),
    /// Host created too many checkpoints without creating a commitment.
    TooManyCheckpointsInCommitment,
}

/// The kernel caused a trap condition to be reached - usually due to incorrect api use.
#[derive(Debug)]
pub enum KernelError {
    /// The kernel may only write up to `u32::MAX` messages per level.
    TooManyOutputsWritten,
    /// Invalid path given by kernel in storage APIs
    InvalidPath(PathError),
    /// A valid path was given by the kernel, but did not exist in the store.
    PathNotFound(String),
    /// An offset into a value in durable storage was too large, compared to the size of
    /// the value in the store.
    OffsetOutOfBounds(usize, usize),
    /// An index into the subkeys of a prefix was out of bounds.
    PrefixSubkeyIndexOutOfBounds {
        /// The prefix of the subkeys lookup.
        prefix: String,
        /// The number of subkeys under the prefix.
        subkey_count: usize,
        /// The subkey index requested.
        given_index: i64,
    },
}

/// `trap` on a [`TrapCondition`].  The mock runtime behaviour is to panic on traps.
///
/// The actual semantics implemented by the PVM & Fast-execution is to reboot the kernel.
///
/// # Panics
/// `trap` always panics.
pub fn trap(on: TrapCondition) -> ! {
    panic!("Trap condition reached: {:?}", on)
}

//! The possible types that may be returned by (Runtime)[crate::runtime::Runtime] when reading an input.
//!
//! *N.B.* Reading input is currently only supported when the `alloc` feature is enabled.
#![cfg(feature = "alloc")]

use alloc::vec::Vec;

/// An input either comes from **Layer 1**, or a **Slot** the rollup is subscribed to.
#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    /// The input came from a slot of Layer 2.
    Slot(SlotData),
    /// The input came from a message on Layer 1.
    Message(MessageData),
}

/// An input from Layer 2 contains the inbox level, slot number, and message payload.
#[derive(Debug, PartialEq, Eq)]
pub struct SlotData {
    /// Inbox level of this message.
    pub level: i32,
    /// Slot Id corresponding to one of the slots to which the rollup is subscribed.
    pub id: i32,
    payload: Vec<u8>,
}

impl SlotData {
    /// Create a message input.
    pub const fn new(level: i32, id: i32, payload: Vec<u8>) -> Self {
        Self { level, id, payload }
    }
}

impl AsRef<[u8]> for SlotData {
    fn as_ref(&self) -> &[u8] {
        self.payload.as_ref()
    }
}

/// An input from Layer 2 contains the inbox level, message number, and message payload.
#[derive(Debug, PartialEq, Eq)]
pub struct MessageData {
    /// Inbox level of this message.
    pub level: i32,
    /// The message index in the Layer 1 inbox.
    pub id: i32,
    payload: Vec<u8>,
}

impl MessageData {
    /// Create a message input.
    pub const fn new(level: i32, id: i32, payload: Vec<u8>) -> Self {
        Self { level, id, payload }
    }
}

impl AsRef<[u8]> for MessageData {
    fn as_ref(&self) -> &[u8] {
        self.payload.as_ref()
    }
}

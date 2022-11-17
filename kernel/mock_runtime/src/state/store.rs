//! Mock runtime store - the container for host state.
use crate::trap::{trap, HostError::*, TrapCondition::*};
use crypto::blake2b::digest_256;
use host::rollup_core::{Input, PREIMAGE_HASH_SIZE};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Store {
    inner: HashMap<String, Vec<u8>>,
    preimages: HashMap<[u8; PREIMAGE_HASH_SIZE], Vec<u8>>,
}

impl Store {
    pub fn get_value<T: StoreValue>(&self, path: &str) -> T {
        let value = self
            .inner
            .get(path)
            .unwrap_or_else(|| trap(HostFailure(ExistingPathNotFound(path.into()))));

        T::from_bytes(value)
    }

    pub fn maybe_get_value<T: StoreValue>(&self, path: &str) -> Option<T> {
        self.inner.get(path).map(|v| T::from_bytes(v))
    }

    pub fn set_value<T: StoreValue>(&mut self, path: &str, value: T) {
        self.inner.insert(path.into(), value.to_bytes());
    }

    pub fn update_value<T: StoreValue>(
        &mut self,
        path: &str,
        update_fn: impl Fn(T) -> T,
    ) {
        if let Some(bytes) = self.inner.get_mut(path) {
            let value = T::from_bytes(bytes);
            let mut value = update_fn(value).to_bytes();

            std::mem::swap(bytes, &mut value);
        } else {
            trap(HostFailure(ExistingPathNotFound(path.to_string())));
        }
    }

    pub fn delete_value(&mut self, path: &str) {
        if self.inner.remove(path).is_none() {
            trap(HostFailure(ExistingPathNotFound(path.into())))
        }
    }

    pub fn has_entry(&self, path: &str) -> bool {
        self.inner.contains_key(path)
    }

    pub fn list_paths(&self) -> impl Iterator<Item = &String> {
        self.inner.keys()
    }

    pub fn add_preimage(&mut self, preimage: Vec<u8>) -> [u8; PREIMAGE_HASH_SIZE] {
        if preimage.len() > 4096 {
            panic!("Preimage limited to 4 KB, got {}", preimage.len())
        }
        let hash = digest_256(preimage.as_slice())
            .expect("hashing failed")
            .try_into()
            .expect("hash is incorrect length");
        self.preimages.insert(hash, preimage);
        hash
    }

    pub fn retrieve_preimage(&self, hash: &[u8; PREIMAGE_HASH_SIZE]) -> &[u8] {
        self.preimages
            .get(hash)
            .expect("Cannot retrieve preimage")
            .as_ref()
    }
}

impl AsRef<HashMap<String, Vec<u8>>> for Store {
    fn as_ref(&self) -> &HashMap<String, Vec<u8>> {
        &self.inner
    }
}

pub trait StoreValue {
    fn to_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl StoreValue for Vec<u8> {
    fn to_bytes(self) -> Vec<u8> {
        self
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes.to_vec()
    }
}

impl StoreValue for bool {
    fn to_bytes(self) -> Vec<u8> {
        match self {
            true => vec![b't'],
            false => vec![b'f'],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes.first() {
            Some(b't') => true,
            Some(b'f') => false,
            _ => trap(HostFailure(InvalidEncoding("bool"))),
        }
    }
}

impl StoreValue for usize {
    fn to_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes = bytes
            .try_into()
            .unwrap_or_else(|_| trap(HostFailure(InvalidEncoding("usize"))));
        usize::from_be_bytes(bytes)
    }
}

impl StoreValue for u32 {
    fn to_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes = bytes
            .try_into()
            .unwrap_or_else(|_| trap(HostFailure(InvalidEncoding("u32"))));
        u32::from_be_bytes(bytes)
    }
}

impl StoreValue for i32 {
    fn to_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes = bytes
            .try_into()
            .unwrap_or_else(|_| trap(HostFailure(InvalidEncoding("i32"))));
        i32::from_be_bytes(bytes)
    }
}

impl StoreValue for u8 {
    fn to_bytes(self) -> Vec<u8> {
        vec![self]
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            [byte] => *byte,
            _ => trap(HostFailure(InvalidEncoding("byte"))),
        }
    }
}

impl StoreValue for Input {
    fn to_bytes(self) -> Vec<u8> {
        match self {
            Input::MessageData => vec![b'h'],
            Input::SlotDataChunk => vec![b's'],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        match bytes.first() {
            Some(b'h') => Input::MessageData,
            Some(b's') => Input::SlotDataChunk,
            _ => trap(HostFailure(InvalidEncoding("Input"))),
        }
    }
}

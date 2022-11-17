use crypto::hash::Layer2Tz4Hash;
use tezos_encoding::nom::NomReader;
use tezos_encoding::encoding::HasEncoding;

pub mod sendable;
pub mod v1;

#[derive(Debug, PartialEq, Eq)]
pub struct ExternalInboxMessage<'a>(pub &'a [u8]);

// Signer

#[derive(Debug, Clone, PartialEq, Eq, NomReader, HasEncoding)]
pub enum Signer {
    Layer2Address(Layer2Tz4Hash),
}
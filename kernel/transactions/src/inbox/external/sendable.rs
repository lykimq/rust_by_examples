use super::v1;
use tezos_encoding_derive::{ HasEncoding };

#[derive(Debug, PartialEq, HasEncoding)]
pub enum ExternalInboxMessage {
    // version 1 of operation batching
    V1(v1::sendable::Batch),
}
use tezos_encoding_derive::{ HasEncoding };

pub use super::{ external::sendable::ExternalInboxMessage, InternalInboxMessage };

#[derive(Debug, PartialEq, HasEncoding)]
pub enum InboxMessage {
    Internal(InternalInboxMessage),
    External(ExternalInboxMessage),
}
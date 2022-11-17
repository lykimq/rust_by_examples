/* This is an inbox messages */

use crypto::hash::{ Layer2Tz4Hash };
use tezos_encoding::encoding::HasEncoding;
use tezos_encoding::nom::NomReader;
use nom::combinator::{ map, rest };
use crate::encoding::micheline::MichelineString;
use crate::encoding::michelson::MichelsonPair;
use crate::encoding::string_ticket::{ StringTicket, StringTicketRepr };

pub mod external;
pub mod sendable;

pub use self::external::*;

#[derive(Debug, PartialEq, Eq, NomReader, HasEncoding)]
enum InboxMessageRepr {
    Internal(InternalInboxMessage),
    External,
}

// Inbox message, received by the kernel as tezos-encoded bytes
// use Internal message only

#[derive(Debug, PartialEq, Eq)]
pub enum InboxMessage<'a> {
    // message sent from an L1 sm
    Internal(InternalInboxMessage),
    External(ExternalInboxMessage<'a>),
}

impl<'a> InboxMessage<'a> {
    pub fn parse(input: &'a [u8]) -> tezos_encoding::nom::NomResult<Self> {
        let (remaining, repr): (&'a [u8], _) = InboxMessageRepr::nom_read(input)?;

        match repr {
            InboxMessageRepr::Internal(i) => Ok((remaining, InboxMessage::Internal(i))),
            InboxMessageRepr::External =>
                map(rest, |ext| { InboxMessage::External(ExternalInboxMessage(ext)) })(remaining),
        }
    }
}

// Declare InternalInboxMessage
#[derive(Debug, PartialEq, Eq, NomReader, HasEncoding)]

// Define only payload and source
pub struct InternalInboxMessage {
    pub payload: InternalMessagePayloadRepr,
}

pub type InternalMessagePayloadRepr = MichelsonPair<MichelineString, StringTicketRepr>;

// deposit ticket
#[derive(Debug, PartialEq, Eq)]
pub struct InboxDeposit {
    pub destination: Layer2Tz4Hash,
    pub ticket: StringTicket,
}
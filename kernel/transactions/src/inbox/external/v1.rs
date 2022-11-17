use crypto::hash::{ Layer2Tz4Hash };
use crate::encoding::string_ticket::StringTicketRepr;
use tezos_encoding::encoding::HasEncoding;
use verifiable::VerifiableTransaction;
use nom::multi::many1;
use nom::combinator::map;
use tezos_encoding::nom::{ dynamic, NomReader };

use super::Signer;

pub mod sendable;
pub mod verifiable;

// transfer
#[derive(Debug, PartialEq, Eq, HasEncoding, NomReader)]
pub struct OperationTransfer {
    destination: Layer2Tz4Hash,
    ticket: StringTicketRepr,
}

// an operation transfer ticket first
#[derive(Debug, PartialEq, Eq, HasEncoding, NomReader)]
pub enum OperationContent {
    Transfer(OperationTransfer),
}

impl OperationContent {
    // create a new transfer operation
    pub fn transfer(
        destination: Layer2Tz4Hash,
        ticket: impl Into<StringTicketRepr>
    ) -> OperationContent {
        OperationContent::Transfer(OperationTransfer {
            destination,
            ticket: ticket.into(),
        })
    }
}

// operation
#[derive(Debug, PartialEq, Eq, HasEncoding, NomReader)]
pub struct Operation {
    pub signer: Signer,
    pub counter: i64,
    pub contents: Vec<OperationContent>,
}

// A patch of operations, associated with an aggregated signature
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedBatch<'a> {
    pub transactions: Vec<VerifiableTransaction<'a>>,
}

impl<'a> ParsedBatch<'a> {
    // parse a batch where each transaction is verifiable
    pub fn parse(input: &'a [u8]) -> tezos_encoding::nom::NomResult<Self> {
        map(dynamic(many1(VerifiableTransaction::parse)), |transactions| ParsedBatch {
            transactions,
        })(input)
    }
}
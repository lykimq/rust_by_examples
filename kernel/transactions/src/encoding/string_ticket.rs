use super::contract::Contract;
use super::micheline::{ MichelineInt, MichelineString };
use super::michelson::{ MichelsonContract, MichelsonPair };
use tezos_encoding::enc::{ self };
use crypto::blake2b::{ digest_256, Blake2bError };
use thiserror::Error;
use tezos_encoding::enc::{ BinWriter, BinError };

// The hash of a string ticket
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct StringTicketHash(Vec<u8>);

// Proof that a ticket-identiy matches a ticket

pub struct TrustlessTicketIdentity(StringTicketHash, StringTicket);

// errors occuring when identifying tickets
#[derive(Error, Debug)]
pub enum TicketHashError {
    #[error("Unable to serialize ticket for hashing: {0}")] Serialization(#[from] BinError),
    #[error("Unable to hash ticket bytes: {0}")] Hashing(#[from] Blake2bError),
}

impl TrustlessTicketIdentity {
    // Break the link between the identiy and the ticket
    pub fn consume(self) -> (StringTicketHash, StringTicket) {
        (self.0, self.1)
    }

    // Access the identity without breaking the trustless-link
    pub fn identify(&self) -> &StringTicketHash {
        &self.0
    }

    // Access the ticket without breaking the trustless-link
    pub fn ticket(&self) -> &StringTicket {
        &self.1
    }
}

/* Define String ticket repr */

pub(crate) type StringTicketRepr = MichelsonPair<
    MichelsonContract,
    MichelsonPair<MichelineString, MichelineInt>
>;

/* Define string ticket */

#[derive(Debug, PartialEq, Eq)]
pub struct StringTicket {
    pub(crate) creator: Contract,
    pub(crate) contents: String,
    pub(crate) amount: u64,
}

impl StringTicket {
    // Create a new string ticket

    pub fn new(creator: Contract, contents: String, amount: u64) -> Self {
        Self { creator, contents, amount }
    }

    // Return an identifying hash of the ticket creator and contents
    pub fn identify(&self) -> Result<StringTicketHash, TicketHashError> {
        let mut bytes = Vec::new();
        self.creator.bin_write(&mut bytes)?;
        enc::string(&self.contents, &mut bytes)?;
        let digest = digest_256(bytes.as_slice())?;
        Ok(StringTicketHash(digest))
    }

    pub fn identify_trustless(self) -> Result<TrustlessTicketIdentity, TicketHashError> {
        Ok(TrustlessTicketIdentity(self.identify()?, self))
    }

    // amount
    pub fn amount(&self) -> u64 {
        self.amount
    }
}
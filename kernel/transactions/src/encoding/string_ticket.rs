use super::contract::Contract;
use super::micheline::{ MichelineInt, MichelineString };
use super::michelson::{ MichelsonContract, MichelsonPair };

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
}
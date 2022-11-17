use super::{ Operation };
use tezos_encoding::encoding::{ Encoding, HasEncoding };
use tezos_encoding::has_encoding;

// transaction
#[derive(Debug, PartialEq, HasEncoding)]
pub struct Transaction {
    #[encoding(dynamic, list)]
    operations: Vec<Operation>,
}

impl Transaction {
    // Get the operations

    pub fn operations(&self) -> &[Operation] {
        self.operations.as_slice()
    }
}

// batch
#[derive(Debug, PartialEq)]
pub struct Batch {
    transactions: Vec<Transaction>,
}

has_encoding!(Batch, SENDABLE_BATCH_ENCODING, { Encoding::Custom });

impl Batch {
    // create a new batch from a list of transactions

    pub fn new(transactions: Vec<Transaction>) -> Self {
        Self { transactions }
    }
}
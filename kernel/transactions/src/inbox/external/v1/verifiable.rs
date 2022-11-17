use nom::multi::many1;
use nom::combinator::{ consumed, map };
use tezos_encoding::nom::{ dynamic, NomReader };
use crate::inbox::external::Signer;
use super::{ Operation };

#[derive(Debug, PartialEq, Eq, NomReader)]
pub struct VerifiableOperation {
    operation: Operation,
}

impl VerifiableOperation {
    fn signer(&self) -> &Signer {
        &self.operation.signer
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VerifiableTransaction<'a> {
    encoded: &'a [u8],
    operations: Vec<VerifiableOperation>,
}

impl<'a> VerifiableTransaction<'a> {
    pub fn parse(input: &'a [u8]) -> tezos_encoding::nom::NomResult<Self> {
        map(
            consumed(dynamic(many1(VerifiableOperation::nom_read))),
            |(encoded, operations)| VerifiableTransaction {
                encoded,
                operations,
            }
        )(input)
    }
}
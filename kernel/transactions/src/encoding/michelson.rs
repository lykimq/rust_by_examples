use super::contract::Contract;
use std::fmt::Debug;
use tezos_encoding::encoding::{ Encoding, HasEncoding };
use tezos_encoding::nom::{ NomReader, NomResult };
use nom::combinator::map;

use super::micheline::{ nom_read_micheline_bytes, MichelinePrim2ArgsNoAnnots };

use v1_primitives as prim;

pub mod v1_primitives {
    pub const PAIR_TAG: u8 = 7;
}

#[derive(Debug, PartialEq, Eq)]
pub struct MichelsonContract(pub Contract);

#[derive(Debug, PartialEq, Eq)]
pub struct MichelsonPair<Arg0, Arg1>(pub Arg0, pub Arg1)
    where Arg0: Debug + PartialEq + Eq, Arg1: Debug + PartialEq + Eq;

// Encoding

impl HasEncoding for MichelsonContract {
    fn encoding() -> Encoding {
        Encoding::Custom
    }
}

impl<Arg0, Arg1> HasEncoding
    for MichelsonPair<Arg0, Arg1>
    where Arg0: Debug + PartialEq + Eq, Arg1: Debug + PartialEq + Eq
{
    fn encoding() -> Encoding {
        Encoding::Custom
    }
}

// Decoding implement NomReader

impl NomReader for MichelsonContract {
    fn nom_read(input: &[u8]) -> NomResult<Self> {
        map(nom_read_micheline_bytes(Contract::nom_read), MichelsonContract)(input)
    }
}

impl<Arg0, Arg1> NomReader
    for MichelsonPair<Arg0, Arg1>
    where Arg0: NomReader + Debug + PartialEq + Eq, Arg1: NomReader + Debug + PartialEq + Eq
{
    fn nom_read(input: &[u8]) -> NomResult<Self> {
        map(MichelinePrim2ArgsNoAnnots::<_, _, { prim::PAIR_TAG }>::nom_read, Into::into)(input)
    }
}

impl<Arg0, Arg1> From<MichelinePrim2ArgsNoAnnots<Arg0, Arg1, { prim::PAIR_TAG }>>
    for MichelsonPair<Arg0, Arg1>
    where Arg0: Debug + PartialEq + Eq, Arg1: Debug + PartialEq + Eq
{
    fn from(micheline: MichelinePrim2ArgsNoAnnots<Arg0, Arg1, { prim::PAIR_TAG }>) -> Self {
        Self(micheline.arg1, micheline.arg2)
    }
}

impl<Arg0, Arg1> From<MichelsonPair<Arg0, Arg1>>
    for MichelinePrim2ArgsNoAnnots<Arg0, Arg1, { prim::PAIR_TAG }>
    where Arg0: Debug + PartialEq + Eq, Arg1: Debug + PartialEq + Eq
{
    fn from(michelson: MichelsonPair<Arg0, Arg1>) -> Self {
        Self {
            arg1: michelson.0,
            arg2: michelson.1,
        }
    }
}
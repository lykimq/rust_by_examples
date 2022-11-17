use tezos_encoding::types::Zarith;
use tezos_encoding::encoding::{ Encoding, HasEncoding };
use tezos_encoding::has_encoding;
use tezos_encoding::nom::{ self as nom_read, NomInput, NomReader, NomResult };
use nom::sequence::{ pair, preceded };
use nom::combinator::map;
use nom::bytes::complete::tag;
use std::fmt::Debug;

pub const MICHELINE_PRIM_2_ARGS_NO_ANNOTS_TAG: u8 = 7;
pub const MICHELINE_BYTES_TAG: u8 = 10;
pub const MICHELINE_INT_TAG: u8 = 0;
pub const MICHELINE_STRING_TAG: u8 = 1;

#[derive(Debug, PartialEq, Eq)]
pub struct MichelinePrim2ArgsNoAnnots<Arg1, Arg2, const PRIM_TAG: u8>
    where Arg1: Debug + PartialEq + Eq, Arg2: Debug + PartialEq + Eq {
    pub(crate) arg1: Arg1,
    pub(crate) arg2: Arg2,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MichelineInt(pub Zarith);

#[derive(Debug, PartialEq, Eq)]
pub struct MichelineString(pub String);

//Conversion
impl From<i32> for MichelineInt {
    fn from(int: i32) -> Self {
        MichelineInt(Zarith(int.into()))
    }
}

// Encoding
has_encoding!(MichelineInt, MICHELINE_INT_ENCODING, { Encoding::Custom });
has_encoding!(MichelineString, MICHELINE_STRING_ENCODING, { Encoding::Custom });

// Deserialization combinators
pub fn nom_read_micheline_bytes<'a, T: Clone>(
    parser: impl FnMut(NomInput) -> NomResult<'a, T>
) -> impl FnMut(NomInput<'a>) -> NomResult<'a, T> {
    preceded(tag([MICHELINE_BYTES_TAG]), nom_read::dynamic(parser))
}

// Nom reader

impl NomReader for MichelineInt {
    fn nom_read(input: &[u8]) -> NomResult<Self> {
        map(preceded(tag([MICHELINE_INT_TAG]), Zarith::nom_read), MichelineInt)(input)
    }
}

impl NomReader for MichelineString {
    fn nom_read(input: &[u8]) -> NomResult<Self> {
        map(preceded(tag([MICHELINE_STRING_TAG]), nom_read::string), MichelineString)(input)
    }
}

impl<Arg1, Arg2, const PRIM_TAG: u8> NomReader
    for MichelinePrim2ArgsNoAnnots<Arg1, Arg2, PRIM_TAG>
    where Arg1: NomReader + Debug + PartialEq + Eq, Arg2: NomReader + Debug + PartialEq + Eq
{
    fn nom_read(input: &[u8]) -> NomResult<Self> {
        let parse = preceded(
            tag([MICHELINE_PRIM_2_ARGS_NO_ANNOTS_TAG, PRIM_TAG]),
            pair(Arg1::nom_read, Arg2::nom_read)
        );

        map(parse, |(arg1, arg2)| MichelinePrim2ArgsNoAnnots {
            arg1,
            arg2,
        })(input)
    }
}
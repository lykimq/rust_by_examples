use nom::combinator::map;
use nom::bytes::complete::tag;
use nom::sequence::preceded;
use tezos_encoding::nom::{ NomReader, NomResult };
use tezos_encoding::enc::{ self, BinResult, BinWriter };
use super::public_key_hash::PublicKeyHash;

// Create contract

#[derive(Debug, Clone, PartialEq, Eq)]
// Use only implicit
pub enum Contract {
    Implicit(PublicKeyHash),
}

impl Contract {
    // convert from base58-encoded string, checking for the prefix
    /*
    pub fn from_b58check(data: &str) -> Result<Self, FromBase58CheckError> {
        let bytes = data.from_b58check()?;
        match bytes {
            _ => Ok(Self::Implicit(PublicKeyHash::from_b58check(data)?)),
        }
    }*/

    // convert to a b58-encoding string, including the prefix
    pub fn to_b58check(&self) -> String {
        match self {
            Self::Implicit(pkh) => pkh.to_b58check(),
        }
    }
}

// implement nomreader for contract
impl NomReader for Contract {
    fn nom_read(input: &[u8]) -> NomResult<Self> {
        map(preceded(tag([0]), PublicKeyHash::nom_read), Contract::Implicit)(input)
    }
}

// implement BinWrite for Contract

impl BinWriter for Contract {
    fn bin_write(&self, output: &mut Vec<u8>) -> BinResult {
        match self {
            Self::Implicit(implicit) => {
                enc::put_byte(&0, output);
                BinWriter::bin_write(implicit, output)
            }
        }
    }
}
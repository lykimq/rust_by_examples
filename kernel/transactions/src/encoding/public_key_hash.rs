use crypto::hash::{ ContractTz1Hash, Hash, HashTrait };
use tezos_encoding::encoding::HasEncoding;
use tezos_encoding::nom::NomReader;
use tezos_encoding::enc::BinWriter;

#[derive(Debug, Clone, PartialEq, Eq, HasEncoding, NomReader, BinWriter)]
pub enum PublicKeyHash {
    //tz1-contract
    Ed25519(ContractTz1Hash),
}

impl PublicKeyHash {
    // Convert to base58-encoding string (with prefix)

    pub fn to_b58check(&self) -> String {
        match self {
            Self::Ed25519(tz1) => tz1.to_b58check(),
        }
    }
}

impl From<PublicKeyHash> for Hash {
    fn from(pkh: PublicKeyHash) -> Self {
        match pkh {
            PublicKeyHash::Ed25519(tz1) => tz1.into(),
        }
    }
}
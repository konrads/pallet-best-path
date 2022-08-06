use core::fmt::Debug;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use sp_std::str;
use best_path::prelude::{Currency, Provider, ProviderPair};

pub trait Conversions {
    fn to_str(&self) -> &str;
    fn from_vecu8(vec: Vec<u8>) -> Self;
}

impl Conversions for Vec<u8> {
    fn to_str(&self) -> &str {
        str::from_utf8(self).ok().unwrap()
    }
    fn from_vecu8(vec: Vec<u8>) -> Self {
        vec
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub struct ProviderPairOperation<C: Currency, P: Provider> {
    pub provider_pair: ProviderPair<C, P>,
    pub operation: Operation,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum Operation {
	Add,
	Del,
}

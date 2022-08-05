use core::fmt::Debug;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use sp_std::str;

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

pub trait Currency: Clone + Ord {}
impl <T: Clone + Ord> Currency for T {}

pub trait Provider: Clone + Ord {}
impl <T: Clone + Ord> Provider for T {}

pub trait Amount: Copy + TryInto<u128> + TryFrom<u128> {}
impl<T: Copy + TryInto<u128> + TryFrom<u128>> Amount for T {}

/// Per provider, source and target currency. Represents price points from each provider
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo, Ord, PartialOrd)]
pub struct Pair<C: Currency> {
    pub source: C,
    pub target: C,
}

/// Per provider, source and target currency. Represents price points from each provider
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo, Ord, PartialOrd)]
pub struct ProviderPair<C: Currency, P: Provider> {
    pub pair: Pair<C>,
    pub provider: P,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub struct ProviderPairOperation<C: Currency, P: Provider> {
    pub provider_pair: ProviderPair<C, P>,
    pub operation: Operation,
}

/// Path for every ProviderPair. Consists of `hops` and overall cost
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub struct PricePath<C: Currency, A: Amount, P: Provider> {
    pub total_cost: A,
    pub steps: Vec<PathStep<C, A, P>>,
}

/// A `hop` between different currencies, via a provider.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub struct PathStep<C: Currency, A: Amount, P: Provider> {
    pub pair: Pair<C>,
    pub provider: P,
    pub cost: A,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, TypeInfo)]
pub enum Operation {
	Add,
	Del,
}

#[derive(Debug)]
pub enum CalculatorError {
    NegativeCyclesError,
    ConversionError,
}

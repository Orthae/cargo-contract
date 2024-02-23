// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use std::str::FromStr;

use ink_env::{
    DefaultEnvironment,
    Environment,
};
use sp_core::Pair;
use subxt::{
    config::PolkadotExtrinsicParams,
    tx::{
        PairSigner,
        Signer as SignerT,
    },
    utils::MultiAddress,
    Config,
    SubstrateConfig,
};

/// A runtime configuration for the ecdsa test chain.
/// This thing is not meant to be instantiated; it is just a collection of types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ecdsachain {}

impl Config for Ecdsachain {
    type Hash = <SubstrateConfig as Config>::Hash;
    type AccountId = <SubstrateConfig as Config>::AccountId;
    type Address = MultiAddress<Self::AccountId, ()>;
    type Signature = <SubstrateConfig as Config>::Signature;
    type Hasher = <SubstrateConfig as Config>::Hasher;
    type Header = <SubstrateConfig as Config>::Header;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
    type AssetId = <SubstrateConfig as Config>::AssetId;
}

impl Environment for Ecdsachain {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;
    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type ChainExtension = <DefaultEnvironment as Environment>::ChainExtension;
}

impl SignerConfig<Self> for Ecdsachain
where
    <Self as Config>::Signature: From<sp_core::ecdsa::Signature>,
{
    type Signer = SignerEcdsa<Self>;
}

/// A runtime configuration for the Polkadot based chain.
/// This thing is not meant to be instantiated; it is just a collection of types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Polkadot {}

impl Config for Polkadot {
    type Hash = <SubstrateConfig as Config>::Hash;
    type AccountId = <SubstrateConfig as Config>::AccountId;
    type Address = MultiAddress<Self::AccountId, ()>;
    type Signature = <SubstrateConfig as Config>::Signature;
    type Hasher = <SubstrateConfig as Config>::Hasher;
    type Header = <SubstrateConfig as Config>::Header;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
    type AssetId = <SubstrateConfig as Config>::AssetId;
}

impl Environment for Polkadot {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;
    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type ChainExtension = <DefaultEnvironment as Environment>::ChainExtension;
}

pub trait SignerConfig<C: Config> {
    type Signer: SignerT<C> + FromStr;
}

impl SignerConfig<Self> for Polkadot {
    type Signer = SignerSR25519<Self>;
}

/// Struct representing the implementation of the sr25519 signer
#[derive(Clone)]
pub struct SignerSR25519<C: Config>(pub PairSigner<C, sp_core::sr25519::Pair>);

impl<C: Config> FromStr for SignerSR25519<C>
where
    <C as Config>::AccountId: From<sp_core::crypto::AccountId32>,
{
    type Err = anyhow::Error;

    /// Attempts to parse the Signer suri string
    fn from_str(input: &str) -> Result<SignerSR25519<C>, Self::Err> {
        let keypair = sp_core::sr25519::Pair::from_string(input, None)?;
        let signer = PairSigner::<C, _>::new(keypair);
        Ok(Self(signer))
    }
}

impl<C: Config> SignerT<C> for SignerSR25519<C>
where
    <C as Config>::Signature: From<sp_core::sr25519::Signature>,
{
    fn account_id(&self) -> <C as Config>::AccountId {
        self.0.account_id().clone()
    }

    fn address(&self) -> C::Address {
        self.0.address()
    }

    fn sign(&self, signer_payload: &[u8]) -> C::Signature {
        self.0.sign(signer_payload)
    }
}

/// Struct representing the implementation of the ecdsa signer
#[derive(Clone)]
pub struct SignerEcdsa<C: Config>(pub PairSigner<C, sp_core::ecdsa::Pair>);

impl<C: Config> FromStr for SignerEcdsa<C>
where
    // Requirements of the `PairSigner where:
    // T::AccountId: From<SpAccountId32>`
    <C as Config>::AccountId: From<sp_core::crypto::AccountId32>,
{
    type Err = anyhow::Error;

    /// Attempts to parse the Signer suri string
    fn from_str(input: &str) -> Result<SignerEcdsa<C>, Self::Err> {
        let keypair = sp_core::ecdsa::Pair::from_string(input, None)?;
        let signer = PairSigner::<C, _>::new(keypair);
        Ok(Self(signer))
    }
}

impl<C: Config> SignerT<C> for SignerEcdsa<C>
where
    <C as Config>::Signature: From<sp_core::ecdsa::Signature>,
{
    fn account_id(&self) -> <C as Config>::AccountId {
        self.0.account_id().clone()
    }

    fn address(&self) -> C::Address {
        self.0.address()
    }

    fn sign(&self, signer_payload: &[u8]) -> C::Signature {
        self.0.sign(signer_payload)
    }
}

#[macro_export]
macro_rules! call_with_config_internal {
    ($obj:tt ,$function:tt, $config_name:expr, $($config:ty),*) => {
        match $config_name {
            $(
                stringify!($config) => $obj.$function::<$config>().await,
            )*
            _ => {

                let configs = vec![$(stringify!($config)),*].iter()
                .map(|s| s.trim_start_matches("crate::config::"))
                .collect::<Vec<_>>()
                .join(", ");
                Err(ErrorVariant::Generic(
                    contract_extrinsics::GenericError::from_message(
                        format!("Chain configuration not found, Allowed configurations: {configs}")
                )))
            },
        }
    };
}

/// Macro that allows calling the command member function with chain configuration
#[macro_export]
macro_rules! call_with_config {
    ($obj:tt, $function:ident, $config_name:expr) => {{
        let config_name = format!("crate::config::{}", $config_name);
        $crate::call_with_config_internal!(
            $obj,
            $function,
            config_name.as_str(),
            // All available chain configs need to be specified here
            $crate::config::Polkadot,
            $crate::config::Ecdsachain
        )
    }};
}

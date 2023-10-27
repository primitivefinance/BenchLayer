use arbiter_core::bindings::{
    arbiter_math::{self, ArbiterMath},
    arbiter_token::{self, ArbiterToken},
};

use anyhow::{Ok, Result};
use ethers::{
    providers::Middleware,
    types::{Address, I256, U256},
};

use std::sync::Arc;

use crate::bindings;

pub(crate) async fn lookup<M: Middleware + 'static>(token: ArbiterToken<M>) -> Result<()> {
    let address = token.client().default_sender().unwrap();
    token.balance_of(address).call().await?;
    Ok(())
}

pub(crate) async fn create_call<M: Middleware + 'static>(
    client: Arc<M>,
) -> Result<()> {
    bindings::counter::Counter::deploy(client.clone(), ()).unwrap().send().await?;
    Ok(())
}

pub(crate) async fn stateless_call<M: Middleware + 'static>(
    arbiter_math: ArbiterMath<M>,
) -> Result<()> {
    let iwad = I256::from(10_u128.pow(18));
    arbiter_math.cdf(iwad).call().await?;
    Ok(())
}

pub(crate) async fn stateful_call<M: Middleware + 'static>(
    arbiter_token: arbiter_token::ArbiterToken<M>,
    mint_address: Address,
) -> Result<()> {
    let wad = U256::from(10_u128.pow(18));
    arbiter_token.mint(mint_address, wad).send().await?.await?;
    Ok(())
}

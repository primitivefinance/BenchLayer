use arbiter_core::bindings::{
    arbiter_math::{self, ArbiterMath},
    arbiter_token::{self, ArbiterToken},
};

use anyhow::{Ok, Result};
use ethers::providers::Middleware;

use std::sync::Arc;
pub(crate) async fn lookup<M: Middleware + 'static>(token: ArbiterToken<M>) -> Result<()> {
    let address = token.client().default_sender().unwrap();
    token.balance_of(address).call().await?;
    Ok(())
}

pub(crate) async fn deployments<M: Middleware + 'static>(
    client: Arc<M>,
) -> Result<(ArbiterMath<M>, ArbiterToken<M>)> {
    let math = arbiter_math::ArbiterMath::deploy(client.clone(), ())?
        .send()
        .await?;
    let token = arbiter_token::ArbiterToken::deploy(
        client.clone(),
        ("Arbiter Token".to_string(), "ARBT".to_string(), 18_u8),
    )?
    .send()
    .await?;
    pub struct ArbiterMath<M>(::ethers::contract::Contract<M>);

    Ok((math, token))
}

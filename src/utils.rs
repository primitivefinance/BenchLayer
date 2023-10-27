// Todo allow for users to pick the contracts they want to benchmark with
use anyhow::{Ok, Result};
use arbiter_core::{environment::builder::EnvironmentBuilder, middleware::RevmMiddleware};
use ethers::{
    core::{k256::ecdsa::SigningKey, utils::Anvil},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer, Wallet},
};

use std::{convert::TryFrom, sync::Arc, time::Duration};

pub async fn get_middleware() -> Result<(
    Arc<RevmMiddleware>,
    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
)> {
    let arbiter = arbiter_startup()?;
    let anvil = anvil_startup().await?;
    Ok((arbiter, anvil))
}

fn arbiter_startup() -> Result<Arc<RevmMiddleware>> {
    let environment = EnvironmentBuilder::new().build();
    let client = RevmMiddleware::new(&environment, Some("name"))?;
    Ok(client)
}

async fn anvil_startup() -> Result<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    // Create an Anvil instance
    // No blocktime mines a new block for each tx, which is fastest.
    let anvil = Anvil::new().spawn();

    // Create a client
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::ZERO);

    let wallet: LocalWallet = anvil.keys()[0].clone().into();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.with_chain_id(anvil.chain_id()),
    ));
    Ok(client)
}

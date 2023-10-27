// Todo allow for users to pick the contracts they want to benchmark with

use anyhow::{Ok, Result};
use bench_functions::deployments;
use ethers::{
    providers::Middleware,
    types::{I256, U256},
};

use std::sync::Arc;

use crate::bench_functions::lookup;
use criterion::async_executor::FuturesExecutor;
use criterion::Criterion;

mod bench_functions;
mod utils;

pub async fn bench_middleware<M: Middleware + 'static>(
    c: &mut Criterion,
    client: Arc<M>,
    label: &str,
) -> Result<()> {
    let wad = U256::from(10_u128.pow(18));

    // these are the contracts we are benching against
    // this can happily change if people want to bench against more general contracts
    let (math_contract, token_contract) = deployments(client.clone()).await?;

    c.bench_function(&format!("{} Stateful Call", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            token_contract
                .mint(client.default_sender().unwrap(), wad)
                .send()
                .await
                .unwrap();
        })
    });

    c.bench_function(&format!("{} Stateless Call", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            math_contract
                .cdf(I256::from(10_u128.pow(18)))
                .call()
                .await
                .unwrap();
        })
    });
    c.bench_function(&format!(" {} Deploys", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            deployments(client.clone()).await.unwrap();
        })
    });
    c.bench_function(&format!("{} Lookups", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            lookup(token_contract.clone()).await.unwrap();
        })
    });
    Ok(())
}

#[allow(unused_imports)]
mod tests {
    use std::time::Duration;

    use super::*;

    use arbiter_core::{environment::builder::EnvironmentBuilder, middleware::RevmMiddleware};
    use ethers::utils::Anvil;
    use ethers::{
        core::k256::ecdsa::SigningKey,
        middleware::SignerMiddleware,
        providers::{Http, Provider},
        signers::{LocalWallet, Signer, Wallet},
    };
    #[tokio::test]
    async fn arbiter_anvil() {
        // get arbiter middleware
        let environment = EnvironmentBuilder::new().build();
        let arbiter_middleware = RevmMiddleware::new(&environment, Some("name")).unwrap();

        // get anvil middlewar
        let anvil = Anvil::new().spawn();

        // Create a client
        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(Duration::ZERO);

        let wallet: LocalWallet = anvil.keys()[0].clone().into();
        let anvil_middleware = Arc::new(SignerMiddleware::new(
            provider,
            wallet.with_chain_id(anvil.chain_id()),
        ));

        let mut c = Criterion::default().configure_from_args();

        let arbiter_results = bench_middleware(&mut c, arbiter_middleware, "Arbiter").await;
        if let Err(err) = &arbiter_results {
            eprintln!("Error with Arbiter middleware: {:?}", err);
        }
        assert!(arbiter_results.is_ok());
        // let arbiter_results = arbiter_results.unwrap();

        let anvil_results = bench_middleware(&mut c, anvil_middleware, "Anvil").await;
        if let Err(err) = &anvil_results {
            eprintln!("Error with Anvil middleware: {:?}", err);
        }
        assert!(anvil_results.is_ok());
        // let anvil_results = anvil_results.unwrap();
    }
}

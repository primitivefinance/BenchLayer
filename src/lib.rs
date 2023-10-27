// Todo allow for users to pick the contracts they want to benchmark with

use anyhow::{Ok, Result};
use bench_functions::create_call;
use ethers::{
    providers::Middleware,
    types::{I256, U256},
};

use std::sync::Arc;

use crate::bench_functions::{lookup, stateful_call, stateless_call};
use criterion::async_executor::FuturesExecutor;
use criterion::Criterion;

mod bench_functions;
mod utils;
mod bindings;

pub async fn bench_middleware<M: Middleware + 'static>(
    c: &mut Criterion,
    client: Arc<M>,
    label: &str,
) -> Result<()> {
    println!("Start bench_middleware with label: {}", label);
    let wad = U256::from(10_u128.pow(18));

    // these are the contracts we are benching against
    // this can happily change if people want to bench against more general contracts
    let (math_contract, token_contract) =
        utils::deploy_contracts_for_benchmarks(client.clone()).await?;

    c.bench_function(&format!("{} Stateful Call", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            stateful_call(token_contract.clone(), client.default_sender().unwrap())
                .await
                .unwrap();
        })
    });
    c.bench_function(&format!("{} Stateless Call", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            stateless_call(math_contract.clone()).await.unwrap();
        })
    });
    c.bench_function(&format!(" {} Create", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            create_call(client.clone()).await.unwrap();
        })
    });
    c.bench_function(&format!("{} Lookups", label), |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            lookup(token_contract.clone()).await.unwrap();
        })
    });
    println!("End bench_middleware with label: {}", label);
    Ok(())
}

#[allow(unused_imports)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use crate::utils::deploy_contracts_for_benchmarks;

    use super::*;

    use arbiter_core::{environment::builder::EnvironmentBuilder, middleware::RevmMiddleware, bindings::arbiter_math};
    use ethers::utils::Anvil;
    use ethers::{
        core::k256::ecdsa::SigningKey,
        middleware::SignerMiddleware,
        providers::{Http, Provider},
        signers::{LocalWallet, Signer, Wallet},
    };
    use serde::de;
    #[tokio::test]
    async fn arbiter_anvil() {
        // get arbiter middleware
        let environment = EnvironmentBuilder::new().build();
        let arbiter_middleware = RevmMiddleware::new(&environment, Some("name")).unwrap();

        // get anvil middleware
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
        drop(anvil);
        // let anvil_results = anvil_results.unwrap();
    }

    #[tokio::test]
    async fn test_deployments_anvil() {
        // get anvil middleware
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

        let math = arbiter_math::ArbiterMath::deploy(anvil_middleware.clone(), ());

        assert!(math.is_ok());
        let math = math.unwrap().send().await;
        assert!(math.is_ok());

        // let result = deploy_contracts_for_benchmarks(anvil_middleware).await;
        // assert!(result.is_ok());
    }
}

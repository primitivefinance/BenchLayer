// Todo allow for users to pick the contracts they want to benchmark with

use anyhow::{Ok, Result};
use bench_functions::create_call;
use ethers::{providers::Middleware, utils::AnvilInstance};

use std::sync::Arc;

use crate::bench_functions::{lookup, stateful_call, stateless_call};
use criterion::async_executor::FuturesExecutor;
use criterion::Criterion;

mod bench_functions;
mod bindings;
mod utils;

pub async fn bench_middleware<M: Middleware + 'static>(
    c: &mut Criterion,
    client: Arc<M>,
    label: &str,
    _anvil: Option<AnvilInstance>,
) -> Result<()> {
    println!("Start bench_middleware with label: {}", label);
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
    if let Some(anvil) = _anvil {
        drop(anvil);
    }
    Ok(())
}

#[allow(unused_imports)]
mod tests {
    use std::time::Duration;
    use std::{str::FromStr, thread};

    use crate::{bindings::counter::Counter, utils::deploy_contracts_for_benchmarks};

    use super::*;

    use arbiter_core::{
        bindings::arbiter_math::ArbiterMath, environment::builder::EnvironmentBuilder,
        middleware::RevmMiddleware,
    };
    use ethers::{
        core::k256::ecdsa::SigningKey,
        middleware::SignerMiddleware,
        providers::{Http, Provider},
        signers::{LocalWallet, Signer, Wallet},
        types::Address,
        utils::{Anvil, AnvilInstance},
    };

    #[tokio::test]
    async fn arbiter() {
        // get arbiter middleware
        let environment = EnvironmentBuilder::new().build();
        let arbiter_middleware = RevmMiddleware::new(&environment, Some("name")).unwrap();

        let mut c = Criterion::default().configure_from_args();

        let arbiter_results = bench_middleware(&mut c, arbiter_middleware, "Arbiter", None).await;
        if let Err(err) = &arbiter_results {
            eprintln!("Error with Arbiter middleware: {:?}", err);
        }
        assert!(arbiter_results.is_ok());
        // let arbiter_results = arbiter_results.unwrap();
        // let anvil_results = anvil_results.unwrap();
    }

    #[tokio::test]
    async fn anvil() {
        let anvil = Anvil::new().spawn();
        // Create a client
        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(Duration::ZERO);

        // check client is working
        let block = provider.get_block_number().await;
        assert!(block.is_ok());
        let block = block.unwrap();
        assert_eq!(block, 0_u64.into());

        let wallet = LocalWallet::from(anvil.keys()[0].clone());
        assert_eq!(
            wallet.address(),
            Address::from_str("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap()
        );
        let anvil_middleware = Arc::new(SignerMiddleware::new(
            provider,
            wallet.with_chain_id(anvil.chain_id()),
        ));

        let mut c = Criterion::default().configure_from_args();
        let anvil_results = bench_middleware(&mut c, anvil_middleware, "Anvil", Some(anvil)).await;
        if let Err(err) = &anvil_results {
            eprintln!("Error with Anvil middleware: {:?}", err);
        }
        assert!(anvil_results.is_ok());
    }

    #[tokio::test]
    async fn debugging_anvil_deployments() {
        // get anvil middleware

        let anvil = Anvil::new().spawn();
        // Create a client
        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(Duration::ZERO);

        // check client is working
        let block = provider.get_block_number().await;
        assert!(block.is_ok());
        let block = block.unwrap();
        assert_eq!(block, 0_u64.into());

        let wallet = LocalWallet::from(anvil.keys()[0].clone());
        assert_eq!(
            wallet.address(),
            Address::from_str("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266").unwrap()
        );
        // let wallet: LocalWallet = anvil.keys()[0].clone().into();
        let anvil_middleware = Arc::new(SignerMiddleware::new(
            provider,
            wallet.with_chain_id(anvil.chain_id()),
        ));

        println!("{:?}#", anvil.port());
        // or the other arbiter bindings
        let math = ArbiterMath::deploy(anvil_middleware.clone(), ())
            .unwrap()
            .send()
            .await
            .unwrap();
        println!("1");
        println!("{:#?}", math.address());
    }
}

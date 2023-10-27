// Todo allow for users to pick the contracts they want to benchmark with
use arbiter_core::{
    bindings::{
        arbiter_math::{self, ArbiterMath},
        arbiter_token::{self, ArbiterToken},
    },
    environment::{builder::EnvironmentBuilder, Environment},
    middleware::RevmMiddleware,
};

use anyhow::{Ok, Result};
use ethers::{
    core::{k256::ecdsa::SigningKey, utils::Anvil},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, I256, U256},
    utils::AnvilInstance,
};

mod bench_functions;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

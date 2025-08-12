use ethers::prelude::*;
use std::{collections::HashSet, env, sync::Arc};
use tokio::sync::Mutex;
use ethers::{abi::Abi, providers::{Http, Provider, Ws}, signers::{LocalWallet, Signer}, contract::Contract, middleware::SignerMiddleware, types::{Transaction, Address, H256, U256}};
use futures_util::stream::StreamExt;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let (pool1, pool2, rpc_url, private_key, tomb_address) = (
        env::var("TARGET_POOL1")?.to_lowercase(),
        env::var("TARGET_POOL2")?.to_lowercase(),
        env::var("RPC_URL")?,
        env::var("PRIVATE_KEY")?,
        env::var("TOMB_CONTRACT")?.parse::<Address>()?,
    );
    let min_value = ethers::utils::parse_ether(env::var("MIN_ETH_VALUE").unwrap_or("5".into()))?;
    let ws = Arc::new(Provider::<Ws>::connect(&rpc_url).await?);
    let http = Arc::new(Provider::<Http>::try_from(&rpc_url)?);
    let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(ws.get_chainid().await?.as_u64());
    let signer = Arc::new(SignerMiddleware::new(ws.clone(), wallet));
    let tomb = Contract::new(tomb_address, tomb_abi(), signer.clone());
    let selectors = vec![
    "38ed1739", // swapExactTokensForTokens
    "7ff36ab5", // swapExactETHForTokens
    "8803dbee", // swapTokensForExactTokens
    "fb0fc03b", // fulfillBasicOrder_efficient_6GL6yc (Seaport)
    "a9059cbb", // ERC20 transfer
    "23b872dd", // ERC20 transferFrom
    ];
    let cache = Arc::new(Mutex::new(HashSet::new()));
    let mut stream = ws.subscribe_pending_txs().await?;

    while let Some(tx_hash) = stream.next().await {
        let (http, tomb, cache, pool1, pool2, selectors) = 
            (http.clone(), tomb.clone(), cache.clone(), pool1.clone(), pool2.clone(), selectors.clone());

        tokio::spawn(async move {
            if let Ok(Some(tx)) = http.get_transaction(tx_hash).await {
                handle_tx(tx, &pool1, &pool2, min_value, &selectors, tomb, cache).await;
            }
        });
    }
    Ok(())
}

async fn handle_tx(
    tx: Transaction,
    pool1: &str,
    pool2: &str,
    min_value: U256,
    selectors: &[&str],
    tomb: Contract<SignerMiddleware<Arc<Provider<Ws>>, LocalWallet>>,
    cache: Arc<Mutex<HashSet<H256>>>,
) {
    if tx.to.is_none() || tx.input.0.is_empty() || tx.value < min_value {
        return;
    }
    let to = format!("{:?}", tx.to.unwrap()).to_lowercase();
    if to != *pool1 && to != *pool2 {
        return;
    }

    let calldata = hex::encode(&tx.input.0);
    if !selectors.iter().any(|sel| calldata.starts_with(sel)) {
        return;
    }

    let signal = format!(
    "{}|{}|{}|{}|{}|{}",
    pool2,
    to,
    ethers::utils::format_ether(tx.value),
    &calldata[..20],
    tx.hash,
    tx.from.unwrap_or_default()
    );

    let sig_hash = H256::from(ethers::utils::keccak256(signal.as_bytes()));
    
    let royalty_bps = if tx.value > U256::exp10(18) { 500u64 } else { 1000u64 };


    let mut lock = cache.lock().await;
    if !lock.insert(sig_hash) {
        return;
    }

    println!("Signal emitted: {} with royalty: {}bps", signal, royalty_bps);

    if let Ok(sent) = tomb.method::<_, ()>("emitCascade", (signal.clone(), royalty_bps))
        .unwrap()
        .gas_price(ethers::utils::parse_units("0.1", "gwei").unwrap())
        .send()
        .await 
    {
        let _ = sent.await;
        if let Ok(claimed) = tomb.method::<_, ()>("claimYield", signal.clone())
            .unwrap()
            .gas_price(ethers::utils::parse_units("0.1", "gwei").unwrap())
            .send()
            .await 
        {
            let _ = claimed.await;
            println!("Yield claimed for signal: {}", signal);
        }
    }
}

fn tomb_abi() -> Abi {
    serde_json::from_str(include_str!("tomb_abi.json")).unwrap()
}

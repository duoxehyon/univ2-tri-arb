use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::providers::Provider;
use std::sync::Arc;

// Main Config
pub struct Config {
    // Http provider
    pub http: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    // Websocket provider
    pub wss: Arc<Provider<Ws>>,
    // pub ipc: Arc<Provider<Ipc>>,
    pub wallet: Arc<Wallet<SigningKey>>,
}

impl Config {
    // Implement a constructor for the configuration struct
    pub async fn new() -> Self {
        let http_url = std::env::var("NETWORK_HTTP").expect("missing NETWORK_RPC");
        let provider: Provider<Http> = Provider::<Http>::try_from(http_url).unwrap();

        let wss_url = std::env::var("NETWORK_WSS").expect("missing NETWORK_WSS");
        let ws_provider: Provider<Ws> = Provider::<Ws>::connect(wss_url).await.unwrap();

        let chain_id = provider.get_chainid().await.unwrap().as_u64();

        let private_key = std::env::var("PRIVATE_KEY").expect("missing PRIVATE_KEY");
        let wallet = private_key
            .parse::<LocalWallet>()
            .expect("invalid PRIVATE_KEY")
            .with_chain_id(chain_id);

        let middleware = Arc::new(SignerMiddleware::new(provider, wallet.clone()));
        Self {
            http: middleware,
            wss: Arc::new(ws_provider),
            wallet: Arc::new(wallet),
        }
    }
}

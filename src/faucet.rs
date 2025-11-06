use bitreq::Response;
use elements::Address;
use tracing::info;

use crate::error::Error;

/// Blockstream's L-BTC TestnetV1 faucet.
pub(crate) const FAUCET_URL: &str =
    "https://liquidtestnet.com/faucet?address=<PLACEHOLDER>&action=lbtc";

/// Request L-BTC TestnetV1 coins to an address.
pub(crate) async fn get_coins(address: &Address) -> Result<Response, Error> {
    let url: String = FAUCET_URL.replace("<PLACEHOLDER>", &address.to_string());

    info!("Requesting L-BTC coin from faucet: {}", url);
    let response: Response = bitreq::get(&url).send_async().await?;
    info!("Received response code {}", response.status_code);

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_coins() {
        tracing_subscriber::fmt().init();

        let address: Address = "ex1qxdrsg0z86ca6q848jyczql8c5zkuax9n90kk6t"
            .parse::<Address>()
            .unwrap();

        let _ = get_coins(&address).await.unwrap();
    }
}

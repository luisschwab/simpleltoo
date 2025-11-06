use bitreq::Response;
use elements::Address;
use tracing::info;

use crate::error::Error;

/// Blockstream's L-BTC TestnetV1 faucet.
pub(crate) const FAUCET_URL: &str =
    "https://liquidtestnet.com/faucet?address=<PLACEHOLDER>&action=lbtc";

/// Request L-BTC TestnetV1 coins to an address.
pub(crate) async fn get_coins(address: &Address) -> Result<(), Error> {
    let url: String = FAUCET_URL.replace("<PLACEHOLDER>", &address.to_string());

    info!("Requesting L-BTC coin from faucet `https://liquidtestnet.com/faucet`...");
    let response: Response = bitreq::get(&url).send_async().await?;
    info!(
        "Received respose from faucet `https://liquidtestnet.com/faucet`: {:?}",
        response.as_str()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_coins() {
        let address: Address = "ex1qxdrsg0z86ca6q848jyczql8c5zkuax9n90kk6t"
            .parse::<Address>()
            .unwrap();

        let _ = get_coins(&address).await.unwrap();
    }
}

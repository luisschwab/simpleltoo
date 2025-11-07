#![allow(unused)]

use std::str::FromStr;

use bitreq::Response;
use elements::{Address, Txid};
use regex::Regex;
use tracing::info;

use crate::error::Error;

/// Blockstream's L-BTC TestnetV1 faucet.
pub(crate) const FAUCET_URL: &str =
    "https://liquidtestnet.com/faucet?address=<PLACEHOLDER>&action=lbtc";

/// Request L-BTC TestnetV1 coins to an [`Address`].
pub(crate) async fn get_testnet_coins(address: &Address) -> Result<Txid, Error> {
    let url: String = FAUCET_URL.replace("<PLACEHOLDER>", &address.to_string());

    info!("Requesting L-BTC coin from faucet: {}", url);
    let response: Response = bitreq::get(&url).send_async().await?;
    info!("Received response code {}", response.status_code);

    let txid = extract_txid_from_shit_response(response.as_str()?)?;

    Ok(txid)
}

pub(crate) fn extract_txid_from_shit_response(html: &str) -> Result<Txid, Error> {
    // Pattern to match "with transaction" followed by a 64-character hex string.
    let re = Regex::new(r"with transaction ([0-9a-fA-F]{64})\b").unwrap();

    if let Some(captures) = re.captures(html) {
        let txid_str = captures[1].to_string();
        let txid = Txid::from_str(&txid_str)?;

        return Ok(txid);
    }

    Err(Error::HtmlParsing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_coins() {
        tracing_subscriber::fmt().init();

        let address: Address = "tex1p53ct8hcvnr7zznfjawxwetycthxyv6c06vh4dk2zymc3c3laps5q94kptw"
            .parse::<Address>()
            .unwrap();

        let txid = get_testnet_coins(&address).await.unwrap();

        info!("Funding Txid: {}", txid);
    }
}

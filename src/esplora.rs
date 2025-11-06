use std::collections::HashMap;

use elements::bitcoin::Amount;
use elements::{Address, Transaction, Txid};
use lwk_wollet::{
    ElementsNetwork,
    clients::asyncr::{EsploraClient as AsyncClient, EsploraClientBuilder as Builder},
};

pub(crate) const LIQUIDV1_MAIN_URL: &str = "https://liquid.network/api/";
pub(crate) const LIQUIDV1_TEST_URL: &str = "https://liquid.network/liquidtestnet/api/";

use crate::error::Error;

/// Create a new Esplora [`AsyncClient`].
pub(crate) fn create_client(url: &str, network: ElementsNetwork) -> Result<AsyncClient, Error> {
    Ok(AsyncClient::new(network, url))
}

/// Broadcast a [`Transaction`] through Esplora.
pub(crate) async fn broadcast_transaction(
    client: &AsyncClient,
    transaction: &Transaction,
) -> Result<Txid, Error> {
    let txid = client.broadcast(transaction).await?;
    Ok(txid)
}

/// Get [`Transaction`]s by [`Txid`]s.
pub(crate) async fn get_transactions(
    client: &AsyncClient,
    txids: &[Txid],
) -> Result<Vec<Transaction>, Error> {
    let transactions = client.get_transactions(txids).await?;
    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use elements::bitcoin::Network;

    use super::*;

    static LIQUID_V1_ADDR: LazyLock<Address> = LazyLock::new(|| {
        "Go65t19hP2FuhBMYtgbdMDgdmEzNwh1i48"
            .parse::<Address>()
            .unwrap()
    });

    // TODO(@luisschwab): add tests
}

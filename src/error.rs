#![allow(unused)]

use thiserror::Error;

/// Unified error variants.
#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Bitreq error: {0}")]
    Bitreq(#[from] bitreq::Error),

    #[error("Esplora error: {0}")]
    Esplora(#[from] lwk_wollet::Error),

    #[error("Simplicity error: {0}")]
    Simplicity(#[from] simplicityhl::error::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Txid parsing from faucet's HTML error")]
    HtmlParsing,

    #[error("Hex parsing error: {0}")]
    HexParsing(#[from] elements::bitcoin::hex::HexToArrayError),
}

use thiserror::Error;

/// Unified error variants.
#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Bitreq error: {0}")]
    Bitreq(#[from] bitreq::Error),

    #[error("Esplora error: {0}")]
    Esplora(#[from] lwk_wollet::Error),
}

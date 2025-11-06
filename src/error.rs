use thiserror::Error;

/// Unified error variants.
#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Esplora error: {0}")]
    Esplora(#[from] lwk_wollet::Error),
}

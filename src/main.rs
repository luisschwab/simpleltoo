// TODO(@luisschwab): remove this
#![allow(unused)]

use tracing_subscriber::EnvFilter;

mod compile;
mod error;
mod esplora;
mod faucet;
mod sign;
mod transaction;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

use std::net::Ipv4Addr;

use clap::Parser;
use std::path::PathBuf;

use crate::load_generator::Targets;
use crate::metrics::Metrics;

mod conn;
mod conn_initializer;
mod discord;
mod limiter;
mod load_generator;
mod metrics;
mod reporter;
mod request;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(long, env, value_delimiter = ',', default_value = "0.0.0.0")]
    sender_ips: Vec<Ipv4Addr>,

    #[clap(long, env, value_delimiter = ',', default_value = "0.0.0.0")]
    retry_ips: Vec<Ipv4Addr>,

    #[clap(long, env, default_value_t = 1)]
    multiplier: u8,

    #[clap(long, env, default_value_t = 1)]
    rty_multiplier: u8,

    #[clap(long, env)]
    report_in: url::Url,

    #[clap(long, env, default_value = "600s")]
    report_interval: humantime::Duration,

    #[clap(long, env, default_value = "Hello World!")]
    message: String,

    #[clap(long, env)]
    targets: PathBuf,

    #[clap(long, env, default_value = "100ms")]
    send_interval: humantime::Duration,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let metrics = Metrics::new();

    let format = tracing_subscriber::fmt::format()
        .with_target(false)
        .compact();

    tracing_subscriber::fmt()
        .event_format(format)
        .with_max_level(tracing::Level::INFO)
        .init();

    let targets = Targets::try_new(&cli.targets).unwrap();

    tokio::spawn({
        let metrics = metrics.clone();
        async move { reporter::run(&cli.report_interval, &cli.report_in, metrics).await }
    });

    let (sender, _limiter) = conn_initializer::initialize(
        &cli.retry_ips,
        &cli.sender_ips,
        cli.multiplier,
        cli.rty_multiplier,
        metrics,
    )
    .await
    .expect("failed to initialize connection");

    load_generator::run(targets, sender, &cli.send_interval, &cli.message).await;
}

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result};
use bytes::Bytes;
use serde_json::json;

use crate::request::{Context, JobSender, Request};

#[derive(Debug, Clone)]
pub struct Targets {
    targets: Vec<url::Url>,
}

impl Targets {
    pub fn try_new(path: &Path) -> Result<Self> {
        let file = File::open(path)?;

        let targets: Result<Vec<url::Url>> = BufReader::new(file)
            .lines()
            .map(|line| {
                line.context("Failed to read line")
                    .and_then(|line| line.parse().context("Failed to parse as URL"))
            })
            .map(|v| v)
            .collect();

        let targets = targets?;

        Ok(Self { targets })
    }
}

pub async fn run(targets: Targets, sender: JobSender, interval: &Duration, message: &str) {
    let mut interval = tokio::time::interval(*interval);

    let body = Bytes::from(
        json!({
            "content": message,
        })
        .to_string()
        .into_bytes(),
    );

    let context = Arc::new(Context {
        body,
        retry_limit: 0,
    });

    loop {
        tracing::info!("Iteration Start!");
        for target in &targets.targets {
            let _ = interval.tick().await;

            sender
                .send(Request {
                    context: context.clone(),
                    retry_count: 0,
                    target: target.clone(),
                    identity: format!("benchmark#benchmark#benchmark"),
                })
                .await
                .unwrap();
        }
    }
}

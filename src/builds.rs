use anyhow::Context;
use concat_in_place::strcat;
use http::StatusCode;
use isahc::prelude::*;
use serde::Deserialize;
use std::{future::Future, time::Duration};
use thiserror::Error;

const BASE: &str = "https://api.pop-os.org/";

#[derive(Debug, Error)]
pub enum BuildsError {
    #[error("build ({}) is not a number", _0)]
    BuildNaN(Box<str>),
    #[error("failed to GET release API")]
    Get(#[source] anyhow::Error),
    #[error("failed to parse JSON response")]
    Json(#[source] serde_json::Error),
    #[error("server responded with an error status: {}", _0)]
    Status(StatusCode),
}

#[derive(Debug)]
pub struct Build {
    pub channel: Box<str>,
    pub sha_sum: Box<str>,
    pub url: Box<str>,
    pub version: Box<str>,
    pub build: u16,
    pub size: u64,
}

impl Build {
    /// Fetches the current build from api.pop-os.org for the given release
    ///
    /// # Implementation Notes
    ///
    /// Convenience method which calls `Build::get_with()` with a freshly-allocated
    /// string buffer, and a freshly-created `Isahc` client.
    pub async fn get(version: &str, channel: &str) -> Result<Build, BuildsError> {
        Self::get_with(&mut String::new(), version, channel, |url| {
            async move {
                Request::get(url)
                    .connect_timeout(Duration::from_secs(5))
                    .body(())
                    .unwrap()
                    .send_async()
                    .await
                    .context("GET failed")?
                    .text_async()
                    .await
                    .context("GET body fetch failed")
            }
        })
        .await
    }

    /// Fetches the current build from api.pop-os.org for the given release
    pub async fn get_with<'a, F, O>(
        buffer: &'a mut String,
        version: &str,
        channel: &str,
        client: F,
    ) -> Result<Build, BuildsError>
    where
        F: Fn(&'a str) -> O + 'a,
        O: Future<Output = Result<String, anyhow::Error>>,
    {
        let url = Self::url(buffer, version, channel);

        let text = client(url).await.map_err(BuildsError::Get)?;

        serde_json::from_str::<BuildRaw>(&text)
            .map_err(BuildsError::Json)?
            .into_release()
    }

    /// Fetches the latest build ID for this release, if it exists.
    pub async fn build_exists(version: &str, channel: &str) -> Result<u16, BuildsError> {
        Self::get(version, channel).await.map(|r| r.build)
    }

    /// Stores the generated URL for this request in the `out` string buffer
    pub fn url<'a>(out: &'a mut String, version: &str, channel: &str) -> &'a str {
        strcat!(out, BASE "builds/" version "/" channel)
    }
}

#[derive(Debug, Deserialize)]
struct BuildRaw {
    pub build: String,
    pub channel: String,
    pub sha_sum: String,
    pub url: String,
    pub version: String,
    pub size: u64,
}

impl BuildRaw {
    fn into_release(self) -> Result<Build, BuildsError> {
        let BuildRaw {
            version,
            url,
            size,
            sha_sum,
            channel,
            build,
        } = self;
        let build = build
            .parse::<u16>()
            .map_err(|_| BuildsError::BuildNaN(build.into()))?;

        Ok(Build {
            channel: channel.into(),
            sha_sum: sha_sum.into(),
            url: url.into(),
            version: version.into(),
            build,
            size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor;

    #[test]
    pub fn release_exists() {
        assert!(executor::block_on(Build::get("18.04", "intel")).is_ok());
    }
}

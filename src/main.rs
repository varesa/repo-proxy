use crate::config::Config;

mod ca;
mod config;
mod metalink;
mod proxy;
mod prefix_match;
mod request_meta;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::try_from_args()?;
    let ca = ca::Ca::get_or_create(&config.paths.data)?;
    let prefix_matcher = prefix_match::PrefixMatcher::try_new(config.views)?;
    let proxy = proxy::Proxy::try_new(ca, prefix_matcher)?;
    proxy.start().await?;
    Ok(())
}

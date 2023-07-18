use crate::config::Config;

mod ca;
mod config;
mod proxy;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::try_from_args()?;
    let ca = ca::Ca::get_or_create(&config.paths.data)?;
    let proxy = proxy::Proxy::try_new(ca)?;
    proxy.start().await?;
    Ok(())
}

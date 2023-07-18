use crate::config::Config;

mod ca;
mod config;

fn main() -> Result<(), anyhow::Error> {
    let config = Config::try_from_args()?;
    let _ca = ca::Ca::get_or_create(&config.paths.data)?;
    Ok(())
}

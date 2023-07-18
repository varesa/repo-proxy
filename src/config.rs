use std::{fs::read_to_string};
use std::path::PathBuf;
use ipnet::IpNet;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Paths {
    pub data: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct View {
    pub name: String,
    pub source: Vec<IpNet>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub paths: Paths,
    pub views: Vec<View>,
}

impl Config {
    pub fn try_from_file(path: PathBuf) -> Result<Self, anyhow::Error> {
        let conf_string = read_to_string(path)?;
        let config = toml::from_str(&conf_string)?;
        Ok(config)
    }

    pub fn try_from_args() -> Result<Self, anyhow::Error> {
        let args: Vec<String> = std::env::args().collect();
        let config_path = match args.get(1) {
            Some(path) => path,
            None => return Err(anyhow::Error::msg("Missing config file as parameter")),
        };
        let config = Config::try_from_file(PathBuf::from(config_path))?;
        println!("{config:#?}");
        Ok(config)
    }
}


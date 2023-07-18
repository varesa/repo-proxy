use std::net::IpAddr;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use prefix_trie::PrefixMap;
use crate::config::View;

pub struct PrefixMatcher {
    prefixes4: PrefixMap<Ipv4Net, String>,
    prefixes6: PrefixMap<Ipv6Net, String>,
}

impl PrefixMatcher {
    pub fn try_new(views: Vec<View>) -> Result<Self, anyhow::Error> {
        let mut matcher = Self {
            prefixes4: PrefixMap::new(),
            prefixes6: PrefixMap::new(),
        };
        matcher.create_mapping(views);
        Ok(matcher)
    }

    fn create_mapping(&mut self, views: Vec<View>) {
        for view in views {
            for source in view.source {
                match source {
                    IpNet::V4(prefix) => self.prefixes4.insert(prefix, view.name.clone()),
                    IpNet::V6(prefix) => self.prefixes6.insert(prefix, view.name.clone()),
                };
            }
        }
    }

    pub fn lookup(&self, address: IpAddr) -> Result<String, anyhow::Error> {
        let view = match address {
            IpAddr::V4(addr) => self.prefixes4.get_lpm(&Ipv4Net::new(addr, 32)?).map(|view| view.1.clone()),
            IpAddr::V6(addr) => self.prefixes6.get_lpm(&Ipv6Net::new(addr, 128)?).map(|view| view.1.clone()),
        };
        if let Some(name) = view {
            Ok(name)
        }  else {
            Err(anyhow::Error::msg(format!("No view matching {address}")))
        }
    }
}
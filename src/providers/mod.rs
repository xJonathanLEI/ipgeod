mod herrbischoff;
use std::net::Ipv4Addr;

pub use herrbischoff::HerrbischoffProvider;

mod ip2location;
pub use ip2location::Ip2locationProvider;

#[derive(Debug)]
pub enum IpgeoProvider {
    Herrbischoff(HerrbischoffProvider),
    Ip2location(Ip2locationProvider),
}

impl IpgeoProvider {
    pub fn get_ipv4_country(&self, ip_address: &Ipv4Addr) -> Option<String> {
        match self {
            Self::Herrbischoff(provider) => provider.get_ipv4_country(ip_address),
            Self::Ip2location(provider) => provider.get_ipv4_country(ip_address),
        }
    }
}

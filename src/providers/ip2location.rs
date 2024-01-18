use std::{
    io::{BufRead, BufReader},
    net::Ipv4Addr,
};

#[derive(Debug)]
pub struct Ip2locationProvider {
    ip_ranges: Vec<IpRange>,
}

#[derive(Debug)]
struct IpRange {
    start: u32,
    end: u32,
    country: String,
}

impl Ip2locationProvider {
    pub fn from_db(db_path: &std::path::Path) -> anyhow::Result<Self> {
        let mut ranges: Vec<IpRange> = vec![];

        let mut file = std::fs::File::open(db_path)?;
        let reader = BufReader::new(&mut file);

        // TODO: use a proper CSV reader
        for line in reader.lines() {
            let line = line?;

            let cols = line
                .split("\",\"")
                .map(|col| col.trim_matches('"'))
                .collect::<Vec<_>>();

            if cols.len() < 3 {
                anyhow::bail!("invalid row");
            }

            let start: u32 = cols[0].parse()?;
            let end: u32 = cols[1].parse()?;
            let country_code = cols[2];

            if country_code != "-" {
                if country_code.len() != 2 {
                    anyhow::bail!("invalid country code: {}", country_code);
                }

                // Makes sure that the list is sorted
                if !ranges.is_empty() {
                    let last_element = &ranges[ranges.len() - 1];

                    if last_element.end >= start {
                        anyhow::bail!("list not sorted");
                    }
                }

                ranges.push(IpRange {
                    start,
                    end,
                    country: country_code.to_uppercase(),
                });
            }
        }

        Ok(Self { ip_ranges: ranges })
    }

    pub fn get_ipv4_country(&self, ip_address: &Ipv4Addr) -> Option<String> {
        let ip_value = u32::from_be_bytes(ip_address.octets());

        match self
            .ip_ranges
            .binary_search_by_key(&ip_value, |item| item.start)
        {
            Ok(ind) => {
                // `start` matches perfectly with `ip_value`
                let range = &self.ip_ranges[ind];
                Some(range.country.to_owned())
            }
            Err(ind) => {
                if ind > 0 {
                    // No exact `start` matches. This is the closest range
                    let range = &self.ip_ranges[ind - 1];

                    if range.end >= ip_value {
                        // The closest range includes `ip_value`
                        Some(range.country.to_owned())
                    } else {
                        // `ip_value` falls in the gap between two ranges
                        None
                    }
                } else {
                    // `ip_value` is smaller even than the first record
                    None
                }
            }
        }
    }
}

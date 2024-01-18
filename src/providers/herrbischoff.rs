use std::{
    io::{BufRead, BufReader},
    net::Ipv4Addr,
};

use cidr::Ipv4Cidr;

#[derive(Debug)]
pub struct HerrbischoffProvider {
    cidr_blocks: Vec<CidrBlock>,
}

#[derive(Debug)]
struct CidrBlock {
    cidr: Ipv4Cidr,
    country: String,
}

impl HerrbischoffProvider {
    pub fn from_repo(repo_path: &std::path::Path) -> anyhow::Result<Self> {
        let mut cidr_blocks = vec![];

        for entry in std::fs::read_dir(repo_path.join("ipv4"))? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().is_some_and(|value| value == "cidr") {
                let country_code = file_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("unable to read file name"))?
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("invalid file name"))?
                    .split_once('.')
                    .expect("already checked that extension exists")
                    .0
                    .to_uppercase();

                if country_code.len() != 2 {
                    anyhow::bail!("invalid country code: {}", country_code);
                }

                let mut file = std::fs::File::open(&file_path)?;
                let reader = BufReader::new(&mut file);
                for line in reader.lines() {
                    let line = line?;

                    let cidr: Ipv4Cidr = line.parse()?;

                    cidr_blocks.push(CidrBlock {
                        cidr,
                        country: country_code.clone(),
                    })
                }
            }
        }

        Ok(Self { cidr_blocks })
    }

    // This implementation is extremely inefficient, with O(n) for each lookup. This can be
    // optimized with a sorted list of CIDR blocks, and use binary search to reduce the steps to
    // O(log n). Though slow and inefficient, it's good enough for an MVP.
    //
    // TODO: optimize with sorted CIDR blocks and binary search.
    pub fn get_ipv4_country(&self, ip_address: &Ipv4Addr) -> Option<String> {
        for block in self.cidr_blocks.iter() {
            if block.cidr.contains(ip_address) {
                return Some(block.country.clone());
            }
        }

        None
    }
}

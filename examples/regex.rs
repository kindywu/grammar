use std::net::Ipv4Addr;

use anyhow::Result;
use chrono::DateTime;
use http::Method;
use regex::Regex;

fn main() -> Result<()> {
    let s = r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;

    let re = Regex::new(
        r#"^(?<ip>\S+)\s\S\s\S\s\[(?<date>[\s\S]*)\]\s\"(?<method>\S+)\s(?<url>\S+)\s(?<proto>\S+)\"\s(?<status>\d+)\s(?<bytes>\d+)\s\"(?<refer>\S+)\"\s\"(?<ua>[\S\s]+)\""#,
    )?;

    let captures = re.captures(s).unwrap();
    let ip = captures.name("ip").unwrap().as_str().parse::<Ipv4Addr>()?;
    println!("{ip}");
    let format = "%d/%b/%Y:%H:%M:%S %z";
    let date = DateTime::parse_from_str(captures.name("date").unwrap().as_str(), &format)?;
    println!("{date}");

    let method = Method::from_bytes(captures.name("method").unwrap().as_str().as_bytes())?;
    println!("{method}");
    let url = captures.name("url").unwrap().as_str();
    println!("{url}");
    let proto = captures.name("proto").unwrap().as_str();
    println!("{proto}");
    let status = captures.name("status").unwrap().as_str();
    println!("{status}");
    let bytes = captures.name("bytes").unwrap().as_str();
    println!("{bytes}");
    let refer = captures.name("refer").unwrap().as_str();
    println!("{refer}");
    let ua = captures.name("ua").unwrap().as_str();
    println!("{ua}");
    Ok(())
}

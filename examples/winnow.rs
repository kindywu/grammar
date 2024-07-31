use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use http::Method;
use winnow::{
    ascii::{digit1, space0, space1},
    combinator::{alt, delimited, separated},
    token::{take_till, take_until},
    PResult, Parser,
};

fn main() -> Result<()> {
    let s = r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
    let log = parse_nginx_log(s).map_err(|e| anyhow!("{}", e))?;
    println!("{:?}", log);
    Ok(())
}

fn parse_nginx_log(mut s: &str) -> PResult<NginxLog> {
    let input = &mut s;
    let ip = parse_ip(input)?;
    println!("{ip}");
    space1(input)?;
    "- ".parse_next(input)?;
    "- ".parse_next(input)?;

    let datetime = parse_datetime(input)?;
    println!("{datetime}");
    space1(input)?;

    let (method, url, proto) = parse_http(input)?;
    println!("{method} {url} {proto:?}");

    space0(input)?;

    let status = parse_status(input)?;
    println!("{status}");

    space0(input)?;

    let body_bytes = parse_body_bytes(input)?;
    println!("{body_bytes}");

    space0(input)?;

    let referer = parse_referer(input)?;
    println!("{referer}");

    space0(input)?;

    let user_agent = parse_user_agent(input)?;
    println!("{user_agent}");

    space0(input)?;

    println!("remain:{input}");
    Ok(NginxLog {
        ip,
        datetime,
        method,
        url,
        protocol: proto,
        status,
        body_bytes,
        referer,
        user_agent,
    })
}

fn parse_ip(input: &mut &str) -> PResult<IpAddr> {
    let arr: Vec<u8> = separated(4, digit1.parse_to::<u8>(), '.').parse_next(input)?;
    Ok(IpAddr::V4(Ipv4Addr::new(arr[0], arr[1], arr[2], arr[3])))
}

fn parse_datetime(input: &mut &str) -> PResult<DateTime<Utc>> {
    let str = delimited('[', take_until(1.., ']'), ']').parse_next(input)?;
    let datetime = DateTime::parse_from_str(str, "%d/%b/%Y:%H:%M:%S %z").unwrap();
    Ok(datetime.with_timezone(&Utc))
}

fn parse_http(input: &mut &str) -> PResult<(Method, String, HttpProto)> {
    let parsers = (parse_method, space1, parse_url, space1, parse_proto);
    let ret = delimited('"', parsers, '"').parse_next(input)?;
    Ok((ret.0, ret.2, ret.4))
}

fn parse_method(input: &mut &str) -> PResult<Method> {
    let str = take_till(0.., |c| c == ' ').parse_next(input)?;
    Ok(Method::from_bytes(str.as_bytes()).unwrap())
}

fn parse_url(input: &mut &str) -> PResult<String> {
    let str = take_till(0.., |c| c == ' ').parse_next(input)?;
    Ok(str.to_string())
}

fn parse_proto(input: &mut &str) -> PResult<HttpProto> {
    let ret = alt(("HTTP/1.0", "HTTP/1.1", "HTTP/2.0", "HTTP/3.0"))
        .parse_to()
        .parse_next(input)?;
    Ok(ret)
}

fn parse_status(input: &mut &str) -> PResult<u16> {
    // let ret = take_while(3, |c: char| c.is_ascii_digit())
    //     .parse_to()
    //     .parse_next(input)?;
    let ret = digit1.parse_to().parse_next(input)?;
    Ok(ret)
}

fn parse_body_bytes(input: &mut &str) -> PResult<u64> {
    // let ret = take_while(1.., |c: char| c.is_ascii_digit())
    //     .parse_to()
    //     .parse_next(input)?;
    let ret = digit1.parse_to().parse_next(input)?;
    Ok(ret)
}

fn parse_referer(input: &mut &str) -> PResult<String> {
    let ret = delimited('"', take_until(1.., '"'), '"')
        .parse_to()
        .parse_next(input)?;
    Ok(ret)
}

fn parse_user_agent(input: &mut &str) -> PResult<String> {
    let ret = delimited('"', take_until(1.., '"'), '"')
        .parse_to()
        .parse_next(input)?;
    Ok(ret)
}

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    ip: IpAddr,
    datetime: DateTime<Utc>,
    method: Method,
    url: String,
    protocol: HttpProto,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

#[derive(Debug, PartialEq, Eq)]
enum HttpProto {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

impl FromStr for HttpProto {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "HTTP/1.0" => Ok(HttpProto::HTTP1_0),
            "HTTP/1.1" => Ok(HttpProto::HTTP1_1),
            "HTTP/2.0" => Ok(HttpProto::HTTP2_0),
            "HTTP/3.0" => Ok(HttpProto::HTTP3_0),
            _ => Err(anyhow::anyhow!("Invalid HTTP protocol")),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use http::Method;

    use super::{parse_http, HttpProto};

    #[test]
    fn parse_http_should_work() -> Result<()> {
        let mut s = "\"GET /downloads/product_1 HTTP/1.1\"";
        let (method, url, protocol) = parse_http(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(method, Method::GET);
        assert_eq!(url, "/downloads/product_1");
        assert_eq!(protocol, HttpProto::HTTP1_1);
        Ok(())
    }
}

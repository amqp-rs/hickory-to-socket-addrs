use hickory_to_socket_addrs::HickoryToSocketAddrs;
use std::{net::ToSocketAddrs, str::FromStr};

#[test]
fn resolve() {
    for addr in HickoryToSocketAddrs::from_str("www.rust-lang.org:443")
        .unwrap()
        .to_socket_addrs()
        .unwrap()
    {
        println!("{addr:?}");
    }
}

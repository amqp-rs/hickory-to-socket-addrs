<div align="center">

[![API Docs](https://docs.rs/hickory-to-socket-addrs/badge.svg)](https://docs.rs/hickory-to-socket-addrs)
[![Build status](https://github.com/amqp-rs/hickory-to-socket-addrs/workflows/Build%20and%20test/badge.svg)](https://github.com/amqp-rs/hickory-to-socket-addrs/actions)
[![Downloads](https://img.shields.io/crates/d/hickory-to-socket-addrs.svg)](https://crates.io/crates/hickory-to-socket-addrs)
[![Dependency Status](https://deps.rs/repo/github/amqp-rs/hickory-to-socket-addrs/status.svg)](https://deps.rs/repo/github/amqp-rs/hickory-to-socket-addrs)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

 <strong>
   hickory-to-socket-addrs: std::net::ToSocketAddrs on top of hickory-dns
 </strong>

</div>

<br />

The entry point is the `HickoryToSocketAddrs` struct, which wraps the host and port and use
`hickory-dns` under the hood to perform DNS resolution instead of glibc's `getaddrinfo` which
can block or has a lot of other known issues.

If this is run in a `tokio` context, we use it, otherwise we spawn a new `tokio` runtime to
perform the query.

## Example

```rust
use hickory_to_socket_addrs::HickoryToSocketAddrs;
use std::net::ToSocketAddrs;

let socket_addrs = "www.rust-lang.org:443"
    .parse::<HickoryToSocketAddrs<_>>()?
    .to_socket_addrs()?
    .collect::<Vec<_>>();

Ok::<(), std::io::Error>(())
```

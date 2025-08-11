#![deny(missing_docs, missing_debug_implementations)]

//! hickory-to-socket-addrs
//!
//! `std::net::ToSocketAddrs` on top of `hickory-dns`
//!
//! The entry point is the [`HickoryToSocketAddrs`] struct, which wraps the host and port and use
//! `hickory-dns` under the hood to perform DNS resolution instead of glibc's `getaddrinfo` which
//! can block or has a lot of other known issues.
//!
//! If this is run in a `tokio` context, we use it, otherwise we spawn a new `tokio` runtime to
//! perform the query.
//!
//! ## Example
//!
//! ```rust
//! use hickory_to_socket_addrs::HickoryToSocketAddrs;
//! use std::net::ToSocketAddrs;
//!
//! let socket_addrs = "www.rust-lang.org:443"
//!     .parse::<HickoryToSocketAddrs<_>>()?
//!     .to_socket_addrs()?
//!     .collect::<Vec<_>>();
//! # Ok::<(), std::io::Error>(())
//! ```

use hickory_resolver::{IntoName, Resolver, lookup_ip::LookupIpIntoIter};
use std::{
    fmt,
    future::Future,
    io,
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
};

/// Wrapper around host and port to resolve to `SocketAddr` through `hickory-dns`
///
/// ```rust
/// use hickory_to_socket_addrs::HickoryToSocketAddrs;
/// use std::net::ToSocketAddrs;
///
/// let socket_addrs = "www.rust-lang.org:443"
///     .parse::<HickoryToSocketAddrs<_>>()?
///     .to_socket_addrs()?
///     .collect::<Vec<_>>();
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug)]
pub struct HickoryToSocketAddrs<T: IntoName + Clone> {
    host: T,
    port: u16,
}

impl<T: IntoName + Clone> HickoryToSocketAddrs<T> {
    /// Create a `HickoryToSocketAddrs` from split host and port components.
    pub fn new(host: T, port: u16) -> Self {
        Self { host, port }
    }
}

async fn lookup<H: IntoName>(host: H) -> io::Result<LookupIpIntoIter> {
    Ok(Resolver::builder_tokio()?
        .build()
        .lookup_ip(host)
        .await?
        .into_iter())
}

impl FromStr for HickoryToSocketAddrs<String> {
    type Err = io::Error;

    fn from_str(s: &str) -> io::Result<Self> {
        let (host, port_str) = s
            .rsplit_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid socket address"))?;
        let port = port_str
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid port value"))?;
        Ok(Self::new(host.to_owned(), port))
    }
}

impl<T: IntoName + Clone> ToSocketAddrs for HickoryToSocketAddrs<T> {
    type Iter = HickorySocketAddrs;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        Ok(HickorySocketAddrs(
            block_on(lookup(self.host.clone()))?,
            self.port,
        ))
    }
}

/// Iterator for SocketAddr resolved by `hickory-dns`
pub struct HickorySocketAddrs(LookupIpIntoIter, u16);

impl Iterator for HickorySocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        Some(SocketAddr::new(self.0.next()?, self.1))
    }
}

impl fmt::Debug for HickorySocketAddrs {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("HickorySocketAddrs").finish()
    }
}

fn block_on<T>(fut: impl Future<Output = io::Result<T>>) -> io::Result<T> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(fut)
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(fut)
    }
}

#[cfg(feature = "reactor-trait")]
mod async_impl {
    use super::*;

    use async_trait::async_trait;
    use reactor_trait::AsyncToSocketAddrs;

    #[async_trait]
    impl<T: IntoName + Clone + Send + Sync> AsyncToSocketAddrs for HickoryToSocketAddrs<T> {
        async fn to_socket_addrs(&self) -> io::Result<Box<dyn Iterator<Item = SocketAddr>>> {
            Ok(Box::new(HickorySocketAddrs(
                lookup(self.host.clone()).await?,
                self.port,
            )))
        }
    }
}

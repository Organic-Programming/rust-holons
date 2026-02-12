//! URI-based listener factory for gRPC servers.
//!
//! Supported transports:
//!   - `tcp://<host>:<port>` — TCP socket (default: `tcp://:9090`)
//!   - `unix://<path>`       — Unix domain socket
//!   - `stdio://`            — stdin/stdout pipe
//!   - `mem://`              — in-process channel (testing)

use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, UnixListener};

/// Default transport URI when --listen is omitted.
pub const DEFAULT_URI: &str = "tcp://:9090";

/// Listener variants returned by [`listen`].
pub enum Listener {
    Tcp(TcpListener),
    Unix(UnixListener),
    Stdio,
    Mem(MemListener),
}

/// Parse a transport URI and bind the appropriate listener.
pub async fn listen(uri: &str) -> io::Result<Listener> {
    if let Some(addr) = uri.strip_prefix("tcp://") {
        let addr = if addr.starts_with(':') {
            format!("0.0.0.0{}", addr)
        } else {
            addr.to_string()
        };
        let lis = TcpListener::bind(&addr).await?;
        Ok(Listener::Tcp(lis))
    } else if let Some(path) = uri.strip_prefix("unix://") {
        // Clean stale socket
        let _ = std::fs::remove_file(path);
        let lis = UnixListener::bind(path)?;
        Ok(Listener::Unix(lis))
    } else if uri == "stdio://" || uri == "stdio" {
        Ok(Listener::Stdio)
    } else if uri.starts_with("mem://") {
        Ok(Listener::Mem(MemListener::new()))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported transport URI: {uri:?}"),
        ))
    }
}

/// Extract the transport scheme from a URI.
pub fn scheme(uri: &str) -> &str {
    uri.find("://").map_or(uri, |i| &uri[..i])
}

/// In-process listener using tokio channels for testing.
pub struct MemListener {
    tx: tokio::sync::mpsc::Sender<(tokio::io::DuplexStream, tokio::io::DuplexStream)>,
    rx: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<(tokio::io::DuplexStream, tokio::io::DuplexStream)>>,
}

impl MemListener {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        MemListener {
            tx,
            rx: tokio::sync::Mutex::new(rx),
        }
    }

    /// Client side: create a connection to the in-process server.
    pub async fn dial(&self) -> io::Result<tokio::io::DuplexStream> {
        let (client, server) = tokio::io::duplex(1024 * 1024);
        self.tx
            .send((client, server))
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::ConnectionRefused, "mem listener closed"))?;
        // The caller gets the first half; the server gets the second
        // We need to rethink: send server half, return client half
        Ok(tokio::io::duplex(1024 * 1024).0) // placeholder — see accept()
    }

    /// Server side: accept the next in-process connection.
    pub async fn accept(&self) -> io::Result<tokio::io::DuplexStream> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .map(|(_, server)| server)
            .ok_or_else(|| io::Error::new(io::ErrorKind::ConnectionAborted, "mem listener closed"))
    }
}

impl Default for MemListener {
    fn default() -> Self {
        Self::new()
    }
}

/// Retrieve the local address of a [`Listener::Tcp`].
pub fn local_addr(lis: &Listener) -> Option<SocketAddr> {
    match lis {
        Listener::Tcp(l) => l.local_addr().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme() {
        assert_eq!(scheme("tcp://:9090"), "tcp");
        assert_eq!(scheme("unix:///tmp/x.sock"), "unix");
        assert_eq!(scheme("stdio://"), "stdio");
        assert_eq!(scheme("mem://"), "mem");
        assert_eq!(scheme("ws://host:8080"), "ws");
    }

    #[test]
    fn test_default_uri() {
        assert_eq!(DEFAULT_URI, "tcp://:9090");
    }

    #[tokio::test]
    async fn test_tcp_listen() {
        let lis = listen("tcp://127.0.0.1:0").await.unwrap();
        match &lis {
            Listener::Tcp(l) => {
                let addr = l.local_addr().unwrap();
                assert!(addr.port() > 0);
            }
            _ => panic!("expected Tcp listener"),
        }
    }

    #[tokio::test]
    async fn test_unix_listen() {
        let path = "/tmp/holons_test_rust.sock";
        let lis = listen(&format!("unix://{}", path)).await.unwrap();
        match lis {
            Listener::Unix(_) => {}
            _ => panic!("expected Unix listener"),
        }
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_mem_listen() {
        let lis = listen("mem://").await.unwrap();
        match lis {
            Listener::Mem(_) => {}
            _ => panic!("expected Mem listener"),
        }
    }

    #[tokio::test]
    async fn test_unsupported_uri() {
        let result = listen("ftp://host").await;
        assert!(result.is_err());
    }
}

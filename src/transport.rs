//! URI-based listener factory for gRPC servers.
//!
//! Supported transport URIs:
//!   - `tcp://<host>:<port>` — TCP socket (default: `tcp://:9090`)
//!   - `unix://<path>`       — Unix domain socket
//!   - `stdio://`            — stdin/stdout pipe
//!   - `mem://`              — in-process duplex channel (testing)
//!   - `ws://<host>:<port>`  — WebSocket URI surface
//!   - `wss://<host>:<port>` — WebSocket-over-TLS URI surface

use std::io;
use std::net::SocketAddr;

use tokio::net::{TcpListener, UnixListener};

/// Default transport URI when --listen is omitted.
pub const DEFAULT_URI: &str = "tcp://:9090";

/// Parsed transport URI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedURI {
    pub raw: String,
    pub scheme: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub secure: bool,
}

/// Listener variants returned by [`listen`].
pub enum Listener {
    Tcp(TcpListener),
    Unix(UnixListener),
    Stdio,
    Mem(MemListener),
    Ws(WSListener),
}

/// Lightweight ws/wss listener metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WSListener {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub secure: bool,
}

/// Parse a transport URI and bind/create the appropriate listener.
pub async fn listen(uri: &str) -> io::Result<Listener> {
    let parsed = parse_uri(uri).map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    match parsed.scheme.as_str() {
        "tcp" => {
            let host = parsed.host.unwrap_or_else(|| "0.0.0.0".to_string());
            let port = parsed.port.unwrap_or(9090);
            let lis = TcpListener::bind(format!("{}:{}", host, port)).await?;
            Ok(Listener::Tcp(lis))
        }
        "unix" => {
            let path = parsed.path.unwrap_or_default();
            let _ = std::fs::remove_file(&path);
            let lis = UnixListener::bind(path)?;
            Ok(Listener::Unix(lis))
        }
        "stdio" => Ok(Listener::Stdio),
        "mem" => Ok(Listener::Mem(MemListener::new())),
        "ws" | "wss" => Ok(Listener::Ws(WSListener {
            host: parsed.host.unwrap_or_else(|| "0.0.0.0".to_string()),
            port: parsed.port.unwrap_or(if parsed.secure { 443 } else { 80 }),
            path: parsed.path.unwrap_or_else(|| "/grpc".to_string()),
            secure: parsed.secure,
        })),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported transport URI: {uri:?}"),
        )),
    }
}

/// Extract the transport scheme from a URI.
pub fn scheme(uri: &str) -> &str {
    uri.find("://").map_or(uri, |i| &uri[..i])
}

/// Parse a URI into a normalized structure.
pub fn parse_uri(uri: &str) -> Result<ParsedURI, String> {
    let s = scheme(uri);

    match s {
        "tcp" => {
            let addr = uri
                .strip_prefix("tcp://")
                .ok_or_else(|| format!("invalid tcp URI: {uri:?}"))?;
            let (host, port) = split_host_port(addr, 9090)?;
            Ok(ParsedURI {
                raw: uri.to_string(),
                scheme: "tcp".to_string(),
                host: Some(host),
                port: Some(port),
                path: None,
                secure: false,
            })
        }
        "unix" => {
            let path = uri
                .strip_prefix("unix://")
                .ok_or_else(|| format!("invalid unix URI: {uri:?}"))?;
            if path.is_empty() {
                return Err(format!("invalid unix URI: {uri:?}"));
            }
            Ok(ParsedURI {
                raw: uri.to_string(),
                scheme: "unix".to_string(),
                host: None,
                port: None,
                path: Some(path.to_string()),
                secure: false,
            })
        }
        "stdio" => Ok(ParsedURI {
            raw: "stdio://".to_string(),
            scheme: "stdio".to_string(),
            host: None,
            port: None,
            path: None,
            secure: false,
        }),
        "mem" => Ok(ParsedURI {
            raw: if uri.starts_with("mem://") {
                uri.to_string()
            } else {
                "mem://".to_string()
            },
            scheme: "mem".to_string(),
            host: None,
            port: None,
            path: None,
            secure: false,
        }),
        "ws" | "wss" => {
            let secure = s == "wss";
            let trimmed = uri
                .strip_prefix(if secure { "wss://" } else { "ws://" })
                .ok_or_else(|| format!("invalid ws URI: {uri:?}"))?;

            let (addr, path) = if let Some((a, p)) = trimmed.split_once('/') {
                (a, format!("/{p}"))
            } else {
                (trimmed, "/grpc".to_string())
            };

            let (host, port) = split_host_port(addr, if secure { 443 } else { 80 })?;

            Ok(ParsedURI {
                raw: uri.to_string(),
                scheme: s.to_string(),
                host: Some(host),
                port: Some(port),
                path: Some(path),
                secure,
            })
        }
        _ => Err(format!("unsupported transport URI: {uri:?}")),
    }
}

/// In-process listener using tokio duplex streams for testing.
pub struct MemListener {
    tx: tokio::sync::mpsc::Sender<tokio::io::DuplexStream>,
    rx: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<tokio::io::DuplexStream>>,
}

impl MemListener {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        Self {
            tx,
            rx: tokio::sync::Mutex::new(rx),
        }
    }

    /// Client side: create a connection to the in-process server.
    pub async fn dial(&self) -> io::Result<tokio::io::DuplexStream> {
        let (client, server) = tokio::io::duplex(1024 * 1024);
        self.tx
            .send(server)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::ConnectionRefused, "mem listener closed"))?;
        Ok(client)
    }

    /// Server side: accept the next in-process connection.
    pub async fn accept(&self) -> io::Result<tokio::io::DuplexStream> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
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

fn split_host_port(value: &str, default_port: u16) -> Result<(String, u16), String> {
    if value.is_empty() {
        return Ok(("0.0.0.0".to_string(), default_port));
    }

    if let Some((host, port_s)) = value.rsplit_once(':') {
        let host = if host.is_empty() { "0.0.0.0" } else { host };
        let port = if port_s.is_empty() {
            default_port
        } else {
            port_s
                .parse::<u16>()
                .map_err(|_| format!("invalid port in URI: {value:?}"))?
        };
        Ok((host.to_string(), port))
    } else {
        Ok((value.to_string(), default_port))
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[test]
    fn test_scheme() {
        assert_eq!(scheme("tcp://:9090"), "tcp");
        assert_eq!(scheme("unix:///tmp/x.sock"), "unix");
        assert_eq!(scheme("stdio://"), "stdio");
        assert_eq!(scheme("mem://"), "mem");
        assert_eq!(scheme("ws://host:8080"), "ws");
        assert_eq!(scheme("wss://host:443"), "wss");
    }

    #[test]
    fn test_default_uri() {
        assert_eq!(DEFAULT_URI, "tcp://:9090");
    }

    #[test]
    fn test_parse_uri_wss() {
        let parsed = parse_uri("wss://example.com:8443").unwrap();
        assert_eq!(parsed.scheme, "wss");
        assert_eq!(parsed.host.as_deref(), Some("example.com"));
        assert_eq!(parsed.port, Some(8443));
        assert_eq!(parsed.path.as_deref(), Some("/grpc"));
        assert!(parsed.secure);
    }

    #[tokio::test]
    async fn test_tcp_listen() {
        let lis = match listen("tcp://127.0.0.1:0").await {
            Ok(v) => v,
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => return,
            Err(err) => panic!("tcp listen failed: {err}"),
        };
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
        let lis = match listen(&format!("unix://{}", path)).await {
            Ok(v) => v,
            Err(err) if err.kind() == io::ErrorKind::PermissionDenied => return,
            Err(err) => panic!("unix listen failed: {err}"),
        };
        match lis {
            Listener::Unix(_) => {}
            _ => panic!("expected Unix listener"),
        }
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn test_mem_roundtrip() {
        let lis = listen("mem://").await.unwrap();
        let mem = match lis {
            Listener::Mem(m) => m,
            _ => panic!("expected Mem listener"),
        };

        let mut client = mem.dial().await.unwrap();
        let mut server = mem.accept().await.unwrap();

        client.write_all(b"hello").await.unwrap();
        client.flush().await.unwrap();

        let mut buf = [0u8; 5];
        server.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"hello");
    }

    #[tokio::test]
    async fn test_ws_surface_listen() {
        let lis = listen("ws://127.0.0.1:8080/grpc").await.unwrap();
        match lis {
            Listener::Ws(ws) => {
                assert_eq!(ws.host, "127.0.0.1");
                assert_eq!(ws.port, 8080);
                assert_eq!(ws.path, "/grpc");
                assert!(!ws.secure);
            }
            _ => panic!("expected Ws listener"),
        }
    }

    #[tokio::test]
    async fn test_unsupported_uri() {
        let result = listen("ftp://host").await;
        assert!(result.is_err());
    }
}

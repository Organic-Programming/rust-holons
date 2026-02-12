//! Standard gRPC server runner for Rust holons.

use crate::transport::DEFAULT_URI;

/// Extract --listen or --port from command-line args.
pub fn parse_flags(args: &[String]) -> String {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--listen" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
        if args[i] == "--port" && i + 1 < args.len() {
            return format!("tcp://:{}", args[i + 1]);
        }
        i += 1;
    }
    DEFAULT_URI.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_listen() {
        let args: Vec<String> = vec!["--listen".into(), "tcp://:8080".into()];
        assert_eq!(parse_flags(&args), "tcp://:8080");
    }

    #[test]
    fn test_parse_port() {
        let args: Vec<String> = vec!["--port".into(), "3000".into()];
        assert_eq!(parse_flags(&args), "tcp://:3000");
    }

    #[test]
    fn test_parse_default() {
        let args: Vec<String> = vec![];
        assert_eq!(parse_flags(&args), DEFAULT_URI);
    }
}

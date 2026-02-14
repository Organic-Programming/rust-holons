use std::fs;
use std::path::PathBuf;

#[test]
fn test_cert_json_declares_level1_executables_and_stdio_dial() {
    let cert_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cert.json");
    let cert_raw = fs::read_to_string(cert_path).unwrap();
    let cert: serde_yaml::Value = serde_yaml::from_str(&cert_raw).unwrap();

    assert_eq!(cert["executables"]["echo_server"], "./bin/echo-server");
    assert_eq!(cert["executables"]["echo_client"], "./bin/echo-client");
    assert_eq!(cert["capabilities"]["grpc_dial_tcp"], true);
    assert_eq!(cert["capabilities"]["grpc_dial_stdio"], true);
}

#[test]
fn test_echo_scripts_exist() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let echo_client = manifest_dir.join("bin/echo-client");
    let echo_server = manifest_dir.join("bin/echo-server");

    assert!(echo_client.is_file());
    assert!(echo_server.is_file());
}

#[cfg(unix)]
#[test]
fn test_echo_scripts_are_executable() {
    use std::os::unix::fs::PermissionsExt;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let echo_client = manifest_dir.join("bin/echo-client");
    let echo_server = manifest_dir.join("bin/echo-server");

    let client_mode = fs::metadata(echo_client).unwrap().permissions().mode();
    let server_mode = fs::metadata(echo_server).unwrap().permissions().mode();

    assert_ne!(client_mode & 0o111, 0);
    assert_ne!(server_mode & 0o111, 0);
}

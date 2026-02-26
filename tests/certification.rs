use std::fs;
use std::path::PathBuf;

fn read_cert_json() -> serde_yaml::Value {
    let cert_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cert.json");
    let cert_raw = fs::read_to_string(cert_path).unwrap();
    serde_yaml::from_str(&cert_raw).unwrap()
}

#[test]
fn test_cert_json_declares_executables_and_connected_capabilities() {
    let cert = read_cert_json();

    assert_eq!(cert["executables"]["echo_server"], "./bin/echo-server");
    assert_eq!(cert["executables"]["echo_client"], "./bin/echo-client");
    assert_eq!(
        cert["executables"]["holon_rpc_client"],
        "./bin/holon-rpc-client"
    );
    assert_eq!(
        cert["executables"]["holon_rpc_server"],
        "./bin/holon-rpc-server"
    );
    assert_eq!(cert["capabilities"]["grpc_dial_tcp"], true);
    assert_eq!(cert["capabilities"]["grpc_dial_stdio"], true);
    assert_eq!(cert["capabilities"]["grpc_dial_ws"], true);
    assert_eq!(cert["capabilities"]["holon_rpc_client"], true);
    assert_eq!(cert["capabilities"]["holon_rpc_server"], true);
}

#[test]
fn test_cert_json_executables_point_to_existing_files() {
    let cert = read_cert_json();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let echo_server = cert["executables"]["echo_server"].as_str().unwrap();
    let echo_client = cert["executables"]["echo_client"].as_str().unwrap();
    let holon_rpc_client = cert["executables"]["holon_rpc_client"].as_str().unwrap();
    let holon_rpc_server = cert["executables"]["holon_rpc_server"].as_str().unwrap();

    assert!(manifest_dir.join(echo_server).is_file());
    assert!(manifest_dir.join(echo_client).is_file());
    assert!(manifest_dir.join(holon_rpc_client).is_file());
    assert!(manifest_dir.join(holon_rpc_server).is_file());
}

#[test]
fn test_echo_scripts_exist() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let echo_client = manifest_dir.join("bin/echo-client");
    let echo_server = manifest_dir.join("bin/echo-server");
    let holon_rpc_client = manifest_dir.join("bin/holon-rpc-client");
    let holon_rpc_server = manifest_dir.join("bin/holon-rpc-server");

    assert!(echo_client.is_file());
    assert!(echo_server.is_file());
    assert!(holon_rpc_client.is_file());
    assert!(holon_rpc_server.is_file());
}

#[test]
fn test_go_helper_files_exist_for_wrappers() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let echo_server_helper = manifest_dir.join("cmd/echo-server-go/main.go");
    let holon_rpc_client_helper = manifest_dir.join("cmd/holon-rpc-client-go/main.go");

    assert!(echo_server_helper.is_file());
    assert!(holon_rpc_client_helper.is_file());
}

#[cfg(unix)]
#[test]
fn test_echo_scripts_are_executable() {
    use std::os::unix::fs::PermissionsExt;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let echo_client = manifest_dir.join("bin/echo-client");
    let echo_server = manifest_dir.join("bin/echo-server");
    let holon_rpc_client = manifest_dir.join("bin/holon-rpc-client");
    let holon_rpc_server = manifest_dir.join("bin/holon-rpc-server");

    let client_mode = fs::metadata(echo_client).unwrap().permissions().mode();
    let server_mode = fs::metadata(echo_server).unwrap().permissions().mode();
    let holon_rpc_client_mode = fs::metadata(holon_rpc_client).unwrap().permissions().mode();
    let holon_rpc_server_mode = fs::metadata(holon_rpc_server).unwrap().permissions().mode();

    assert_ne!(client_mode & 0o111, 0);
    assert_ne!(server_mode & 0o111, 0);
    assert_ne!(holon_rpc_client_mode & 0o111, 0);
    assert_ne!(holon_rpc_server_mode & 0o111, 0);
}

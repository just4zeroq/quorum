use std::path::PathBuf;

fn get_proto_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto files are in the crates directory
    let user_proto = get_proto_dir().join("../../crates/user-service/src/pb/user.proto");
    let order_proto = get_proto_dir().join("../../crates/order-service/src/pb/order_service.proto");
    let auth_proto = get_proto_dir().join("../../crates/auth-service/src/pb/auth_service.proto");

    // Include path must be a prefix of the proto paths
    let include_path = get_proto_dir().join("../../crates");

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .out_dir("src/pb")
        .compile_protos(
            &[&user_proto, &order_proto, &auth_proto],
            &[&include_path],
        )?;

    Ok(())
}
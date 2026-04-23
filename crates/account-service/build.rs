//! Build script for account-service
//!
//! Compiles protobuf definitions using tonic-build

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile protobuf
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/pb")
        .compile_protos(&["src/pb/account.proto"], &["src/pb"])?;

    // Generate descriptor file for gRPC reflection
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/pb/account.desc")
        .compile_protos(&["src/pb/account.proto"], &["src/pb"])?;

    Ok(())
}
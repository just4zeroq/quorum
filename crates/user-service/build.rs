fn main() -> Result<(), Box<dyn std::error::Error>> {
    // CARGO_MANIFEST_DIR for user-service is: /home/ubuntu/code/quorum/crates/user-service
    // We need to output to: /home/ubuntu/code/quorum/crates/api/src

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_src = manifest_dir
        .parent().unwrap()  // crates
        .parent().unwrap()  // quorum
        .join("crates/api/src");

    std::fs::create_dir_all(&api_src)?;

    // Output generated code to crates/api/src
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&api_src)
        .compile_protos(
            &[manifest_dir.join("src/pb/user.proto")],
            &[manifest_dir.join("src/pb")],
        )?;
    Ok(())
}
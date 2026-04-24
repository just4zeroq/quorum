fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let api_src = manifest_dir
        .parent().unwrap()
        .parent().unwrap()
        .join("crates/api/src");
    let pb_src = manifest_dir.join("src/pb");

    std::fs::create_dir_all(&api_src)?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&api_src.join("wallet.desc"))
        .out_dir(&pb_src)
        .compile_protos(
            &[manifest_dir.join("src/pb/wallet.proto")],
            &[manifest_dir.join("src/pb")],
        )?;
    Ok(())
}

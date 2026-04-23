fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/pb")
        .file_descriptor_set_path("src/pb/auth_service.desc")
        .compile_protos(&["src/pb/auth_service.proto"], &["src/pb"])?;
    Ok(())
}

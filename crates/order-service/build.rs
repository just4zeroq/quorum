fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/pb")
        .file_descriptor_set_path("src/pb/order_service.desc")
        .compile_protos(&["src/pb/order_service.proto"], &["src/pb"])?;
    Ok(())
}
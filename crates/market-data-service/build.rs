fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/pb")
        .file_descriptor_set_path("src/pb/market_data.desc")
        .compile_protos(
            &["src/pb/market_data.proto"],
            &["src/pb"],
        )?;
    Ok(())
}
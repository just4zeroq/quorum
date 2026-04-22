fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/pb")
        .file_descriptor_set_path("src/pb/prediction_market.desc")
        .compile_protos(
            &["src/pb/prediction_market.proto"],
            &["src/pb"],
        )?;
    Ok(())
}
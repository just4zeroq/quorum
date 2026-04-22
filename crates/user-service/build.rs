fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src/pb")
        .compile_protos(
            &["src/pb/user.proto"],
            &["src/pb"],
        )?;
    Ok(())
}
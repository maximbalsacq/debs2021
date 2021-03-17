fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::configure()
        .out_dir("src/gen")
        .compile(&["proto/challenger.proto"], &["proto/"])?;
    Ok(())
}

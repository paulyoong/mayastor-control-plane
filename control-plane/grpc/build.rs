extern crate tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["protos/common.proto", "protos/pool.proto"], &["protos"])?;
    Ok(())
}

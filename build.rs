fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building Xiron!");
    tonic_build::compile_protos("proto/interface.proto")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::var("PROTO_ROOT").unwrap_or_else(|_| String::from("../../proto"));
    let contract = format!("{root}/hello/v1/hello.proto");
    println!("cargo:rerun-if-changed={contract}");
    println!("cargo:rerun-if-env-changed=PROTO_ROOT");
    tonic_prost_build::configure().compile_protos(&[contract], &[root])?;
    Ok(())
}

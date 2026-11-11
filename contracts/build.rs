//! Generates Rust code from every contract in `proto/`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto");
    tonic_prost_build::configure()
        .compile_protos(&["../proto/hello/v1/hello.proto"], &["../proto"])?;
    Ok(())
}

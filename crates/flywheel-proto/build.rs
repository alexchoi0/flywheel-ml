fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = [
        "proto/inference.proto",
        "proto/control.proto",
        "proto/health.proto",
    ];

    for proto in &proto_files {
        println!("cargo:rerun-if-changed={}", proto);
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&proto_files, &["proto"])?;

    Ok(())
}

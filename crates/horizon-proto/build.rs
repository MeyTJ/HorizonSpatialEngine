fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "proto/spatial/v1/spatial.proto",
                "proto/spatial_compute/v1/spatial_compute.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}

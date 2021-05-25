fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = std::path::PathBuf::from("proto/address-book/address-book.proto");
    let proto_path = proto.as_path();
    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&[proto_path], &[proto_dir])?;
    Ok(())
}

mod client;
mod proto;
use crate::{client::generate_client, proto::compile_proto, proto::generate_protos};

fn main() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let proto_files = std::fs::read_dir(&current_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.extension().and_then(|ext| ext.to_str()) == Some("proto")
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    let mut src_dir = current_dir.clone();
    src_dir.push("src");

    if !proto_files.is_empty() {
        // Generate protobuf code into src/proto/
        let file_descriptor = compile_proto(&proto_files[0])?;

        let package_name = file_descriptor
            .file
            .iter()
            .find_map(|f| {
                f.name
                    .as_deref()
                    .filter(|n| n.ends_with("api.proto"))
                    .and_then(|_| f.package.clone())
            })
            .expect("Proto file must have a package name");

        generate_protos(
            src_dir.clone(),
            &proto_files,
            current_dir.clone(),
            &package_name,
        );

        // Generate custom client code into src/client.rs
        for proto_path in proto_files {
            let fds = compile_proto(&proto_path)?;
            generate_client(&src_dir, &current_dir, &fds)?;
        }
    }

    Ok(())
}

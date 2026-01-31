mod client;
mod proto;
mod utils;
use crate::{client::generate_client, proto::generate_protos, utils::compile_proto};

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
        // Compile proto to get the file descriptor
        let fds = compile_proto(&proto_files[0])?;

        // Extract package name from the file descriptor
        let package_name = fds
            .file
            .iter()
            .find(|f| {
                f.name
                    .as_ref()
                    .map(|n| n.ends_with("api.proto"))
                    .unwrap_or(false)
            })
            .and_then(|f| f.package.clone())
            .expect("Proto file must have a package name");

        // Generate protobuf code into src/proto/
        generate_protos(
            src_dir.clone(),
            &proto_files,
            current_dir.clone(),
            &package_name,
        );

        // Generate custom client code
        for proto_path in proto_files {
            let fds = compile_proto(&proto_path)?;
            generate_client(&src_dir, &current_dir, &fds)?;
        }
    }

    Ok(())
}

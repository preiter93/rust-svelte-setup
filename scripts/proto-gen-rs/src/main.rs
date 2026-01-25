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
        // Generate code from proto
        generate_protos(src_dir.clone(), &proto_files, current_dir.clone());

        // Generate custom client code
        for proto_path in proto_files {
            let fds = compile_proto(&proto_path)?;
            generate_client(&src_dir, &current_dir, &fds)?;
        }
    }

    Ok(())
}

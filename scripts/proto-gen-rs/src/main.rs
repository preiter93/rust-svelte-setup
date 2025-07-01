fn main() -> std::io::Result<()> {
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
        tonic_build::configure()
            // .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
            .out_dir(&src_dir)
            .compile(&proto_files, &[current_dir])
            .expect("Failed to compile protos");
    }

    Ok(())
}

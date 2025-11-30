use anyhow::Result;
use anyhow::anyhow;
use prost_types::FileDescriptorSet;
use protox::compile;
use std::path::Path;

pub fn compile_proto(proto_path: &Path) -> Result<FileDescriptorSet> {
    let dir = proto_path.parent().unwrap();
    let file_str = proto_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid proto filename"))?;

    let protos = vec![file_str.to_string()];
    let includes = vec![dir.to_string_lossy().to_string()];

    let fds: FileDescriptorSet = compile(&protos, &includes)
        .map_err(|e| anyhow!("protox compile error on {:?}: {}", proto_path, e))?;

    Ok(fds)
}

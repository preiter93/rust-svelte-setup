use std::path::Path;

pub(crate) fn generate_protos<P: AsRef<Path>>(src_dir: P, proto_files: &[P], current_dir: P) {
    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_well_known_types(false)
        .extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
        .out_dir(src_dir)
        .compile_protos(&proto_files, &[current_dir])
        .expect("Failed to compile protos");
}

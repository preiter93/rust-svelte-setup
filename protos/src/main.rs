fn main() {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("../service-users/src/")
        .compile(&["./users.proto"], &["./"])
        .expect("Failed to compile users protos");
}

use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("No OUT_DIR"));
    let proto_path = PathBuf::from("./guest_agent.proto");
    let proto_parent_path = proto_path.parent().unwrap().to_owned();

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("guest_agent_descriptor.bin"))
        .compile_protos(&[proto_path], &[proto_parent_path])
        .expect("Could not compile Protobuf descriptor");
}

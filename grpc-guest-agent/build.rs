fn main() {
    tonic_build::compile_protos("./guest_agent.proto")
        .expect("Compiling Protobuf defintion failed");
}

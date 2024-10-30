/* Prebuild script */
fn main() {
    println!("cargo:rerun-if-changed=src/proto");

    protobuf_codegen::Codegen::new()
        .pure()
        // All inputs and imports from the inputs must reside in `includes` directories.
        .includes(["src/proto"])
        // Inputs must reside in some of include paths.
        .input("src/proto/serverdata.proto")
        // Specify output directory relative to Cargo output directory.
        .out_dir("src")
        .run_from_script();
}

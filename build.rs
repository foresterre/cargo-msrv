use vergen::EmitBuilder;

fn main() {
    // generate build info
    if let Err(e) = EmitBuilder::builder()
        .cargo_target_triple()
        .cargo_features()
        .git_sha(true)
        .rustc_semver()
        .emit()
    {
        eprintln!("Unable to set build metadata: '{e}'");
    }
}

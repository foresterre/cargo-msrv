use vergen::{Cargo, Emitter, Rustc};
use vergen_gitcl::Gitcl;

fn main() {
    // generate build info
    if let Err(e) = emit() {
        eprintln!("Unable to set build metadata: '{e}'");
    }
}

fn emit() -> Result<(), Box<dyn std::error::Error>> {
    let cargo = Cargo::builder().target_triple(true).features(true).build();
    let rustc = Rustc::builder().semver(true).build();
    let gitcl = Gitcl::builder().sha(true).build();

    Emitter::default()
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&gitcl)?
        .emit()?;

    Ok(())
}

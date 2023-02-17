use vergen::{vergen, Config, ShaKind};

fn main() {
    // generate build info
    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    if let Err(e) = vergen(config) {
        eprintln!("Unable to set build metadata: '{}'", e);
    }
}

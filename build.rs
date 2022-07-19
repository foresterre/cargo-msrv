use vergen::{vergen, Config, ShaKind};

fn main() {
    // generate build info
    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    vergen(config).expect("Unable to generate build info with 'vergen'");
}

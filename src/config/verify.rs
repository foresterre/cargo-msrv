use crate::manifest::bare_version::BareVersion;

#[derive(Clone, Debug)]
pub struct VerifyCmdConfig {
    pub rust_version: Option<BareVersion>,
}

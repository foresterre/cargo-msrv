use crate::manifest::bare_version::BareVersion;

#[derive(Clone, Debug)]
pub struct SetCmdConfig {
    pub msrv: BareVersion,
}

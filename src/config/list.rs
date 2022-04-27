use clap::ArgEnum;

#[derive(Clone, Debug)]
pub struct ListCmdConfig {
    pub variant: ListMsrvVariant,
}

#[derive(Copy, Clone, Debug, ArgEnum)]
pub enum ListMsrvVariant {
    DirectDeps,
    OrderedByMsrv,
}

impl Default for ListMsrvVariant {
    fn default() -> Self {
        Self::OrderedByMsrv
    }
}

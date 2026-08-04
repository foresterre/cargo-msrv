use crate::values::{CliValue, CliValues};
use cargo_msrv_context::types::{DIRECT_DEPS, ListMsrvVariant, ORDERED_BY_MSRV};

pub const VALUES: CliValues<ListMsrvVariant> = CliValues::new(&[
    CliValue::new(DIRECT_DEPS, ListMsrvVariant::DirectDeps),
    CliValue::new(ORDERED_BY_MSRV, ListMsrvVariant::OrderedByMSRV),
]);

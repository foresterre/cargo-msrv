use std::convert::TryInto;

fn some() {
    // Rust >= 1.56
    let _: u8 = 1u32.try_into().unwrap();
}

#[cfg(feature = "unrequired_feature")]
fn compile() {
    compile_error!("If the 'unrequired_feature' is set, compiling will fail!");
}

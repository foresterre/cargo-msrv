#[cfg(feature = "required_feature")]
fn compile() {
    // Rust >= 1.56
    let _: u8 = 1u32.try_into().unwrap();
}

#[cfg(not(feature = "required_feature"))]
fn compile() {
    compile_error!("Requires 'required_feature' to compile!");
}

use cargo_msrv::run_cargo_msrv;

fn main() {
    if let Err(err) = run_cargo_msrv() {
        println!("{}", err);
    }
}

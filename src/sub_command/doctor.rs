use crate::context::DoctorContext;
use crate::error::TResult;
use crate::reporter::Reporter;
use crate::SubCommand;

#[derive(Default)]
pub struct Doctor;

impl SubCommand for Doctor {
    type Context = DoctorContext;
    type Output = ();

    fn run(&self, _ctx: &Self::Context, _reporter: &impl Reporter) -> TResult<Self::Output> {
        todo!("Implement cargo msrv doctor!")
    }
}

#[derive(serde::Serialize)]
pub struct Message {
    sub_command: SubCommand,
}

#[derive(serde::Serialize)]
pub enum SubCommand {
    Find,
}

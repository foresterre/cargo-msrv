use std::fmt;

// #[derive(serde::Serialize)]
// pub struct Message {
//     #[serde(flatten)]
//     inner: InnerMessage,
// }
//
// impl Message {
//     pub fn new(status: Status, message: impl Into<String>) -> Self {
//         Self {
//             inner: InnerMessage {
//                 status,
//                 message: message.into(),
//             },
//         }
//     }
// }
//
// #[derive(serde::Serialize)]
// struct InnerMessage {
//     status: Status,  // e.g. Compiling, Downloading, ...
//     message: String, //
// }
//

// #[derive(serde::Serialize)]
// #[serde(rename_all = "snake_case")]
// enum Status {
//     Fetching,
//     Downloading,
//     Checking,
// }
//
//
//
// impl From<Status> for &'static str {
//     fn from(it: Status) -> Self {
//         match it {
//             Status::Fetching => "Fetching",
//             Status::Downloading => "Downloading",
//             Status::Checking => "Checking",
//         }
//     }
// }

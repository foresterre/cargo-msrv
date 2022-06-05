use owo_colors::OwoColorize;

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    FetchingIndex,
}

impl From<Action> for String {
    fn from(action: Action) -> Self {
        match action {
            Action::FetchingIndex => {
                HumanStatusMessage::new(Status::Fetching).fmt("Obtaining rust-releases index")
            }
        }
    }
}

#[derive(serde::Serialize)]
struct HumanStatusMessage {
    status: Status, // e.g. Compiling, Downloading, ...
}

impl HumanStatusMessage {
    pub fn new(status: Status) -> Self {
        Self { status }
    }

    pub fn fmt<'text>(&self, message: impl Into<&'text str>) -> String {
        format!("{:>12} {}", self.status.as_str().green(), message.into(),)
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Fetching,
    Downloading,
    Checking,
}

impl Status {
    fn as_str(&self) -> &'static str {
        match self {
            Status::Fetching => "Fetching",
            Status::Downloading => "Downloading",
            Status::Checking => "Checking",
        }
    }
}
